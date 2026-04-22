//! Export coordination and PST message extraction.
//!
//! This module orchestrates the full export pipeline: opening PST files,
//! traversing folder hierarchies, extracting message data (body, metadata,
//! attachments, headers), running duplicate detection and keyword/email
//! filtering, and writing output files via the exporter.

use crate::cli::progress::ProgressReporter;
use crate::cli::ExportArgs;
use crate::duplicate::DuplicateTracker;
use crate::error::Result;
use crate::export::conversation::{assign_conversation_folders, ConversationCandidate};
use crate::export::csv::{CsvExporter, CsvRow};
use crate::export::exporter::{Attachment, MessageData};
use crate::filter::email::EmailMatcher;
use crate::filter::keyword::KeywordMatcher;
use chrono::TimeZone;
use outlook_pst::ltp::prop_context::PropertyValue;
use outlook_pst::messaging::folder::Folder;
use outlook_pst::messaging::message::Message;
use outlook_pst::messaging::store::Store;
use outlook_pst::ndb::node_id::NodeId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub mod conversation;
pub mod csv;
pub mod exporter;
pub mod html;
pub mod metadata;

use exporter::MessageExporter;

#[derive(Debug, Clone)]
struct StagedExportRecord {
    sequence_number: u32,
    message_data: MessageData,
    pst_store_name: String,
    is_duplicate: bool,
    matched_keywords: Vec<String>,
    matched_emails: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct ExportArtifactOptions {
    metadata: bool,
    attachments: bool,
    headers: bool,
}

/// Extension trait for convenient value extraction from [`PropertyValue`].
///
/// Eliminates repetitive match arms when extracting typed data from PST properties.
trait PropertyValueExt {
    /// Extract a string from String8 or Unicode variants.
    fn as_string(&self) -> Option<String>;
    /// Extract raw bytes from a Binary variant.
    fn as_binary(&self) -> Option<Vec<u8>>;
    /// Extract an i32 from an Integer32 variant.
    fn as_i32(&self) -> Option<i32>;
    /// Extract a string, also accepting Binary (interpreted as UTF-8).
    fn as_string_or_binary_utf8(&self) -> Option<String>;
}

impl PropertyValueExt for PropertyValue {
    fn as_string(&self) -> Option<String> {
        match self {
            PropertyValue::String8(s) => Some(s.to_string()),
            PropertyValue::Unicode(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn as_binary(&self) -> Option<Vec<u8>> {
        match self {
            PropertyValue::Binary(b) => Some(b.buffer().to_vec()),
            _ => None,
        }
    }

    fn as_i32(&self) -> Option<i32> {
        match self {
            PropertyValue::Integer32(i) => Some(*i),
            _ => None,
        }
    }

    fn as_string_or_binary_utf8(&self) -> Option<String> {
        match self {
            PropertyValue::String8(s) => Some(s.to_string()),
            PropertyValue::Unicode(s) => Some(s.to_string()),
            PropertyValue::Binary(b) => decode_html_binary(b.buffer()),
            _ => None,
        }
    }
}

fn decode_html_binary(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Some(s.trim_end_matches('\0').to_string());
    }

    // Handle UTF-16 with BOM
    if bytes.len() >= 2 {
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return Some(decode_utf16le_without_bom(&bytes[2..]));
        }
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return Some(decode_utf16be_without_bom(&bytes[2..]));
        }
    }

    // Heuristic UTF-16 detection for BOM-less content
    let even_nuls = bytes.iter().step_by(2).filter(|&&b| b == 0).count();
    let odd_nuls = bytes.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
    let half_len = bytes.len() / 2;

    if half_len > 0 {
        if odd_nuls > half_len / 3 {
            return Some(decode_utf16le_without_bom(bytes));
        }
        if even_nuls > half_len / 3 {
            return Some(decode_utf16be_without_bom(bytes));
        }
    }

    // MAPI HTML body payloads are frequently ANSI codepage bytes; use CP-1252 fallback.
    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    Some(decoded.trim_end_matches('\0').to_string())
}

fn decode_utf16le_without_bom(bytes: &[u8]) -> String {
    let even_len = bytes.len() - (bytes.len() % 2);
    let (decoded, _) = encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes[..even_len]);
    decoded.trim_end_matches('\0').to_string()
}

fn decode_utf16be_without_bom(bytes: &[u8]) -> String {
    let even_len = bytes.len() - (bytes.len() % 2);
    let (decoded, _) = encoding_rs::UTF_16BE.decode_without_bom_handling(&bytes[..even_len]);
    decoded.trim_end_matches('\0').to_string()
}

