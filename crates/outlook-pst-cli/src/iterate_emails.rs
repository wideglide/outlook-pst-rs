use outlook_pst::{
    ltp::prop_context::PropertyValue,
    messaging::{
    attachment::{UnicodeAttachment, UnicodeAttachmentData},
        folder::UnicodeFolder,
        message::UnicodeMessage,
        store::UnicodeStore,
    },
    ndb::node_id::NodeId,
    *,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::args;
use crate::encoding;
use chrono::{TimeZone, Utc};

const MAPI_TO: i32 = 1;
const MAPI_CC: i32 = 2;
const MAPI_BCC: i32 = 3;

// Distinguish behavior between console listing and HTML dumping
enum Action<'a> {
    List(&'a args::ListArgs),
    Dump(&'a args::DumpArgs),
}

#[derive(Default, Debug)]
struct TransportHeaders {
    from: Option<String>,
    to: Option<String>,
    cc: Option<String>,
    bcc: Option<String>,
    subject: Option<String>,
    date: Option<String>,
    message_id: Option<String>,
    in_reply_to: Option<String>,
    references: Option<String>,
    x_mailer: Option<String>,
    x_originating_ip: Option<String>,
    received_chain: Vec<String>,
    return_path: Option<String>,
    other_headers: HashMap<String, String>,
}

#[derive(Default)]
struct ProcessingStats {
    total_messages: usize,
    total_folders: usize,
    messages_with_attachments: usize,
    total_size_bytes: u64,
    processing_errors: usize,
}

impl ProcessingStats {
    fn add_message(&mut self, has_attachments: bool, size_bytes: u32) {
        self.total_messages += 1;
        if has_attachments {
            self.messages_with_attachments += 1;
        }
        self.total_size_bytes += size_bytes as u64;
    }
    fn add_folder(&mut self) { self.total_folders += 1; }
    fn add_error(&mut self) { self.processing_errors += 1; }
    fn print_summary(&self) {
        println!("\n{}", "=".repeat(80));
        println!("PROCESSING SUMMARY");
        println!("{}", "=".repeat(80));
        println!("Total folders processed: {}", self.total_folders);
        println!("Total messages processed: {}", self.total_messages);
        println!("Messages with attachments: {}", self.messages_with_attachments);
        if self.processing_errors > 0 { println!("Messages skipped due to errors: {}", self.processing_errors); }
        println!(
            "Total size: {} bytes ({:.2} MB)",
            self.total_size_bytes,
            self.total_size_bytes as f64 / (1024.0 * 1024.0)
        );
        if self.total_messages > 0 {
            let avg_size = self.total_size_bytes as f64 / self.total_messages as f64;
            println!("Average message size: {:.2} bytes", avg_size);
            let attachment_percentage = (self.messages_with_attachments as f64 / self.total_messages as f64) * 100.0;
            println!("Messages with attachments: {:.1}%", attachment_percentage);
        }
        println!("{}", "=".repeat(80));
    }
}

fn filetime_to_datetime(filetime: i64) -> String {
    // FILETIME: 100ns intervals since 1601-01-01 00:00:00 UTC
    // Convert to Unix epoch seconds + nanoseconds
    const WINDOWS_TO_UNIX_EPOCH_SECS: i64 = 11_644_473_600; // seconds between 1601 and 1970
    let secs = (filetime / 10_000_000).saturating_sub(WINDOWS_TO_UNIX_EPOCH_SECS);
    let nanos = (filetime % 10_000_000).rem_euclid(10_000_000) as u32 * 100; // 100ns -> ns
    Utc
        .timestamp_opt(secs, nanos)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Invalid Date".to_string())
}

// (date helpers removed; no longer used for filename generation)


fn parse_transport_headers(headers_text: &str) -> TransportHeaders {
    let mut headers = TransportHeaders::default();
    let mut current_header_name = String::new();
    let mut current_header_value = String::new();
    let mut in_header = false;
    for line in headers_text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if in_header { current_header_value.push(' '); current_header_value.push_str(line.trim()); }
        } else if let Some(colon_pos) = line.find(':') {
            if in_header && !current_header_name.is_empty() {
                process_header(&mut headers, &current_header_name, &current_header_value);
            }
            current_header_name = line[..colon_pos].trim().to_lowercase();
            current_header_value = line[colon_pos + 1..].trim().to_string();
            in_header = true;
        }
    }
    if in_header && !current_header_name.is_empty() {
        process_header(&mut headers, &current_header_name, &current_header_value);
    }
    headers
}

fn process_header(headers: &mut TransportHeaders, name: &str, value: &str) {
    match name {
        "from" => headers.from = Some(value.to_string()),
        "to" => headers.to = Some(value.to_string()),
        "cc" => headers.cc = Some(value.to_string()),
        "bcc" => headers.bcc = Some(value.to_string()),
        "subject" => headers.subject = Some(value.to_string()),
        "date" => headers.date = Some(value.to_string()),
        "message-id" => headers.message_id = Some(value.to_string()),
        "in-reply-to" => headers.in_reply_to = Some(value.to_string()),
        "references" => headers.references = Some(value.to_string()),
        "x-mailer" | "user-agent" => headers.x_mailer = Some(value.to_string()),
        "x-originating-ip" => headers.x_originating_ip = Some(value.to_string()),
        "return-path" => headers.return_path = Some(value.to_string()),
        "received" => headers.received_chain.push(value.to_string()),
        _ => {
            if name.starts_with("x-") || name.contains("spam") || name.contains("virus")
                || name == "authentication-results" || name == "dkim-signature"
                || name == "arc-authentication-results" {
                headers.other_headers.insert(name.to_string(), value.to_string());
            }
        }
    }
}

fn format_mailbox(display_name: Option<String>, email: Option<String>) -> String {
    match (display_name, email) {
        (Some(name), Some(email)) => {
            let clean_name = clean_exchange_dn(&name);
            if clean_name == email { email } else { format!("{} <{}>", clean_name, email) }
        }
        (Some(name), None) => { if name.contains('@') { name } else { clean_exchange_dn(&name) } }
        (None, Some(email)) => { if email.contains('@') { email } else { clean_exchange_dn(&email) } }
        (None, None) => "Unknown".to_string(),
    }
}

