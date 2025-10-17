use outlook_pst::{
    ltp::prop_context::PropertyValue,
    messaging::{folder::Folder, message::Message, store::Store},
    ndb::node_id::NodeId,
    open_store,
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


// Distinguish behavior between console listing and HTML exporting
enum Action<'a> {
    List(&'a args::ListArgs),
    Export(&'a args::ExportArgs),
}


#[derive(Default, Debug)]
pub struct TransportHeaders {
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<String>,
    pub x_mailer: Option<String>,
    pub x_originating_ip: Option<String>,
    pub received_chain: Vec<String>,
    pub return_path: Option<String>,
    pub other_headers: HashMap<String, String>,
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


pub fn parse_transport_headers(headers_text: &str) -> TransportHeaders {
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
        "to" => headers.to = Some(crate::encoding::decode_rfc2047(value)),
        "cc" => headers.cc = Some(value.to_string()),
        "bcc" => headers.bcc = Some(value.to_string()),
        "subject" => headers.subject = Some(crate::encoding::decode_rfc2047(value)),
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

fn get_sender_info(message: &dyn Message) -> (String, String) {
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

fn get_recipients(message: &dyn Message) -> (Vec<String>, Vec<String>, Vec<String>) {
    let rt = message.recipient_table();
    let info = rt.context();
    let mut to_recipients = Vec::new();
    let mut cc_recipients = Vec::new();
    let mut bcc_recipients = Vec::new();
    for row in rt.rows_matrix() {
        let columns = match row.columns(info) { Ok(c) => c, Err(_) => continue };
        let mut recipient_type: Option<i32> = None;
        let mut display_name: Option<String> = None;
        let mut smtp_address: Option<String> = None;
        let mut pr_email_address: Option<String> = None;
        for (col_def, value) in info.columns().iter().zip(columns) {
            if let Some(valref) = value.as_ref() {
                match col_def.prop_id() {
                    0x0C15 => { // PR_RECIPIENT_TYPE
                        if let Ok(PropertyValue::Integer32(v)) = rt.read_column(valref, col_def.prop_type()) { recipient_type = Some(v); }
                    }
                    0x3001 => { // PR_DISPLAY_NAME
                        if let Ok(PropertyValue::String8(s)) = rt.read_column(valref, col_def.prop_type()) { display_name = Some(s.to_string()); }
                        else if let Ok(PropertyValue::Unicode(s)) = rt.read_column(valref, col_def.prop_type()) { display_name = Some(s.to_string()); }
                    }
                    0x39FE => { // PR_SMTP_ADDRESS
                        if let Ok(PropertyValue::String8(s)) = rt.read_column(valref, col_def.prop_type()) { smtp_address = Some(s.to_string()); }
                        else if let Ok(PropertyValue::Unicode(s)) = rt.read_column(valref, col_def.prop_type()) { smtp_address = Some(s.to_string()); }
                    }
                    0x3003 => { // PR_EMAIL_ADDRESS
                        if let Ok(PropertyValue::String8(s)) = rt.read_column(valref, col_def.prop_type()) { pr_email_address = Some(s.to_string()); }
                        else if let Ok(PropertyValue::Unicode(s)) = rt.read_column(valref, col_def.prop_type()) { pr_email_address = Some(s.to_string()); }
                    }
                    _ => {}
                }
            }
        }
        let email_address = match smtp_address {
            Some(ref smtp) if smtp.contains('@') && !smtp.starts_with("/O=") => Some(smtp.clone()),
            Some(smtp) if pr_email_address.is_none() => Some(smtp),
            _ => pr_email_address,
        };
        if let Some(rt_val) = recipient_type {
            let formatted = format_mailbox(display_name, email_address);
            match rt_val { 1 => to_recipients.push(formatted), 2 => cc_recipients.push(formatted), 3 => bcc_recipients.push(formatted), _ => {} }
        }
    }
    (to_recipients, cc_recipients, bcc_recipients)
}

fn get_transport_headers(message: &dyn Message) -> Option<TransportHeaders> {
    let properties = message.properties();
    properties.get(0x007D).and_then(|value| match value {
        PropertyValue::String8(s) => Some(s.to_string()),
        PropertyValue::Unicode(s) => Some(s.to_string()),
        _ => None,
    }).map(|headers_text| parse_transport_headers(&headers_text))
}

pub fn parse_recipients_from_header(header_value: &str) -> Vec<String> {
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

struct AttachmentInfo { name: String, size: i32, attach_method: i32, content_id: String, is_inline: bool }

fn list_attachments(message: &Rc<dyn Message>) -> Option<Vec<AttachmentInfo>> {
    let at = message.attachment_table()?;
    let info = at.context();
    let mut results = Vec::new();
    // Column property ids of interest (common Outlook attachment columns)
    // 0x3704 PR_ATTACH_FILENAME (short) Unicode/String8
    // 0x3707 PR_ATTACH_LONG_FILENAME Unicode/String8
    // 0x370E PR_ATTACH_CONTENT_ID Unicode/String8
    // 0x3712 PR_ATTACH_FLAGS Integer32 (inline flag often 0x00000004) (MS-OXCMSG 2.2.2.10)
    // 0x3705 PR_ATTACH_METHOD Integer32
    // 0x0E20 PR_ATTACH_SIZE Integer32
    for row in at.rows_matrix() {
        let cols = match row.columns(info) { Ok(c) => c, Err(_) => continue };
        let mut file_name: Option<String> = None;
        let mut long_file_name: Option<String> = None;
        let mut content_id: Option<String> = None;
        let mut attach_flags: Option<i32> = None; // inline, rendering flags
        let mut attach_method: Option<i32> = None;
        let mut attach_size: Option<i32> = None;
        for (col_def, value) in info.columns().iter().zip(cols) {
            if let Some(valref) = value.as_ref() {
                let pid = col_def.prop_id();
                match pid {
                    0x3704 | 0x3707 | 0x370E => {
                        if let Ok(pv) = at.read_column(valref, col_def.prop_type()) {
                            let s_opt = match pv { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(u) => Some(u.to_string()), _ => None };
                            match pid { 0x3704 => file_name = s_opt, 0x3707 => long_file_name = s_opt, 0x370E => content_id = s_opt, _ => {} }
                        }
                    }
                    0x3712 | 0x3705 | 0x0E20 => {
                        if let Ok(PropertyValue::Integer32(v)) = at.read_column(valref, col_def.prop_type()) {
                            match pid { 0x3712 => attach_flags = Some(v), 0x3705 => attach_method = Some(v), 0x0E20 => attach_size = Some(v), _ => {} }
                        }
                    }
                    _ => {}
                }
            }
        }
        let name = long_file_name.or(file_name).unwrap_or_else(|| "(unnamed)".to_string());
        let size = attach_size.unwrap_or(0);
        let method = attach_method.unwrap_or(0);
        let cid = content_id.unwrap_or_default();
        let flags = attach_flags.unwrap_or(0);
        let is_inline = (flags & 0x00000004) != 0 || (!cid.is_empty() && (name.is_empty() || name.starts_with("image")));
        results.push(AttachmentInfo { name, size, attach_method: method, content_id: cid, is_inline });
    }
    Some(results)
}

fn detect_body_types(message: &dyn Message) -> Vec<String> {
    let props = message.properties();
    let mut bodies = Vec::new();
    if let Some(val) = props.get(0x1000) { if matches!(val, PropertyValue::String8(_) | PropertyValue::Unicode(_)) { bodies.push("text".into()) } }
    if let Some(val) = props.get(0x1013) { if matches!(val, PropertyValue::Binary(_) | PropertyValue::String8(_) | PropertyValue::Unicode(_)) { bodies.push("html".into()) } }
    if let Some(PropertyValue::Binary(_)) = props.get(0x1009) { bodies.push("rtf".into()); }
    bodies
}

// Extract a best-effort plain text body for keyword scanning.
pub fn extract_plain_body(message: &dyn Message) -> Option<String> {
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
    message: Rc<dyn Message>,
    folder_path: &str,
    stats: &mut ProcessingStats,
    action: &Action,
    csv_rows: Option<&mut Vec<Vec<String>>>,
    seen_message_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    let properties = message.properties();
    let mut text_out: Vec<String> = Vec::new();
    let transport_headers = get_transport_headers(message.as_ref());
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
            let (sender_name, sender_email) = get_sender_info(message.as_ref());
            format_mailbox(if sender_name.is_empty() { None } else { Some(sender_name) }, if sender_email.is_empty() { None } else { Some(sender_email) })
        }
    } else {
    let (sender_name, sender_email) = get_sender_info(message.as_ref());
        format_mailbox(if sender_name.is_empty() { None } else { Some(sender_name) }, if sender_email.is_empty() { None } else { Some(sender_email) })
    };
    // Common lines to render in list or HTML header
    text_out.push(format!("Subject: {}", subject));
    text_out.push(format!("From: {}", from_display));
    text_out.push(format!("Date: {}", received_time));

    let force_full = match action { Action::List(a) => a.full_recipients, Action::Export(a) => a.full_recipients };
    let (mut to_recipients, mut cc_recipients, mut bcc_recipients) = if let Some(headers) = &transport_headers {
        let to_list = headers.to.as_ref().map(|to| parse_recipients_from_header(to)).unwrap_or_default();
        let cc_list = headers.cc.as_ref().map(|cc| parse_recipients_from_header(cc)).unwrap_or_default();
        let bcc_list = headers.bcc.as_ref().map(|bcc| parse_recipients_from_header(bcc)).unwrap_or_default();
        if force_full || (to_list.is_empty() && cc_list.is_empty() && bcc_list.is_empty()) {
            get_recipients(message.as_ref())
        } else {
            (to_list, cc_list, bcc_list)
        }
    } else {
        get_recipients(message.as_ref())
    };
    if force_full {
        // Merge header-derived recipients (if any) for completeness without duplicates
        if let Some(h) = &transport_headers {
            let push_unique = |lst: &mut Vec<String>, incoming: Vec<String>| {
                for r in incoming {
                    if !lst.iter().any(|e| e.eq_ignore_ascii_case(&r)) { lst.push(r); }
                }
            };
            if let Some(ref v) = h.to { push_unique(&mut to_recipients, parse_recipients_from_header(v)); }
            if let Some(ref v) = h.cc { push_unique(&mut cc_recipients, parse_recipients_from_header(v)); }
            if let Some(ref v) = h.bcc { push_unique(&mut bcc_recipients, parse_recipients_from_header(v)); }
        }
    }
    if !to_recipients.is_empty() { text_out.push(format!("To: {}", to_recipients.join("; "))); }
    if !cc_recipients.is_empty() { text_out.push(format!("Cc: {}", cc_recipients.join("; "))); }
    if !bcc_recipients.is_empty() { text_out.push(format!("Bcc: {}", bcc_recipients.join("; "))); }
        // Responsive Emails: if user provided any responsive email addresses, check participants
        let mut responsive_matches: Vec<String> = Vec::new();
        let provided: Vec<String> = match action {
            Action::List(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
            Action::Export(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
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
            Action::Export(a) => a.keywords.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        };
    let mut keyword_count_for_csv: usize = 0;
        if !provided_keywords.is_empty() {
            if let Some(body) = extract_plain_body(message.as_ref()) {
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

    // Retrieve attachments once and reuse across list/export/CSV paths
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
                        // Show non-inline or explicitly requested inline attachments
                        if !info.is_inline {
                            extra_list_lines.push(format!("  - {} (size: {}, method: {}, cid: {})", info.name, info.size, info.attach_method, info.content_id));
                        }
                    }
                }
            }
        }
        if list_args.show_body_types {
            let bodies = detect_body_types(message.as_ref());
            if !bodies.is_empty() { extra_list_lines.push(format!("Bodies: {}", bodies.join(", "))); } else { extra_list_lines.push("Bodies: none".to_string()); }
        }
    }
    if let Action::Export(_) = action {
        if let Some(infos) = attachments.as_ref() {
            if !infos.is_empty() {
                let mut attachments = Vec::new();
                for info in infos.iter() {
                    if !info.is_inline { attachments.push(info.name.clone()); }
                }
                if !attachments.is_empty() {
                    text_out.push(format!("Attachments: {}", attachments.join(" ;;; ")));
                }
            }
        }
    }

    // Render or export
    match action {
        Action::List(_) => {
            println!("{}", "=".repeat(80));
            for line in &text_out { println!("{}", line); }
            for line in &extra_list_lines { println!("{}", line); }
            println!();
        }
    Action::Export(export_args) => {
            // For export, we will write HTML after we update stats to maintain zero-based index
            // Defer actual writing until after stats.add_message below
            // We'll compute and write immediately after bump
            // To keep code simple, we perform export here but compute index using current (pre-increment) value - 0
            // We'll instead write after increment; see below
            let _ = &export_args.out_dir; // marker to avoid unused warning in this branch
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
                Action::Export(a) => a.responsive_emails.iter().map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()).collect(),
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

    // If export, write HTML file now using the zero-based index convention from previous implementation
    if let Action::Export(export_args) = action {
        let out_dir = &export_args.out_dir;
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

    // Optionally export attachments alongside message.html
    if export_args.attachments {
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


fn save_attachments_to_dir(message: &Rc<dyn Message>, out_dir: &Path) -> anyhow::Result<()> {
    let attachments = message.attachments_export()?;
    for (idx, att) in attachments.into_iter().enumerate() {
        // Skip inline unless we want them (current behavior: include all, but mark inline in filename)
        if let Some(ref data) = att.data {
            let base = att.name.unwrap_or_else(|| format!("attachment-{idx:03}.bin"));
            let mut fname: String = base.chars().map(|c| match c { '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_', c if c.is_control() => '_', c => c }).collect();
            if att.is_inline { fname = format!("inline_{}", fname); }
            let mut path = out_dir.to_path_buf();
            path.push(&fname);
            if let Err(e) = fs::write(&path, data) {
                eprintln!("Warning: failed to write attachment {}: {}", path.display(), e);
            }
        }
    }
    Ok(())
}


fn process_folder_recursive(
    store: Rc<dyn Store>,
    folder: &dyn Folder,
    folder_path: &str,
    stats: &mut ProcessingStats,
    action: &Action,
    mut csv_rows: Option<&mut Vec<Vec<String>>>,
    seen_message_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    stats.add_folder();
    if let Some(contents_table) = folder.contents_table() {
    // Deterministic ordering of messages by row id
    let info = contents_table.context();
    let mut rows: Vec<_> = contents_table.rows_matrix().collect();
    rows.sort_by_key(|r| u32::from(r.id()));
    for row in rows {
            let node_id = NodeId::from(u32::from(row.id()));
            match store.properties().make_entry_id(node_id) {
                Ok(entry_id) => match store.open_message(&entry_id, None) {
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
                    Err(e) => {
                        stats.add_error();
                        eprintln!("Warning: Skipped message in folder '{}' due to error: {}", folder_path, e);
                        // Optional diagnostics for missing recipient table
                        let debug_missing_rt = match action { Action::List(a) => a.debug_missing_rt, Action::Export(a) => a.debug_missing_rt };
                        if debug_missing_rt && e.to_string().contains("Missing NID_TYPE_RECIPIENT_TABLE") {
                            // Try to dump a few key properties from the contents table row
                            let columns = match row.columns(info) { Ok(c) => c, Err(_) => Vec::new() };
                            let get_col = |pid: u16| -> Option<outlook_pst::ltp::prop_context::PropertyValue> {
                                for (col_def, value) in info.columns().iter().zip(columns.iter()) {
                                    if col_def.prop_id() == pid {
                                        if let Some(vref) = value.as_ref() {
                                            if let Ok(pv) = contents_table.read_column(vref, col_def.prop_type()) { return Some(pv); }
                                        }
                                    }
                                }
                                None
                            };
                            let msg_class = get_col(0x001A).and_then(|v| match v { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(s) => Some(s.to_string()), _ => None });
                            let subject = get_col(0x0037).and_then(|v| crate::encoding::decode_subject(&v));
                            let recv_time = get_col(0x0E06).and_then(|v| match v { PropertyValue::Time(t) => Some(filetime_to_datetime(t)), _ => None });
                            let flags = get_col(0x0E07).and_then(|v| match v { PropertyValue::Integer32(i) => Some(i), _ => None });
                            let size = get_col(0x0E08).and_then(|v| match v { PropertyValue::Integer32(i) => Some(i), _ => None });
                            let has_atts = get_col(0x0E1B).and_then(|v| match v { PropertyValue::Boolean(b) => Some(b), PropertyValue::Integer32(i) => Some(i != 0), _ => None });
                            let hdr_len = get_col(0x007D).and_then(|v| match v {
                                PropertyValue::String8(s) => Some(s.buffer().len()),
                                PropertyValue::Unicode(s) => Some(s.buffer().len()),
                                _ => None,
                            });

                            eprintln!("  [missing-rt] folder='{}' row_id={} message_class='{}'", folder_path, u32::from(row.id()), msg_class.unwrap_or_else(|| "<unknown>".to_string()));
                            if let Some(s) = subject { eprintln!("  [missing-rt] subject='{}'", s); }
                            if let Some(ts) = recv_time { eprintln!("  [missing-rt] received='{}'", ts); }
                            if let Some(f) = flags { eprintln!("  [missing-rt] flags=0x{:08X}", f); }
                            if let Some(sz) = size { eprintln!("  [missing-rt] message_size={} bytes", sz); }
                            if let Some(ha) = has_atts { eprintln!("  [missing-rt] has_attachments={}", ha); }
                            if let Some(h) = hdr_len { eprintln!("  [missing-rt] transport_headers_length={} chars", h); }
                            if folder_path.contains("Recoverable-Items") { eprintln!("  [missing-rt] note: message is under 'Recoverable-Items' subtree; metadata may be incomplete"); }
                        }
                    }
                },
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
                if let Ok(subfolder) = store.open_folder(&entry_id) {
                    let subfolder_name = match subfolder.properties().display_name() { Ok(name) => name, Err(_) => "Unknown Folder".to_string() };
                    children.push((subfolder_name, node_id_u32));
                }
            }
        }
        children.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        for (subfolder_name, node_id_u32) in children {
            let node_id = NodeId::from(node_id_u32);
            match store.properties().make_entry_id(node_id) {
                Ok(entry_id) => match store.open_folder(&entry_id) {
                    Ok(subfolder) => {
                        let subfolder_path = if folder_path.is_empty() { subfolder_name } else { format!("{}/{}", folder_path, subfolder_name) };
                        let res = if let Some(rows) = &mut csv_rows {
                            process_folder_recursive(
                                store.clone(),
                                subfolder.as_ref(),
                                &subfolder_path,
                                stats,
                                action,
                                Some(*rows),
                                seen_message_ids,
                            )
                        } else {
                            process_folder_recursive(
                                store.clone(),
                                subfolder.as_ref(),
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
                },
                Err(e) => { stats.add_error(); eprintln!("Warning: Could not create entry ID for subfolder: {}", e); }
            }
        }
    }
    Ok(())
}

pub fn collect_pst_files(input: &str) -> anyhow::Result<Vec<PathBuf>> {
    let path = Path::new(input);
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if path.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| p.extension().map(|e| e.eq_ignore_ascii_case("pst")).unwrap_or(false))
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
    let store = open_store(pst_path)?; // unified open
    let ipm_sub_tree = store.properties().ipm_sub_tree_entry_id()?;
    let root_folder = store.open_folder(&ipm_sub_tree)?;
    let store_name = store
        .properties()
        .display_name()
        .unwrap_or_else(|_| "PST Store".to_string());
    println!("Processing emails from PST store: {}", store_name);
    println!();
    process_folder_recursive(store.clone(), root_folder.as_ref(), "", stats, action, csv_rows, seen_message_ids)?;
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
    Action::Export(a) if a.csv => Some(Vec::new()),
        _ => None,
    };
    // Global MessageId set for the entire run
    let mut seen_message_ids: HashSet<String> = HashSet::new();
    for f in files {
        if let Err(e) = process_single_pst(&f, &mut stats, &action, csv_acc.as_mut(), &mut seen_message_ids) {
            eprintln!("Warning: Skipping PST '{}': {}", f.display(), e);
        }
    }
    stats.print_summary();
    // If CSV was requested, write it now
    if let Some(rows) = csv_acc {
        match &action {
            Action::Export(a) if a.csv => {
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

pub fn run_export(args: args::ExportArgs) -> anyhow::Result<()> {
    run_internal_multi(&args.input, Action::Export(&args))
}