fn filetime_to_rfc3339(filetime: i64) -> Option<String> {
    const WINDOWS_TO_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;
    const HUNDRED_NS_PER_SECOND: i64 = 10_000_000;

    let unix_100ns = filetime.checked_sub(WINDOWS_TO_UNIX_EPOCH_100NS)?;
    let seconds = unix_100ns.div_euclid(HUNDRED_NS_PER_SECOND);
    let sub_second_100ns = unix_100ns.rem_euclid(HUNDRED_NS_PER_SECOND);
    let nanos = u32::try_from(sub_second_100ns.checked_mul(100)?).ok()?;

    chrono::Utc
        .timestamp_opt(seconds, nanos)
        .single()
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
}

fn build_folder_path(parent_path: Option<&str>, folder_name: &str) -> String {
    match parent_path {
        Some(parent) if !parent.is_empty() => format!("{parent}/{folder_name}"),
        _ => folder_name.to_string(),
    }
}

fn build_conversation_assignments(staged_exports: &[StagedExportRecord]) -> HashMap<u32, String> {
    let mut main_candidates = Vec::new();
    let mut duplicate_candidates = Vec::new();

    for record in staged_exports {
        let candidate = ConversationCandidate {
            sequence_number: record.sequence_number,
            conversation_id: record.message_data.conversation_id.clone(),
            conversation_index: record.message_data.conversation_index.clone(),
        };

        if record.is_duplicate {
            duplicate_candidates.push(candidate);
        } else {
            main_candidates.push(candidate);
        }
    }

    // Conversation grouping must be scoped by output root (main vs duplicates)
    // so every conv_XXXXX folder has at least two members in that root.
    let mut assignments = assign_conversation_folders(&main_candidates);
    assignments.extend(assign_conversation_folders(&duplicate_candidates));
    assignments
}

fn conversation_number_from_folder(conversation_folder: Option<&str>) -> String {
    conversation_folder
        .and_then(|folder| folder.strip_prefix("conv_"))
        .unwrap_or_default()
        .to_string()
}

/// Export coordinator managing the overall export workflow
#[derive(Debug)]
pub struct ExportCoordinator {
    /// Export arguments from CLI
    args: ExportArgs,
    /// Next sequence number for messages
    next_sequence: u32,
    /// Duplicate tracker for deduplication
    duplicate_tracker: DuplicateTracker,
    /// Count of duplicates found
    duplicate_count: usize,
    /// CSV exporter (if --csv flag enabled)
    csv_exporter: Option<CsvExporter>,
    /// Keyword matcher (if --keywords flag enabled)
    keyword_matcher: Option<KeywordMatcher>,
    /// Email matcher (if --emails flag enabled)
    email_matcher: Option<EmailMatcher>,
    /// Export records staged before final path assignment and filesystem writes.
    staged_exports: Vec<StagedExportRecord>,
}

impl ExportCoordinator {
    /// Create a new export coordinator
    #[must_use]
    pub fn new(args: ExportArgs) -> Self {
        // Initialize keyword matcher if keywords provided
        let keyword_matcher = args
            .keywords
            .as_ref()
            .map(|kws| KeywordMatcher::new(kws.clone()));

        // Initialize email matcher if emails provided
        let email_matcher = args
            .emails
            .as_ref()
            .map(|emails| EmailMatcher::new(emails.clone()));

        Self {
            args,
            next_sequence: 1,
            duplicate_tracker: DuplicateTracker::new(),
            duplicate_count: 0,
            csv_exporter: None,
            keyword_matcher,
            email_matcher,
            staged_exports: Vec::new(),
        }
    }

    /// Get the next sequence number and increment counter
    pub fn next_sequence_number(&mut self) -> u32 {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        seq
    }

    /// Format a sequence number as zero-padded 5-digit string (00001, 00002, etc.)
    #[must_use]
    pub fn format_sequence(sequence: u32) -> String {
        format!("{sequence:05}")
    }

    /// Get output directory for a message (main or duplicates/)
    #[must_use]
    pub fn get_message_output_dir(&self, sequence: u32, is_duplicate: bool) -> PathBuf {
        let seq_str = Self::format_sequence(sequence);
        let mut path = self.args.output.clone();

        if is_duplicate {
            path.push("duplicates");
        }

        path.push(seq_str);
        path
    }