fn clean_exchange_dn(name: &str) -> String {
    if name.starts_with("/O=") {
        name.split("/CN=")
            .last()
            .map(|part| {
                if part.contains("-") && part.len() > 40 {
                    let parts: Vec<&str> = part.split("-").collect();
                    if parts.len() > 1 && parts[0].len() > 3 { parts[0].to_string() } else { part.to_string() }
                } else { part.to_string() }
            })
            .unwrap_or_else(|| "Exchange User".to_string())
    } else if name == "Unknown Sender" || name.is_empty() {
        String::new()
    } else {
        name.to_string()
    }
}

fn get_sender_info(message: &UnicodeMessage) -> (String, String) {
    let properties = message.properties();
    let get_str = |pid: u16| -> Option<String> {
        properties.get(pid).and_then(|v| match v {
            PropertyValue::String8(s) => Some(s.to_string()),
            PropertyValue::Unicode(s) => Some(s.to_string()),
            _ => None,
        })
    };
    let is_email_like = |s: &str| -> bool { let t = s.trim(); t.contains('@') && !t.starts_with("/O=") };
    let sender_name = get_str(0x0042).or_else(|| get_str(0x0C1A)).unwrap_or_default();
    let sender_addrtype = get_str(0x0C1E).map(|s| s.to_uppercase());
    let repr_addrtype = get_str(0x0064).map(|s| s.to_uppercase());
    let sender_smtp = get_str(0x5D01);
    let repr_smtp = get_str(0x5D02);
    let sender_addr = get_str(0x0C1F);
    let repr_addr = get_str(0x0065);
    let orig_sender_addr = get_str(0x0067);
    let creator_smtp = get_str(0x5D0A);
    let best_email = sender_smtp.as_ref().filter(|s| is_email_like(s)).cloned()
        .or_else(|| repr_smtp.as_ref().filter(|s| is_email_like(s)).cloned())
        .or_else(|| match (sender_addrtype.as_deref(), sender_addr.as_ref()) { (Some("SMTP"), Some(val)) if is_email_like(val) => Some(val.clone()), _ => None })
        .or_else(|| match (repr_addrtype.as_deref(), repr_addr.as_ref()) { (Some("SMTP"), Some(val)) if is_email_like(val) => Some(val.clone()), _ => None })
        .or_else(|| sender_addr.as_ref().filter(|s| is_email_like(s)).cloned())
        .or_else(|| repr_addr.as_ref().filter(|s| is_email_like(s)).cloned())
        .or_else(|| orig_sender_addr.as_ref().filter(|s| is_email_like(s)).cloned())
        .or_else(|| creator_smtp.as_ref().filter(|s| is_email_like(s)).cloned());
    if let Some(email) = best_email { (sender_name, email) } else {
        let cleaned_display = if sender_name.trim().is_empty() {
            sender_smtp.or(repr_smtp).or(sender_addr).or(repr_addr).or(orig_sender_addr)
                .map(|raw| clean_exchange_dn(&raw)).unwrap_or_default()
        } else { sender_name };
        (cleaned_display, String::new())
    }
}

fn get_recipients(message: &UnicodeMessage) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut to_recipients = Vec::new();
    let mut cc_recipients = Vec::new();
    let mut bcc_recipients = Vec::new();
    let recipient_table = message.recipient_table();
    let context = recipient_table.context();
    for row in recipient_table.rows_matrix() {
        let columns: Vec<_> = match row.columns(context) { Ok(cols) => context.columns().iter().zip(cols).collect(), Err(_) => { continue; } };
        let recipient_type = columns.iter().find_map(|(col, value)| {
            if col.prop_id() == 0x0C15 { Some((value.as_ref(), col.prop_type())) } else { None }
        }).and_then(|(value, prop_type)| {
            message.store().read_table_column(recipient_table, value?, prop_type).ok()
        }).and_then(|value| match value { PropertyValue::Integer32(val) => Some(val), _ => None });
        let display_name = columns.iter().find_map(|(col, value)| {
            if col.prop_id() == 0x3001 { Some((value.as_ref(), col.prop_type())) } else { None }
        }).and_then(|(value, prop_type)| {
            message.store().read_table_column(recipient_table, value?, prop_type).ok()
        }).and_then(|value| match value { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(s) => Some(s.to_string()), _ => None });
        let smtp_address = columns.iter().find_map(|(col, value)| {
            if col.prop_id() == 0x39FE { Some((value.as_ref(), col.prop_type())) } else { None }
        }).and_then(|(value, prop_type)| {
            message.store().read_table_column(recipient_table, value?, prop_type).ok()
        }).and_then(|value| match value { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(s) => Some(s.to_string()), _ => None });
        let pr_email_address = columns.iter().find_map(|(col, value)| {
            if col.prop_id() == 0x3003 { Some((value.as_ref(), col.prop_type())) } else { None }
        }).and_then(|(value, prop_type)| {
            message.store().read_table_column(recipient_table, value?, prop_type).ok()
        }).and_then(|value| match value { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(s) => Some(s.to_string()), _ => None });
        let email_address = match smtp_address {
            Some(ref smtp) if smtp.contains('@') && !smtp.starts_with("/O=") => Some(smtp.clone()),
            Some(smtp) if pr_email_address.is_none() => Some(smtp),
            _ => pr_email_address,
        };
        if let Some(recipient_type) = recipient_type {
            let formatted_recipient = format_mailbox(display_name, email_address);
            match recipient_type { MAPI_TO => to_recipients.push(formatted_recipient), MAPI_CC => cc_recipients.push(formatted_recipient), MAPI_BCC => bcc_recipients.push(formatted_recipient), _ => {} }
        }
    }
    (to_recipients, cc_recipients, bcc_recipients)
}

fn get_transport_headers(message: &UnicodeMessage) -> Option<TransportHeaders> {
    let properties = message.properties();
    properties.get(0x007D).and_then(|value| match value {
        PropertyValue::String8(s) => Some(s.to_string()),
        PropertyValue::Unicode(s) => Some(s.to_string()),
        _ => None,
    }).map(|headers_text| parse_transport_headers(&headers_text))
}

fn parse_recipients_from_header(header_value: &str) -> Vec<String> {
    if header_value.trim().is_empty() { return Vec::new(); }
    let mut recipients = Vec::new();
    let mut current_recipient = String::new();
    let mut in_quoted_name = false;
    let mut bracket_depth = 0;
    for ch in header_value.chars() {
        match ch {
            '"' => { in_quoted_name = !in_quoted_name; current_recipient.push(ch); }
            '<' if !in_quoted_name => { bracket_depth += 1; current_recipient.push(ch); }
            '>' if !in_quoted_name => { bracket_depth -= 1; current_recipient.push(ch); }
            ',' if !in_quoted_name && bracket_depth == 0 => {
                let recipient = current_recipient.trim();
                if !recipient.is_empty() { recipients.push(recipient.to_string()); }
                current_recipient.clear();
            }
            _ => current_recipient.push(ch),
        }
    }
    let recipient = current_recipient.trim();
    if !recipient.is_empty() { recipients.push(recipient.to_string()); }
    recipients
}

struct AttachmentInfo { name: String, size: i32, attach_flags: i32, content_id: String }

fn list_attachments(message: &Rc<UnicodeMessage>) -> Option<Vec<AttachmentInfo>> {
    let table = message.attachment_table()?;
    let mut results = Vec::new();
    for row in table.rows_matrix() {
        let sub_node = NodeId::from(u32::from(row.id()));
        if let Ok(att) = UnicodeAttachment::read(message.clone(), sub_node, None) {
            let props = att.properties();
            let read_str = |id: u16| -> Option<String> {
                props.get(id).and_then(|v| match v {
                    PropertyValue::String8(s) => Some(s.to_string()),
                    PropertyValue::Unicode(s) => Some(s.to_string()),
                    _ => None,
                })
            };
            let name = read_str(0x3707).or_else(|| read_str(0x3704)).or_else(|| read_str(0x3001)).unwrap_or_else(|| "(unnamed attachment)".to_string());
            let size = props.attachment_size().unwrap_or(0);
            let _mime = read_str(0x370E);
            let attach_flags: i32 = match props.get(0x3714) {
                Some(PropertyValue::Integer32(flags)) => *flags,
                _ => 0,
            };
            let content_id = read_str(0x3712).unwrap_or_else(|| String::new());
            results.push(AttachmentInfo { name, size, attach_flags, content_id });
        }
    }
    Some(results)
}

fn detect_body_types(message: &UnicodeMessage) -> Vec<String> {
    let props = message.properties();
    let mut bodies = Vec::new();
    if let Some(val) = props.get(0x1000) { if matches!(val, PropertyValue::String8(_) | PropertyValue::Unicode(_)) { bodies.push("text".into()) } }
    if let Some(val) = props.get(0x1013) { if matches!(val, PropertyValue::Binary(_) | PropertyValue::String8(_) | PropertyValue::Unicode(_)) { bodies.push("html".into()) } }
    if let Some(PropertyValue::Binary(_)) = props.get(0x1009) { bodies.push("rtf".into()); }
    bodies
}

// Extract a best-effort plain text body for keyword scanning.
fn extract_plain_body(message: &UnicodeMessage) -> Option<String> {
    let props = message.properties();
    // Prefer text body if available (0x1000)
    if let Some(val) = props.get(0x1000) {
        match val {
            PropertyValue::String8(s) => return Some(s.to_string()),
            PropertyValue::Unicode(u) => return Some(u.to_string()),
            _ => {}
        }
    }
    // Fallback to HTML body (0x1013) and strip tags crudely
    if let Some(val) = props.get(0x1013) {
        let html_opt = match val {
            PropertyValue::Binary(b) => {
                let code_page = match props.get(0x3FDE) { Some(PropertyValue::Integer32(cpid)) => u16::try_from(*cpid).ok(), _ => None };
                if let Some(cp) = code_page { encoding::decode_html_body(b.buffer(), cp) } else { encoding::decode_html_body(b.buffer(), 20127) }
            }
            PropertyValue::String8(s) => Some(s.to_string()),
            PropertyValue::Unicode(u) => Some(u.to_string()),
            _ => None,
        };
        if let Some(html) = html_opt {
            return Some(strip_html_like(&html));
        }
    }
    // Fallback to RTF (0x1009) by using decoded RTF text if available
    if let Some(PropertyValue::Binary(b)) = props.get(0x1009) {
        if let Some(rtf) = encoding::decode_rtf_compressed(b.buffer()) {
            return Some(rtf);
        }
    }
    None
}

// Very simple HTML tag stripper for keyword scanning; not a full HTML parser.
fn strip_html_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    htmlescape::decode_html(&out).unwrap_or(out)
}