    /// Run the export operation
    ///
    /// # Errors
    ///
    /// Returns an error if the input path doesn't exist, the output directory
    /// cannot be created, or a critical PST processing error occurs.
    pub fn run(&mut self, reporter: &mut ProgressReporter) -> Result<()> {
        // Validate input path exists
        if !self.args.input.exists() {
            return Err(crate::error::Error::pst_not_found(&self.args.input));
        }

        // Validate output directory is writable
        if self.args.output.exists() && !self.args.output.is_dir() {
            return Err(crate::error::Error::output_not_writable(&self.args.output));
        }

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.args.output)?;

        // Initialize CSV exporter if --csv flag is set
        if self.args.csv {
            let csv_path = self.args.output.join("emails.csv");
            let mut csv_exporter = CsvExporter::new(&csv_path)?;
            csv_exporter.write_header()?;
            self.csv_exporter = Some(csv_exporter);
        }

        // Determine if input is file or directory
        if self.args.input.is_file() {
            // Single file export
            let input_file = self.args.input.clone();
            self.export_pst_file(&input_file, reporter)?;
        } else if self.args.input.is_dir() {
            // Directory export - process all PST files alphabetically
            let input_dir = self.args.input.clone();
            let mut pst_files: Vec<_> = std::fs::read_dir(&input_dir)?
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        let path = e.path();
                        if path.extension().is_some_and(|ext| ext == "pst") {
                            Some(path)
                        } else {
                            None
                        }
                    })
                })
                .collect();

            pst_files.sort();

            // Process each PST file, continuing even if some fail
            let mut errors_encountered = 0;
            for pst_file in pst_files {
                match self.export_pst_file(&pst_file, reporter) {
                    Ok(()) => {
                        // Success - continue to next file
                    }
                    Err(e) => {
                        // Log error to stderr but continue processing
                        eprintln!("⚠️  Error processing {}: {}", pst_file.display(), e);
                        eprintln!("   Continuing with remaining files...");
                        errors_encountered += 1;
                    }
                }
            }

            // Report summary of errors if any occurred
            if errors_encountered > 0 {
                eprintln!();
                eprintln!(
                    "⚠️  {errors_encountered} file(s) failed to process but export continued"
                );
            }
        }

        self.write_staged_exports(reporter);

        // Flush and close CSV if enabled
        if let Some(ref mut csv) = self.csv_exporter {
            csv.flush()?;
        }

        Ok(())
    }

    /// Export messages from a single PST file
    fn export_pst_file(
        &mut self,
        pst_path: &std::path::Path,
        reporter: &mut ProgressReporter,
    ) -> Result<()> {
        let pst_store_name = pst_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        // Open PST store
        let store = outlook_pst::open_store(pst_path).map_err(|e| {
            crate::error::Error::Other(anyhow::anyhow!(
                "Failed to open PST file {}: {}",
                pst_path.display(),
                e
            ))
        })?;

        // Get properties
        let properties = store.properties();

        // Get IPM subtree (main folder structure)
        let ipm_entry_id = properties.ipm_sub_tree_entry_id().map_err(|e| {
            crate::error::Error::Other(anyhow::anyhow!(
                "Failed to get IPM subtree from {}: {}",
                pst_path.display(),
                e
            ))
        })?;

        // Open the root folder
        let root_folder = store.open_folder(&ipm_entry_id).map_err(|e| {
            crate::error::Error::Other(anyhow::anyhow!("Failed to open root folder: {e}"))
        })?;

        // Track staged messages before this PST.
        let staged_before = self.staged_exports.len();

        // Recursively collect messages from all folders.
        self.export_folder_messages(&store, &root_folder, reporter, None, &pst_store_name);

        // Update total messages in reporter
        let messages_exported = self.staged_exports.len() - staged_before;
        reporter.set_total_messages((self.next_sequence - 1) as usize);

        eprintln!(
            "✅ Exported {} messages from {}",
            messages_exported,
            pst_path.display()
        );

        Ok(())
    }

    /// Recursively export messages from a folder and its subfolders
    #[allow(clippy::too_many_lines)]
    fn export_folder_messages(
        &mut self,
        store: &Rc<dyn Store>,
        folder: &Rc<dyn Folder>,
        reporter: &mut ProgressReporter,
        parent_folder_path: Option<&str>,
        pst_store_name: &str,
    ) {
        let folder_name = folder
            .properties()
            .display_name()
            .unwrap_or_else(|_| "Unknown".to_string());
        let folder_path = build_folder_path(parent_folder_path, &folder_name);

        // Export messages in this folder if it has a contents table
        if let Some(contents_table) = folder.contents_table() {
            // Process each message in the folder
            for row in contents_table.rows_matrix() {
                // Create entry ID for this message
                let node_id = NodeId::from(u32::from(row.id()));
                let entry_id = match store.properties().make_entry_id(node_id) {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("Warning: Failed to create entry ID for message: {e}");
                        continue;
                    }
                };

                // Open and read the message
                match store.open_message(&entry_id, None) {
                    Ok(message) => {
                        let seq = self.next_sequence_number();

                        // Extract message data from PST message object
                        let message_data = extract_message_data(&message, &folder_path);

                        let should_export = self.args.drafts || !message_data.is_draft;

                        let mut is_duplicate = false;

                        if should_export {
                            // Check for duplicates among exported messages
                            let identifier =
                                crate::duplicate::generate_message_identifier(&message_data);
                            let (dup, _first_seq) =
                                self.duplicate_tracker.check_and_record(&identifier, seq);
                            is_duplicate = dup;

                            if is_duplicate {
                                self.duplicate_count += 1;
                                reporter.record_duplicate();
                            }
                        } else {
                            reporter.record_draft_skipped();
                        }

                        // Perform keyword matching if enabled.
                        // For HTML bodies, extract visible text so keywords in
                        // tags, comments, <script>, and <style> do not match.
                        let matched_keywords: Vec<String> = if let Some(kw_matcher) =
                            &self.keyword_matcher
                        {
                            let body_text: Option<String> =
                                if let Some(ref html_body) = message_data.body_html {
                                    Some(html::extract_visible_text(html_body))
                                } else {
                                    message_data.body_plain.clone()
                                };
                            let kw_hits = kw_matcher
                                .search_message(Some(&message_data.subject), body_text.as_deref());
                            let mut kw_list: Vec<_> = kw_hits.into_iter().collect();
                            kw_list.sort();
                            kw_list
                        } else {
                            vec![]
                        };

                        // Perform email matching if enabled
                        let matched_emails: Vec<String> =
                            if let Some(em_matcher) = &self.email_matcher {
                                let em_hits = em_matcher.search_message(
                                    &message_data.from,
                                    &message_data.to,
                                    &message_data.cc,
                                    &message_data.bcc,
                                );
                                let mut em_list: Vec<_> = em_hits.into_iter().collect();
                                em_list.sort();
                                em_list
                            } else {
                                vec![]
                            };

                        if should_export {
                            self.staged_exports.push(StagedExportRecord {
                                sequence_number: seq,
                                message_data,
                                pst_store_name: pst_store_name.to_string(),
                                is_duplicate,
                                matched_keywords,
                                matched_emails,
                            });
                        }
                    }
                    Err(e) => {
                        reporter.record_error();
                        eprintln!(
                            "Warning: Failed to open message with entry ID {entry_id:?}: {e}"
                        );
                    }
                }
            }
        }

        // Process subfolders via hierarchy table
        if let Some(hierarchy_table) = folder.hierarchy_table() {
            let mut subfolders = Vec::new();

            // Collect subfolders from hierarchy table
            for row in hierarchy_table.rows_matrix() {
                // Get the NodeId from the row ID
                let node_id = NodeId::from(u32::from(row.id()));

                // Convert to EntryId using store properties
                if let Ok(entry_id) = store.properties().make_entry_id(node_id) {
                    subfolders.push(entry_id);
                }
            }

            // Process subfolders
            for entry_id in subfolders {
                if let Ok(subfolder) = store.open_folder(&entry_id) {
                    self.export_folder_messages(
                        store,
                        &subfolder,
                        reporter,
                        Some(folder_path.as_str()),
                        pst_store_name,
                    );
                }
            }
        }
    }

    fn write_staged_exports(&mut self, reporter: &mut ProgressReporter) {
        if self.staged_exports.is_empty() {
            return;
        }

        let exporter = MessageExporter::new(self.args.output.clone());
        let artifact_options = ExportArtifactOptions {
            metadata: self.args.metadata,
            attachments: self.args.attachments,
            headers: self.args.headers,
        };
        let mut conversation_assignments = std::collections::HashMap::new();

        if self.args.conversations {
            conversation_assignments = build_conversation_assignments(&self.staged_exports);
        }

        let csv_exporter = &mut self.csv_exporter;
        for record in &self.staged_exports {
            let conversation_folder = conversation_assignments
                .get(&record.sequence_number)
                .map(std::string::String::as_str);
            Self::write_staged_export_record(
                &exporter,
                csv_exporter,
                artifact_options,
                record,
                conversation_folder,
                reporter,
            );
        }
    }

    fn write_staged_export_record(
        exporter: &MessageExporter,
        csv_exporter: &mut Option<CsvExporter>,
        artifact_options: ExportArtifactOptions,
        record: &StagedExportRecord,
        conversation_folder: Option<&str>,
        reporter: &mut ProgressReporter,
    ) {
        let conv_number = conversation_number_from_folder(conversation_folder);
        let mut error = 0_u8;

        // Build attachment plan once per message; reused for both HTML
        // rewriting and file output to keep filenames in sync.
        let attachment_plan = exporter::build_attachment_plan(&record.message_data.attachments);
        let plan_for_html = if artifact_options.attachments {
            Some(&attachment_plan)
        } else {
            None
        };

        if let Err(e) = exporter.export_message(
            &record.message_data,
            record.sequence_number,
            record.is_duplicate,
            conversation_folder,
            plan_for_html,
        ) {
            reporter.record_error();
            error = 1;
            eprintln!(
                "Warning: Failed to export message {}: {}",
                record.sequence_number, e
            );
        }

        if artifact_options.metadata {
            if let Err(e) = exporter.write_metadata(
                &record.message_data,
                record.sequence_number,
                record.is_duplicate,
                conversation_folder,
                &record.matched_keywords,
                &record.matched_emails,
            ) {
                error = 1;
                eprintln!(
                    "Warning: Failed to export metadata for message {}: {}",
                    record.sequence_number, e
                );
            }
        }

        if artifact_options.attachments {
            if let Err(e) = exporter.write_attachments(
                &record.message_data,
                record.sequence_number,
                record.is_duplicate,
                conversation_folder,
                &attachment_plan,
            ) {
                error = 1;
                eprintln!(
                    "Warning: Failed to export attachments for message {}: {}",
                    record.sequence_number, e
                );
            }
        }

        if artifact_options.headers {
            if let Err(e) = exporter.write_headers(
                &record.message_data,
                record.sequence_number,
                record.is_duplicate,
                conversation_folder,
            ) {
                error = 1;
                eprintln!(
                    "Warning: Failed to export headers for message {}: {}",
                    record.sequence_number, e
                );
            }
        }

        if let Some(ref mut csv) = csv_exporter {
            let csv_row = CsvRow {
                sequence_number: record.sequence_number,
                subject: record.message_data.subject.clone(),
                from: record.message_data.from.clone(),
                to: record.message_data.to.join("; "),
                date: record.message_data.date.clone(),
                message_id: record.message_data.message_id.clone().unwrap_or_default(),
                is_duplicate: record.is_duplicate,
                keyword_count: record.matched_keywords.len(),
                email_match_count: record.matched_emails.len(),
                size: record.message_data.size_bytes.unwrap_or(0),
                attachment_count: record.message_data.attachments.len(),
                conv_number,
                pst_store_name: record.pst_store_name.clone(),
                error,
            };

            if let Err(e) = csv.write_row(&csv_row) {
                eprintln!(
                    "Warning: Failed to write CSV row for message {}: {}",
                    record.sequence_number, e
                );
            }
        }
    }
}

/// Extract message data from a PST message object into our export format
#[allow(
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
fn extract_message_data(message: &Rc<dyn Message>, folder_path: &str) -> MessageData {
    let props = message.properties();

    // Extract message flags value (0x0E07 - PidTagMessageFlags)
    // mfUnsent is bit 0x00000008
    let flags_val = props
        .get(0x0E07)
        .and_then(PropertyValueExt::as_i32)
        .unwrap_or(0);
    let is_draft = flags_val & 0x00000008 != 0;

    // Extract subject
    let mut subject = props
        .get(0x0037) // PidTagSubject
        .and_then(PropertyValueExt::as_string)
        .unwrap_or_else(|| "(No Subject)".to_string());

    if is_draft {
        subject = format!("[DRAFT] {subject}");
    }

    // Extract sender: display name (0x0C1A) + SMTP address (0x5D01 → 0x5D0A → 0x0C1F fallback)
    let sender_name = props.get(0x0C1A).and_then(PropertyValueExt::as_string); // PidTagSenderName
    let sender_smtp = props
        .get(0x5D01)
        .and_then(PropertyValueExt::as_string) // PidTagSenderSmtpAddress
        .filter(|s| is_smtp_address(s))
        .or_else(|| {
            props
                .get(0x5D0A)
                .and_then(PropertyValueExt::as_string) // PidTagCreatorSmtpAddress (sender)
                .filter(|s| is_smtp_address(s))
        })
        .or_else(|| {
            props
                .get(0x0C1F)
                .and_then(PropertyValueExt::as_string) // PidTagSenderEmailAddress
                .filter(|s| is_smtp_address(s))
        });
    let from = format_email_field(sender_name.as_deref(), sender_smtp.as_deref());

    // Extract message ID
    let message_id = props.get(0x1035).and_then(PropertyValueExt::as_string); // PidTagInternetMessageId

    // Extract conversation fields
    let conversation_id = props
        .get(0x3013) // PidTagConversationId
        .and_then(PropertyValueExt::as_binary);
    let conversation_index = props
        .get(0x0071) // PidTagConversationIndex
        .and_then(PropertyValueExt::as_binary);

    // Extract date
    let date = props
        .get(0x0E06) // PidTagMessageDeliveryTime
        .and_then(|prop| match prop {
            PropertyValue::Time(t) => filetime_to_rfc3339(*t),
            _ => None,
        })
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract body (HTML > RTF > Plain text)
    let body_html = props
        .get(0x1013)
        .and_then(PropertyValueExt::as_string_or_binary_utf8); // PidTagHtmlBody
    let body_rtf = props.get(0x1009).and_then(PropertyValueExt::as_binary); // PidTagRtfCompressed
    let body_plain = props.get(0x1000).and_then(PropertyValueExt::as_string); // PidTagBody

    // Extract transport headers
    let headers = props.get(0x007D).and_then(PropertyValueExt::as_string); // PidTagTransportMessageHeaders

    // Extract message size
    let size_bytes = props
        .get(0x0E08) // PidTagMessageSize
        .and_then(PropertyValueExt::as_i32)
        .map(|i| i as u64);

    // Extract message flags (0x0E07 - PidTagMessageFlags)
    let flags = if flags_val != 0 {
        let mut flag_list = Vec::new();
        // Common message flags from MS-OXCMSG
        if flags_val & 0x00000001 != 0 {
            flag_list.push("Read".to_string());
        }
        if flags_val & 0x00000002 != 0 {
            flag_list.push("Unmodified".to_string());
        }
        if flags_val & 0x00000004 != 0 {
            flag_list.push("Submitted".to_string());
        }
        if flags_val & 0x00000008 != 0 {
            flag_list.push("Unsent".to_string());
        }
        if flags_val & 0x00000010 != 0 {
            flag_list.push("Has Attachments".to_string());
        }
        if flags_val & 0x00000020 != 0 {
            flag_list.push("From Me".to_string());
        }
        flag_list
    } else {
        Vec::new()
    };

    // Extract attachments from attachment table sub-nodes
    let attachments = if let Some(attachment_table) = message.attachment_table() {
        extract_attachments(message.as_ref(), attachment_table.as_ref())
    } else {
        Vec::new()
    };

    // Extract recipients from recipient table
    let (to, cc, bcc) = if let Some(recipient_table) = message.recipient_table() {
        extract_recipients(recipient_table.as_ref())
    } else {
        (vec![], vec![], vec![])
    };

    MessageData {
        subject,
        from,
        to,
        cc,
        bcc,
        date,
        message_id,
        body_html,
        body_rtf,
        body_plain,
        attachments,
        headers,
        size_bytes,
        flags,
        is_draft,
        folder_path: folder_path.to_string(),
        conversation_id,
        conversation_index,
    }
}