fn process_message(
    message: Rc<UnicodeMessage>,
    folder_path: &str,
    stats: &mut ProcessingStats,
    action: &Action,
    csv_rows: Option<&mut Vec<Vec<String>>>,
    seen_message_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    let properties = message.properties();
    let mut text_out: Vec<String> = Vec::new();
    let transport_headers = get_transport_headers(&message);
    let subject = transport_headers.as_ref().and_then(|h| h.subject.clone()).or_else(|| {
        properties.get(0x0037).and_then(encoding::decode_subject)
    }).unwrap_or_else(|| "(No Subject)".to_string());
    let received_time = transport_headers.as_ref().and_then(|h| h.date.clone()).or_else(|| {
        properties.get(0x0E06).and_then(|value| match value { PropertyValue::Time(time) => Some(*time), _ => None }).map(filetime_to_datetime)
    }).unwrap_or_else(|| "Unknown Date".to_string());
    let message_id = transport_headers.as_ref().and_then(|h| h.message_id.clone()).or_else(|| {
        properties.get(0x1035).and_then(|value| match value { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(s) => Some(s.to_string()), _ => None })
    });
    // Normalize Message-Id for duplicate detection: trim whitespace, strip surrounding <>, lowercase
    let normalize_msg_id = |s: &str| -> String {
        let s = s.trim();
        let s = s.strip_prefix('<').unwrap_or(s);
        let s = s.strip_suffix('>').unwrap_or(s);
        s.trim().to_ascii_lowercase()
    };
    let mut is_duplicate = false;
    if let Some(ref mid) = message_id {
        let norm = normalize_msg_id(mid);
        if !norm.is_empty() {
            if seen_message_ids.contains(&norm) {
                is_duplicate = true;
            } else {
                seen_message_ids.insert(norm);
            }
        }
    }
    let from_display = if let Some(headers) = &transport_headers {
        if let Some(from_header) = &headers.from { from_header.clone() } else {
            let (sender_name, sender_email) = get_sender_info(&message);
            format_mailbox(if sender_name.is_empty() { None } else { Some(sender_name) }, if sender_email.is_empty() { None } else { Some(sender_email) })
        }
    } else {
        let (sender_name, sender_email) = get_sender_info(&message);
        format_mailbox(if sender_name.is_empty() { None } else { Some(sender_name) }, if sender_email.is_empty() { None } else { Some(sender_email) })
    };
    // Common lines to render in list or HTML header
    text_out.push(format!("Subject: {}", subject));
    text_out.push(format!("From: {}", from_display));
    text_out.push(format!("Date: {}", received_time));

    let (to_recipients, cc_recipients, bcc_recipients) = if let Some(headers) = &transport_headers {
        let to_list = headers.to.as_ref().map(|to| parse_recipients_from_header(to)).unwrap_or_else(Vec::new);
        let cc_list = headers.cc.as_ref().map(|cc| parse_recipients_from_header(cc)).unwrap_or_else(Vec::new);
        let bcc_list = headers.bcc.as_ref().map(|bcc| parse_recipients_from_header(bcc)).unwrap_or_else(Vec::new);
        if to_list.is_empty() && cc_list.is_empty() && bcc_list.is_empty() { get_recipients(&message) } else { (to_list, cc_list, bcc_list) }
    } else { get_recipients(&message) };
    if !to_recipients.is_empty() { text_out.push(format!("To: {}", to_recipients.join("; "))); }
    if !cc_recipients.is_empty() { text_out.push(format!("Cc: {}", cc_recipients.join("; "))); }
    if !bcc_recipients.is_empty() { text_out.push(format!("Bcc: {}", bcc_recipients.join("; "))); }
        // Responsive Emails: if user provided any responsive email addresses, check participants
        let mut responsive_matches: Vec<String> = Vec::new();
        let provided: Vec<String> = match action {
            Action::List(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
            Action::Dump(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
        };
        if !provided.is_empty() {
            // Build set of participant emails in lowercase
            let mut participant_emails: Vec<String> = Vec::new();
            // Extract email inside angle brackets if present, otherwise try to parse a bare email-like token
            let mut push_email = |s: &str| {
                let s = s.trim();
                // try find <email>
                if let Some(start) = s.find('<') { if let Some(end) = s[start+1..].find('>') { let e = &s[start+1..start+1+end]; if e.contains('@') { participant_emails.push(e.to_ascii_lowercase()); return; } } }
                // else, find token that looks like email
                let cand = s.trim_matches(['"']).to_string();
                if cand.contains('@') { participant_emails.push(cand.to_ascii_lowercase()); }
            };
            // From
            push_email(&from_display);
            // To/CC/BCC lists
            for r in &to_recipients { push_email(r); }
            for r in &cc_recipients { push_email(r); }
            for r in &bcc_recipients { push_email(r); }
            // Transport header fallback for cases where display strings didn't include emails
            if let Some(h) = &transport_headers {
                if let Some(ref v) = h.from { push_email(v); }
                if let Some(ref v) = h.to { for r in parse_recipients_from_header(v) { push_email(&r); } }
                if let Some(ref v) = h.cc { for r in parse_recipients_from_header(v) { push_email(&r); } }
                if let Some(ref v) = h.bcc { for r in parse_recipients_from_header(v) { push_email(&r); } }
            }
            participant_emails.sort();
            participant_emails.dedup();
            for target in &provided {
                if participant_emails.iter().any(|p| p == target) {
                    responsive_matches.push(target.clone());
                }
            }
            if !responsive_matches.is_empty() {
                text_out.push(format!("Responsive Emails: {}", responsive_matches.join(", ")));
            }
        }
    // Keywords: if user provided any keywords, scan the body and list those found
        let provided_keywords: Vec<String> = match action {
            Action::List(a) => a.keywords.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            Action::Dump(a) => a.keywords.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        };
    let mut keyword_count_for_csv: usize = 0;
        if !provided_keywords.is_empty() {
            if let Some(body) = extract_plain_body(&message) {
                let body_lc = body.to_ascii_lowercase();
                let mut hits: Vec<String> = Vec::new();
                for kw in &provided_keywords {
                    let kw_trim = kw.trim();
                    if kw_trim.is_empty() { continue; }
                    if body_lc.contains(&kw_trim.to_ascii_lowercase()) {
                        hits.push(kw_trim.to_string());
                    }
                }
                hits.sort();
                hits.dedup();
                keyword_count_for_csv = hits.len();
                if !hits.is_empty() {
                    text_out.push(format!("Keywords: {}", hits.join(", ")));
                }
            }
        }
    if let Some(ref msg_id) = message_id { text_out.push(format!("MessageId: {}", msg_id)); }
    text_out.push(format!("Folder: {}", folder_path));

    let message_size = properties.get(0x0E08).and_then(|value| match value { PropertyValue::Integer32(size) => Some(*size as u32), _ => None }).unwrap_or(0);
    if message_size > 0 { text_out.push(format!("Size: {} bytes", message_size)); }

    let mut has_attachments = false;
    let mut flag_descriptions = Vec::new();
    if let Some(PropertyValue::Integer32(flags)) = properties.get(0x0E07) {
        has_attachments = flags & 0x00000010 != 0;
        if flags & 0x00000001 != 0 { flag_descriptions.push("Read"); }
        if flags & 0x00000002 != 0 { flag_descriptions.push("Unmodified"); }
        if flags & 0x00000004 != 0 { flag_descriptions.push("Submit"); }
        if flags & 0x00000008 != 0 { flag_descriptions.push("Unsent"); }
        if has_attachments { flag_descriptions.push("Has Attachments"); }
        if flags & 0x00000020 != 0 { flag_descriptions.push("From Me"); }
        if flags & 0x00000040 != 0 { flag_descriptions.push("Associated"); }
        if flags & 0x00000080 != 0 { flag_descriptions.push("Resend"); }
        if flags & 0x00000100 != 0 { flag_descriptions.push("RN Pending"); }
        if flags & 0x00000200 != 0 { flag_descriptions.push("NRN Pending"); }
    }
    if !flag_descriptions.is_empty() { text_out.push(format!("Flags: {}", flag_descriptions.join(", "))); }

    // Retrieve attachments once and reuse across list/dump/CSV paths
    let attachments = list_attachments(&message);

    // Optional details for list mode only
    let mut extra_list_lines: Vec<String> = Vec::new();
    if let Action::List(list_args) = action {
        if list_args.show_headers {
            if let Some(headers) = &transport_headers {
                if headers.message_id.is_some() || headers.x_mailer.is_some() || headers.x_originating_ip.is_some() || !headers.received_chain.is_empty() {
                    extra_list_lines.push("Transport Headers:".to_string());
                    if let Some(in_reply_to) = &headers.in_reply_to { extra_list_lines.push(format!("  In-Reply-To: {}", in_reply_to)); }
                    if let Some(references) = &headers.references { extra_list_lines.push(format!("  References: {}", references)); }
                    if let Some(mailer) = &headers.x_mailer { extra_list_lines.push(format!("  X-Mailer: {}", mailer)); }
                    if let Some(ip) = &headers.x_originating_ip { extra_list_lines.push(format!("  X-Originating-IP: {}", ip)); }
                    if let Some(return_path) = &headers.return_path { extra_list_lines.push(format!("  Return-Path: {}", return_path)); }
                    for (i, received) in headers.received_chain.iter().take(3).enumerate() { extra_list_lines.push(format!("  Received[{}]: {}", i + 1, received)); }
                    for (name, value) in headers.other_headers.iter().take(5) { extra_list_lines.push(format!("  {}: {}", name, value)); }
                }
            }
        }
        if list_args.show_attachments {
            if let Some(infos) = attachments.as_ref() {
                if !infos.is_empty() {
                    extra_list_lines.push(format!("Attachments ({}):", infos.len()));
                    for info in infos.iter() {
                        if info.attach_flags == 0 || info.content_id.is_empty() {
                            extra_list_lines.push(format!("  - {}, (size: {}, cid: {}, af: {})", info.name, info.size, info.content_id, info.attach_flags));
                        }
                    }
                }
            }
        }
        if list_args.show_body_types {
            let bodies = detect_body_types(&message);
            if !bodies.is_empty() { extra_list_lines.push(format!("Bodies: {}", bodies.join(", "))); } else { extra_list_lines.push("Bodies: none".to_string()); }
        }
    }
    if let Action::Dump(_) = action {
        if let Some(infos) = attachments.as_ref() {
            if !infos.is_empty() {
                let mut attachments = Vec::new();
                for info in infos.iter() {
                    if info.attach_flags == 0 || info.content_id.is_empty() {
                        attachments.push(format!("{}", info.name));
                    }
                }
                if !attachments.is_empty() {
                    text_out.push(format!("Attachments: {}", attachments.join(" ;;; ")));
                }
            }
        }
    }

    // Render or dump
    match action {
        Action::List(_) => {
            println!("{}", "=".repeat(80));
            for line in &text_out { println!("{}", line); }
            for line in &extra_list_lines { println!("{}", line); }
            println!();
        }
        Action::Dump(dump_args) => {
            // For dump, we will write HTML after we update stats to maintain zero-based index
            // Defer actual writing until after stats.add_message below
            // We'll compute and write immediately after bump
            // To keep code simple, we perform dump here but compute index using current (pre-increment) value - 0
            // We'll instead write after increment; see below
            let _ = &dump_args.out_dir; // marker to avoid unused warning in this branch
        }
    }

    // Update stats
    stats.add_message(has_attachments, message_size);

    // If CSV collection is enabled, append row for this email now that index is known
    if let Some(rows) = csv_rows {
        let idx = stats.total_messages - 1; // zero-based
    let num_attachments = attachments.as_ref().map(|v| v.len()).unwrap_or(0);
        let responsive_count = {
            // Recompute from the already-found responsive_matches where available
            // Note: responsive_matches is in scope; use its length
            // If not computed (no provided emails), length is 0
            // The variable exists above; shadow a local reference
            #[allow(unused_mut)]
            let mut count = 0usize;
            // We can’t borrow responsive_matches here directly if it went out of scope, so recompute quickly
            let provided: Vec<String> = match action {
                Action::List(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
                Action::Dump(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
            };
            if !provided.is_empty() {
                let mut participant_emails: Vec<String> = Vec::new();
                let mut push_email = |s: &str| {
                    let s = s.trim();
                    if let Some(start) = s.find('<') {
                        if let Some(end) = s[start + 1..].find('>') {
                            let e = &s[start + 1..start + 1 + end];
                            if e.contains('@') { participant_emails.push(e.to_ascii_lowercase()); return; }
                        }
                    }
                    let cand = s.trim_matches(['"']).to_string();
                    if cand.contains('@') { participant_emails.push(cand.to_ascii_lowercase()); }
                };
                push_email(&from_display);
                for r in &to_recipients { push_email(r); }
                for r in &cc_recipients { push_email(r); }
                for r in &bcc_recipients { push_email(r); }
                if let Some(h) = &transport_headers {
                    if let Some(ref v) = h.from { push_email(v); }
                    if let Some(ref v) = h.to { for r in parse_recipients_from_header(v) { push_email(&r); } }
                    if let Some(ref v) = h.cc { for r in parse_recipients_from_header(v) { push_email(&r); } }
                    if let Some(ref v) = h.bcc { for r in parse_recipients_from_header(v) { push_email(&r); } }
                }
                participant_emails.sort();
                participant_emails.dedup();
                for target in &provided { if participant_emails.iter().any(|p| p == target) { count += 1; } }
            }
            count
        };
        // Keyword count was computed above in keyword_count_for_csv (0 if none)
        // PST store name
        let sane_recipients_str = |recipients: &[String]| {
            if recipients.len() < 16 {
                recipients.join("; ").replace("\"", "")
            } else {
                format!("{} recipients", recipients.len())
            }
        };
        let store_name = message.store().properties().display_name().unwrap_or_else(|_| "PST Store".to_string());
    let mut row = vec![
            format!("{}", idx),
            subject.clone(),
            received_time.clone(),
            from_display.clone().replace("\"", ""),
            sane_recipients_str(&to_recipients),
            sane_recipients_str(&cc_recipients),
            format!("{}", message_size),
            format!("{}", responsive_count),
            format!("{}", keyword_count_for_csv),
            format!("{}", num_attachments),
            message_id.clone().unwrap_or_default(),
            store_name,
    ];
    // Append duplicate flag column
    row.push(if is_duplicate { "true".to_string() } else { "false".to_string() });
    rows.push(row);
    }

    // If dump, write HTML file now using the zero-based index convention from previous implementation
    if let Action::Dump(dump_args) = action {
        let out_dir = &dump_args.out_dir;
        fs::create_dir_all(out_dir)?;
        // Determine body HTML
        let mut body_html: Option<String> = None;
        if let Some(val) = properties.get(0x1013) {
            match val {
                PropertyValue::Binary(b) => {
                    let code_page = match properties.get(0x3FDE) { Some(PropertyValue::Integer32(cpid)) => u16::try_from(*cpid).ok(), _ => None };
                    if let Some(cp) = code_page { body_html = encoding::decode_html_body(b.buffer(), cp); } else { body_html = encoding::decode_html_body(b.buffer(), 20127); }
                }
                PropertyValue::String8(s) => body_html = Some(s.to_string()),
                PropertyValue::Unicode(u) => body_html = Some(u.to_string()),
                _ => {}
            }
        }
        if body_html.is_none() {
            if let Some(val) = properties.get(0x1000) {
                match val {
                    PropertyValue::String8(s) => { let t = htmlescape::encode_minimal(&s.to_string()); body_html = Some(format!("<pre>{}</pre>", t)); }
                    PropertyValue::Unicode(u) => { let t = htmlescape::encode_minimal(&u.to_string()); body_html = Some(format!("<pre>{}</pre>", t)); }
                    _ => {}
                }
            }
        }
        if body_html.is_none() {
            if let Some(PropertyValue::Binary(b)) = properties.get(0x1009) {
                if let Some(rtf) = encoding::decode_rtf_compressed(b.buffer()) { let t = htmlescape::encode_minimal(&rtf); body_html = Some(format!("<pre>{}</pre>", t)); }
            }
        }
        let body_html = body_html.unwrap_or_else(|| "<em>(no body)</em>".to_string());
    let idx = stats.total_messages - 1; // zero-based index
    let index_str = format!("{:05}", idx);
    // Write to default or duplicate directory per requirement
    // default: <out-dir>/<index>/message.html
    // duplicates: <out-dir>/duplicates/<index>/message.html
    let mut path = PathBuf::from(out_dir);
    if is_duplicate { path.push("duplicates"); }
    path.push(&index_str);
    fs::create_dir_all(&path)?;
    path.push("message.html");
        let mut html = String::new();
        html.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>");
        html.push_str(&htmlescape::encode_minimal(&subject));
        html.push_str("</title><style>body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Arial,sans-serif}table.meta{border-collapse:collapse;width:100%;}table.meta td{border:1px solid #ddd;padding:6px;vertical-align:top;}hr{margin:18px 0;border:none;border-top:1px solid #ccc}</style></head><body>");
        html.push_str("<table class=\"meta\" style=\"background-color: #f3f3f3; font-size: 12px;\">");
        for line in &text_out {
            if let Some(pos) = line.find(':') {
                let (k, v) = line.split_at(pos);
                let v = &v[1..];
                html.push_str("<tr><td><strong>");
                html.push_str(&htmlescape::encode_minimal(k));
                html.push_str(":</strong></td><td>");
                html.push_str(&htmlescape::encode_minimal(v.trim()));
                html.push_str("</td></tr>");
            } else {
                html.push_str("<tr><td colspan=2>");
                html.push_str(&htmlescape::encode_minimal(line));
                html.push_str("</td></tr>");
            }
        }
        html.push_str("</table><hr>");
        html.push_str(&body_html);
        html.push_str("</body></html>");
        let mut file = fs::File::create(&path)?;
        file.write_all(html.replace(" ;;; ", " <br> ").as_bytes())?;

        // Optionally dump attachments alongside message.html
        if dump_args.attachments {
            if let Some(parent) = path.parent() {
                if let Err(e) = save_attachments_to_dir(&message, parent) {
                    eprintln!(
                        "Warning: Failed to save one or more attachments for index {}: {}",
                        index_str, e
                    );
                }
            }
        }
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    // Remove any path separators and control chars, keep common filename-safe set
    let mut s: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    s = s.trim_matches([' ', '.', '_']).to_string();
    if s.is_empty() { s = "attachment".to_string(); }
    // Limit filename length
    if s.len() > 120 { s.truncate(120); }
    s
}

fn ensure_unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() { return path; }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("attachment");
    let ext = path.extension().and_then(|e| e.to_str());
    for i in 1..1000 {
        let mut candidate = path.clone();
        candidate.set_file_name(match ext {
            Some(ext) if !ext.is_empty() => format!("{} ({}).{}", stem, i, ext),
            _ => format!("{} ({})", stem, i),
        });
        if !candidate.exists() { return candidate; }
    }
    path
}

fn save_attachments_to_dir(message: &Rc<UnicodeMessage>, out_dir: &Path) -> anyhow::Result<()> {
    let table = match message.attachment_table() {
        Some(t) => t,
        None => return Ok(()),
    };
    for row in table.rows_matrix() {
        let sub_node = NodeId::from(u32::from(row.id()));
        let att = match UnicodeAttachment::read(message.clone(), sub_node, None) {
            Ok(a) => a,
            Err(_) => continue,
        };
        let props = att.properties();
        // Preferred name properties: PidTagAttachLongFilename (0x3707), PidTagAttachFilename (0x3704), DisplayName (0x3001)
        let read_str = |id: u16| -> Option<String> {
            props.get(id).and_then(|v| match v {
                PropertyValue::String8(s) => Some(s.to_string()),
                PropertyValue::Unicode(s) => Some(s.to_string()),
                _ => None,
            })
        };
        let mut name = read_str(0x3707)
            .or_else(|| read_str(0x3704))
            .or_else(|| read_str(0x3001))
            .unwrap_or_else(|| "attachment.bin".to_string());
        name = sanitize_filename(&name);
        if !name.contains('.') {
            // Consider MIME type to append an extension
            if let Some(ct) = read_str(0x370E) {
                let ext = mime_guess_ext_fallback(&ct);
                if let Some(ext) = ext { name.push('.'); name.push_str(ext); }
            }
        }
        // Save content-id attachments again
        let content_id_name = read_str(0x3712)
            .unwrap_or_default();
        let attach_flags: i32 = match props.get(0x3714) {
            Some(PropertyValue::Integer32(flags)) => *flags,
            _ => 0,
        };

        let save_attachment = |name: &str, bin: &[u8]| -> anyhow::Result<()> {
            let mut path = out_dir.to_path_buf();
            path.push(name);
            path = ensure_unique_path(path);
            fs::write(path, bin)?;
            Ok(())  
        };

        if attach_flags == 0 && !name.is_empty(){
            // Only save binary attachments (afByValue). Others are skipped for now.
            if let Some(UnicodeAttachmentData::Binary(bin)) = att.data() {
                save_attachment(&name, bin.buffer())?;
            }
        }  else if !content_id_name.is_empty() {
            // If content-id is present, save it with that name
            if let Some(UnicodeAttachmentData::Binary(bin)) = att.data() {
                save_attachment(&content_id_name, bin.buffer())?;
            }
        } 

    }
    Ok(())
}

fn mime_guess_ext_fallback(ct: &str) -> Option<&'static str> {
    let ct = ct.to_ascii_lowercase();
    match ct.as_str() {
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "application/pdf" => Some("pdf"),
        "text/plain" => Some("txt"),
        "text/html" => Some("html"),
        "application/zip" => Some("zip"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some("docx"),
        "application/msword" => Some("doc"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("xlsx"),
        "application/vnd.ms-excel" => Some("xls"),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => Some("pptx"),
        "application/vnd.ms-powerpoint" => Some("ppt"),
        _ => None,
    }
}

fn process_folder_recursive(
    store: Rc<UnicodeStore>,
    folder: &UnicodeFolder,
    folder_path: &str,
    stats: &mut ProcessingStats,
    action: &Action,
    mut csv_rows: Option<&mut Vec<Vec<String>>>,
    seen_message_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    stats.add_folder();
    if let Some(contents_table) = folder.contents_table() {
    // Deterministic ordering of messages by row id
    let mut rows: Vec<_> = contents_table.rows_matrix().collect();
    rows.sort_by_key(|r| u32::from(r.id()));
    for row in rows {
            let node_id = NodeId::from(u32::from(row.id()));
            match store.properties().make_entry_id(node_id) {
                Ok(entry_id) => {
                    match UnicodeMessage::read(store.clone(), &entry_id, None) {
                        Ok(message) => {
                            let res = if let Some(rows) = &mut csv_rows {
                                process_message(message, folder_path, stats, action, Some(*rows), seen_message_ids)
                            } else {
                                process_message(message, folder_path, stats, action, None, seen_message_ids)
                            };
                            if let Err(e) = res {
                                eprintln!("Warning: Error processing message in folder '{}': {}", folder_path, e);
                            }
                        }
                        Err(e) => { stats.add_error(); eprintln!("Warning: Skipped message in folder '{}' due to error: {}", folder_path, e); }
                    }
                }
                Err(e) => { stats.add_error(); eprintln!("Warning: Could not create entry ID for message in folder '{}': {}", folder_path, e); }
            }
        }
    }
    if let Some(hierarchy_table) = folder.hierarchy_table() {
        // Gather subfolders first to sort deterministically by display name then id
        let mut children: Vec<(String, u32)> = Vec::new();
        for row in hierarchy_table.rows_matrix() {
            let node_id_u32 = u32::from(row.id());
            let node_id = NodeId::from(node_id_u32);
            if let Ok(entry_id) = store.properties().make_entry_id(node_id) {
                if let Ok(subfolder) = UnicodeFolder::read(store.clone(), &entry_id) {
                    let subfolder_name = match subfolder.properties().display_name() { Ok(name) => name, Err(_) => "Unknown Folder".to_string() };
                    children.push((subfolder_name, node_id_u32));
                }
            }
        }
        children.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        for (subfolder_name, node_id_u32) in children {
            let node_id = NodeId::from(node_id_u32);
            match store.properties().make_entry_id(node_id) {
                Ok(entry_id) => {
                    match UnicodeFolder::read(store.clone(), &entry_id) {
                        Ok(subfolder) => {
                            let subfolder_path = if folder_path.is_empty() { subfolder_name } else { format!("{}/{}", folder_path, subfolder_name) };
                            let res = if let Some(rows) = &mut csv_rows {
                                process_folder_recursive(
                                    store.clone(),
                                    &subfolder,
                                    &subfolder_path,
                                    stats,
                                    action,
                                    Some(*rows),
                                    seen_message_ids,
                                )
                            } else {
                                process_folder_recursive(
                                    store.clone(),
                                    &subfolder,
                                    &subfolder_path,
                                    stats,
                                    action,
                                    None,
                                    seen_message_ids,
                                )
                            };
                            if let Err(e) = res { eprintln!("Warning: Error processing folder '{}': {}", subfolder_path, e); }
                        }
                        Err(e) => { stats.add_error(); eprintln!("Warning: Skipped subfolder due to error: {}", e); }
                    }
                }
                Err(e) => { stats.add_error(); eprintln!("Warning: Could not create entry ID for subfolder: {}", e); }
            }
        }
    }
    Ok(())
}

fn collect_pst_files(input: &str) -> anyhow::Result<Vec<PathBuf>> {
    let path = Path::new(input);
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if path.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| p.extension().map(|e| e.to_ascii_lowercase() == "pst").unwrap_or(false))
            .collect();
        files.sort(); // deterministic order
        return Ok(files);
    }
    Err(anyhow::anyhow!("Input path does not exist: {}", input))
}

fn process_single_pst(
    pst_path: &Path,
    stats: &mut ProcessingStats,
    action: &Action,
    csv_rows: Option<&mut Vec<Vec<String>>>,
    seen_message_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    println!("Opening PST file: {}", pst_path.display());
    println!();
    if let Ok(pst) = UnicodePstFile::open(pst_path.to_string_lossy().as_ref()) {
        let store = UnicodeStore::read(Rc::new(pst))?;
        let ipm_sub_tree = store.properties().ipm_sub_tree_entry_id()?;
        let root_folder = UnicodeFolder::read(store.clone(), &ipm_sub_tree)?;
        let store_name = store.properties().display_name().unwrap_or_else(|_| "PST Store".to_string());
        println!("Processing emails from Unicode PST store: {}", store_name);
        println!();
    process_folder_recursive(store, &root_folder, "", stats, action, csv_rows, seen_message_ids)?;
    } else {
        println!("Failed to open as Unicode PST, trying ANSI format...");
        return Err(anyhow::anyhow!("ANSI PST support not implemented in this example yet"));
    }
    Ok(())
}

fn write_csv(path: &Path, rows: &[Vec<String>]) -> anyhow::Result<()> {
    // Ensure parent exists if any
    if let Some(parent) = path.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
    let mut file = fs::File::create(path)?;
    // Header
    let header = [
        "index",
        "subject",
        "date",
        "from",
        "to",
        "cc",
        "size",
        "number-of-responsive-emails",
        "number-of-keywords",
        "number-of-attachments",
        "MessageId",
        "pst-store-name",
        "duplicate",
    ];
    for (i, col) in header.iter().enumerate() {
        if i > 0 { file.write_all(b",")?; }
        let mut s = col.to_string();
        // Minimal quoting
        if s.contains([',', '"', '\n', '\r']) { s = s.replace('"', "\"\""); s = format!("\"{}\"", s); }
        file.write_all(s.as_bytes())?;
    }
    file.write_all(b"\n")?;
    // Rows
    for row in rows {
        let mut first_col = true;
        for col in row {
            if !first_col { file.write_all(b",")?; } else { first_col = false; }
            let mut s = col.clone();
            if s.contains([',', '"', '\n', '\r']) { s = s.replace('"', "\"\""); s = format!("\"{}\"", s); }
            file.write_all(s.as_bytes())?;
        }
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn run_internal_multi(input: &str, action: Action) -> anyhow::Result<()> {
    let files = collect_pst_files(input)?;
    if files.is_empty() {
        println!("No .pst files found to process.");
        return Ok(());
    }
    let mut stats = ProcessingStats::default();
    // Optional CSV accumulator shared across all processed PST files in this run
    let mut csv_acc: Option<Vec<Vec<String>>> = match &action {
        Action::List(a) if a.csv => Some(Vec::new()),
        Action::Dump(a) if a.csv => Some(Vec::new()),
        _ => None,
    };
    // Global MessageId set for the entire run
    let mut seen_message_ids: HashSet<String> = HashSet::new();
    for f in files {
        if let Err(e) = process_single_pst(&f, &mut stats, &action, csv_acc.as_mut().map(|v| v), &mut seen_message_ids) {
            eprintln!("Warning: Skipping PST '{}': {}", f.display(), e);
        }
    }
    stats.print_summary();
    // If CSV was requested, write it now
    if let Some(rows) = csv_acc {
        match &action {
            Action::Dump(a) if a.csv => {
                let mut path = PathBuf::from(&a.out_dir);
                path.push("emails.csv");
                if let Err(e) = write_csv(&path, &rows) { eprintln!("Warning: Failed to write CSV '{}': {}", path.display(), e); }
                else { println!("CSV written to {}", path.display()); }
            }
            Action::List(a) if a.csv => {
                let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                path.push("emails.csv");
                if let Err(e) = write_csv(&path, &rows) { eprintln!("Warning: Failed to write CSV '{}': {}", path.display(), e); }
                else { println!("CSV written to {}", path.display()); }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn run_list(args: args::ListArgs) -> anyhow::Result<()> {
    run_internal_multi(&args.input, Action::List(&args))
}

pub fn run_dump(args: args::DumpArgs) -> anyhow::Result<()> {
    run_internal_multi(&args.input, Action::Dump(&args))
}