/// Check if an address string looks like a valid SMTP email address.
/// Rejects Exchange X500/DN-style addresses like `/O=EXCHANGELABS/OU=.../CN=...`.
fn is_smtp_address(addr: &str) -> bool {
    let trimmed = addr.trim();
    !trimmed.is_empty() && trimmed.contains('@') && !trimmed.starts_with('/')
}

/// Format an email field as "Display Name <email@address>" when both parts
/// are available. Falls back to whichever part is present, or "unknown".
fn format_email_field(display_name: Option<&str>, email_address: Option<&str>) -> String {
    match (
        display_name.filter(|s| !s.is_empty()),
        email_address.filter(|s| is_smtp_address(s)),
    ) {
        (Some(name), Some(addr)) => {
            // Don't duplicate if name is already the email address
            if name.eq_ignore_ascii_case(addr) {
                addr.to_string()
            } else {
                format!("{name} <{addr}>")
            }
        }
        (Some(name), None) => name.to_string(),
        (None, Some(addr)) => addr.to_string(),
        (None, None) => "unknown".to_string(),
    }
}

/// Extract recipients (To, CC, BCC) from a message's recipient table.
/// Returns three vectors: (to, cc, bcc), each with formatted address strings.
fn extract_recipients(
    recipient_table: &dyn outlook_pst::ltp::table_context::TableContext,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut to = Vec::new();
    let mut cc = Vec::new();
    let mut bcc = Vec::new();

    for row in recipient_table.rows_matrix() {
        let context = recipient_table.context();
        let columns_result = row.columns(context);

        if let Ok(columns) = columns_result {
            let mut display_name: Option<String> = None;
            let mut smtp_address: Option<String> = None;
            let mut email_address: Option<String> = None;
            let mut recipient_type: i32 = 1; // Default to TO

            for (col_idx, column_value) in columns.iter().enumerate() {
                if let Some(col_value) = column_value {
                    let col = &context.columns()[col_idx];
                    let prop_id = col.prop_id();

                    if let Ok(value) = recipient_table.read_column(col_value, col.prop_type()) {
                        match prop_id {
                            0x3001 => display_name = value.as_string(), // PidTagDisplayName
                            0x39FE => smtp_address = value.as_string(), // PidTagSmtpAddress
                            0x3003 => email_address = value.as_string(), // PidTagEmailAddress
                            0x0C15 => {
                                if let Some(v) = value.as_i32() {
                                    recipient_type = v;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Prefer SMTP address, then generic email (only if SMTP-like)
            let addr = smtp_address
                .filter(|s| is_smtp_address(s))
                .or_else(|| email_address.filter(|s| is_smtp_address(s)));
            let formatted = format_email_field(display_name.as_deref(), addr.as_deref());

            match recipient_type {
                2 => cc.push(formatted),
                3 => bcc.push(formatted),
                _ => to.push(formatted), // 1 (TO) and unknown types default to TO
            }
        }
    }

    (to, cc, bcc)
}

fn extract_attachments(
    message: &dyn outlook_pst::messaging::message::Message,
    attachment_table: &dyn outlook_pst::ltp::table_context::TableContext,
) -> Vec<Attachment> {
    use outlook_pst::messaging::attachment::AttachmentData;

    let mut attachments = Vec::new();
    let context = attachment_table.context();

    // Column 0x67F2 contains the attachment sub-node NID (LTP row ID).
    let nid_col_idx = context
        .columns()
        .iter()
        .position(|col| col.prop_id() == 0x67F2);
    let Some(nid_col_idx) = nid_col_idx else {
        return attachments;
    };

    for row in attachment_table.rows_matrix() {
        let Ok(columns) = row.columns(context) else {
            continue;
        };

        // Read the attachment sub-node NID from column 0x67F2
        #[allow(clippy::cast_sign_loss)]
        let sub_node_id = columns[nid_col_idx]
            .as_ref()
            .and_then(|col_val| {
                attachment_table
                    .read_column(col_val, context.columns()[nid_col_idx].prop_type())
                    .ok()
            })
            .and_then(|val| match val {
                PropertyValue::Integer32(nid) => Some(NodeId::from(nid as u32)),
                _ => None,
            });

        let Some(sub_node_id) = sub_node_id else {
            continue;
        };

        // Open the attachment sub-node to read full properties
        let Ok((props, data)) = message.open_attachment_data(sub_node_id) else {
            continue;
        };

        // Get filename: prefer long filename (0x3707), fall back to short (0x3704)
        let filename = props
            .get(0x3707)
            .and_then(PropertyValueExt::as_string)
            .or_else(|| props.get(0x3704).and_then(PropertyValueExt::as_string));

        // Get MIME type
        let content_type = props.get(0x370E).and_then(PropertyValueExt::as_string);

        // Get content ID (PidTagAttachContentId, 0x3712) for cid: matching
        let content_id = props.get(0x3712).and_then(PropertyValueExt::as_string);

        // Get content location (PidTagAttachContentLocation, 0x3713)
        let content_location = props.get(0x3713).and_then(PropertyValueExt::as_string);

        // Get attachment data
        let attachment_data = match data {
            Some(AttachmentData::Binary(bin)) => Some(bin.buffer().to_vec()),
            _ => None,
        };

        if let (Some(fname), Some(data_bytes)) = (filename, attachment_data) {
            attachments.push(Attachment {
                filename: fname,
                data: data_bytes,
                content_type,
                content_id,
                content_location,
            });
        }
    }

    attachments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_staged_record(
        sequence_number: u32,
        is_duplicate: bool,
        conversation_id: Option<Vec<u8>>,
    ) -> StagedExportRecord {
        let mut message_data = MessageData::example();
        message_data.conversation_id = conversation_id;

        StagedExportRecord {
            sequence_number,
            message_data,
            pst_store_name: "example.pst".to_string(),
            is_duplicate,
            matched_keywords: Vec::new(),
            matched_emails: Vec::new(),
        }
    }

    #[test]
    fn test_sequence_formatting() {
        assert_eq!(ExportCoordinator::format_sequence(1), "00001");
        assert_eq!(ExportCoordinator::format_sequence(42), "00042");
        assert_eq!(ExportCoordinator::format_sequence(12345), "12345");
    }

    #[test]
    fn test_next_sequence_number() {
        let args = ExportArgs {
            input: PathBuf::from("test.pst"),
            output: PathBuf::from("output"),
            metadata: false,
            attachments: false,
            headers: false,
            csv: false,
            drafts: false,
            conversations: false,
            keywords: None,
            emails: None,
        };

        let mut coord = ExportCoordinator::new(args);
        assert_eq!(coord.next_sequence_number(), 1);
        assert_eq!(coord.next_sequence_number(), 2);
        assert_eq!(coord.next_sequence_number(), 3);
    }

    #[test]
    fn test_decode_html_binary_utf8() {
        let html = b"<p>Hello</p>";
        assert_eq!(decode_html_binary(html), Some("<p>Hello</p>".to_string()));
    }

    #[test]
    fn test_decode_html_binary_utf16le_bom() {
        let html_utf16le = vec![
            0xFF, 0xFE, b'<', 0x00, b'p', 0x00, b'>', 0x00, b'H', 0x00, b'i', 0x00, b'<', 0x00,
            b'/', 0x00, b'p', 0x00, b'>', 0x00,
        ];
        assert_eq!(
            decode_html_binary(&html_utf16le),
            Some("<p>Hi</p>".to_string())
        );
    }

    #[test]
    fn test_decode_html_binary_windows_1252_fallback() {
        let html_cp1252 = [
            b'<', b'p', b'>', b'R', 0xE9, b's', b'u', b'm', 0xE9, b'<', b'/', b'p', b'>',
        ];
        assert_eq!(
            decode_html_binary(&html_cp1252),
            Some("<p>Résumé</p>".to_string())
        );
    }

    #[test]
    fn test_build_folder_path_root() {
        assert_eq!(build_folder_path(None, "Inbox"), "Inbox");
    }

    #[test]
    fn test_build_folder_path_nested() {
        assert_eq!(
            build_folder_path(Some("Top of Outlook Data File/Inbox"), "Receipts"),
            "Top of Outlook Data File/Inbox/Receipts"
        );
    }

    #[test]
    fn test_filetime_to_rfc3339_unix_epoch() {
        // 1970-01-01T00:00:00Z in Windows FILETIME ticks (100ns intervals since 1601-01-01).
        let filetime = 116_444_736_000_000_000_i64;
        assert_eq!(
            filetime_to_rfc3339(filetime),
            Some("1970-01-01T00:00:00+00:00".to_string())
        );
    }

    #[test]
    fn test_filetime_to_rfc3339_with_subsecond() {
        // 1970-01-01T00:00:00.0000001Z (one 100ns tick past unix epoch).
        let filetime = 116_444_736_000_000_001_i64;
        assert_eq!(
            filetime_to_rfc3339(filetime),
            Some("1970-01-01T00:00:00+00:00".to_string())
        );
    }

    #[test]
    fn test_conversation_assignments_are_scoped_per_output_root() {
        let records = vec![
            make_staged_record(1, false, Some(b"thread-a-id-----".to_vec())),
            make_staged_record(2, true, Some(b"thread-a-id-----".to_vec())),
            make_staged_record(3, false, Some(b"thread-b-id-----".to_vec())),
            make_staged_record(4, false, Some(b"thread-b-id-----".to_vec())),
            make_staged_record(5, true, Some(b"thread-c-id-----".to_vec())),
            make_staged_record(6, true, Some(b"thread-c-id-----".to_vec())),
        ];

        let assignments = build_conversation_assignments(&records);

        // Cross-root singleton members must not get a conversation folder.
        assert_eq!(assignments.get(&1), None);
        assert_eq!(assignments.get(&2), None);

        // Per-root multi-message groups are eligible.
        assert!(assignments.contains_key(&3));
        assert!(assignments.contains_key(&4));
        assert!(assignments.contains_key(&5));
        assert!(assignments.contains_key(&6));
    }
}
