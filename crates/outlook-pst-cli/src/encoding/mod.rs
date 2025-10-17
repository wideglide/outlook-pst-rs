use compressed_rtf::*;
use outlook_pst::ltp::prop_context::PropertyValue;
use base64::{engine::general_purpose, Engine as _};

/// Decode RFC 2047 encoded-words inside a header value (e.g., "=?utf-8?B?...?=").
/// Performs a single left-to-right pass; if decoding fails for a token it is left as-is.
pub fn decode_rfc2047(s: &str) -> String {
    if !s.contains("=?") { return s.to_string(); }
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 4 < bytes.len() && bytes[i] == b'=' && bytes[i+1] == b'?' { // minimal =?x?e??=
            let start = i + 2; // charset start
            // find first '?'
            if let Some(rel_q1) = s[start..].find('?') {
                let q1 = start + rel_q1; // position of first '?'
                let charset = &s[start..q1];
                let enc_start = q1 + 1;
                if enc_start >= s.len() { out.push(bytes[i] as char); i += 1; continue; }
                if let Some(rel_q2) = s[enc_start..].find('?') { // end of encoding token
                    let q2 = enc_start + rel_q2; // position of second '?'
                    if q2 + 2 <= s.len() { // ensure room for data and terminator later
                        let data_start = q2 + 1;
                        // Find terminator '?=' starting AFTER second '?'
                        if let Some(term_rel) = s[data_start..].find("?=") {
                            let term_pos = data_start + term_rel; // position of '?'
                            let enc_token = &s[enc_start..q2];
                            let encoded_text = &s[data_start..term_pos];
                            if !enc_token.is_empty() {
                                let decoded = if enc_token.eq_ignore_ascii_case("B") {
                                    let cleaned = encoded_text.replace(['\r', '\n'], "");
                                    general_purpose::STANDARD.decode(cleaned.as_bytes()).ok()
                                        .and_then(|raw| decode_bytes_charset(&raw, charset))
                                } else if enc_token.eq_ignore_ascii_case("Q") {
                                    let qp = encoded_text.replace('_', " ");
                                    let mut buf = Vec::with_capacity(qp.len());
                                    let qb = qp.as_bytes();
                                    let mut j=0; while j<qb.len() { if qb[j]==b'=' && j+2<qb.len() && qb[j+1].is_ascii_hexdigit() && qb[j+2].is_ascii_hexdigit() { if let Ok(v)=u8::from_str_radix(&qp[j+1..j+3],16){ buf.push(v); j+=3; continue; } } else if qb[j]==b'=' && j+1==qb.len() { break; } buf.push(qb[j]); j+=1; }
                                    decode_bytes_charset(&buf, charset)
                                } else { None };
                                if let Some(text) = decoded { out.push_str(&text); i = term_pos + 2; continue; }
                            }
                        }
                    }
                }
            }
        }
        out.push(bytes[i] as char); i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::decode_rfc2047;

    #[test]
    fn decodes_q_with_leading_hex_sequence() {
        let s = "=?UTF-8?q?=F0=9F=A4=A1_Clown_Face_Emoji?=";
        let decoded = decode_rfc2047(s);
        assert!(decoded.contains("Clown Face Emoji"), "decoded: {}", decoded);
    }

    #[test]
    fn decodes_b_basic() {
        let s = "=?utf-8?B?SGVsbG8gV29ybGQ=?=";
        let decoded = decode_rfc2047(s);
        assert_eq!(decoded, "Hello World");
    }
}

fn decode_bytes_charset(data: &[u8], charset: &str) -> Option<String> {
    let lower = charset.to_ascii_lowercase();
    if lower.contains("bad_stoic") {
        eprintln!("Error: Found Bad_Stoic charset in encoded-word!\n Charset: '{}'\n Data: {:?}\n", charset, data);
    }
    match lower.as_str() {
        "utf-8" | "utf8" => Some(String::from_utf8_lossy(data).to_string()),
        "us-ascii" | "ascii" => Some(String::from_utf8_lossy(data).to_string()),
        // Common legacy charsets could be mapped via codepage_strings if needed; fallback lossily
        _ => Some(String::from_utf8_lossy(data).to_string()),
    }
}

/// Decode a subject line from a MAPI property value.
///
/// Observed oddity: some subjects are prefixed with a single 0x01 sentinel (common in OST/PST
/// data for certain wrapped strings). We strip exactly that leading marker when present.
///
/// Behaviour:
/// - String8: treat underlying bytes as the native code page already normalized by upstream
///   property decoding (assume ASCII/Windows-1252 superset) and convert lossily to UTF-8.
///   We DO NOT widen bytes to UTF-16 (previous implementation did and produced mojibake).
/// - Unicode: underlying buffer is UTF-16; if first u16 == 0x0001 treat it as sentinel and skip it.
pub fn decode_subject(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::String8(raw) => {
            let buf = raw.buffer();
            let slice = if matches!(buf.first(), Some(1)) { &buf[1..] } else { buf };
            // Truncate at first NUL if present (defensive; some implementations include trailing NUL)
            let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
            let s = String::from_utf8_lossy(&slice[..end]).to_string();
            Some(decode_rfc2047(&s))
        }
        PropertyValue::Unicode(raw) => {
            let buf = raw.buffer();
            let slice = if matches!(buf.first(), Some(1)) { &buf[1..] } else { buf };
            // Trim trailing UTF-16 NUL if present
            let mut end = slice.len();
            if end > 0 && slice[end-1] == 0 { end -= 1; }
            let s = String::from_utf16_lossy(&slice[..end]);
            Some(decode_rfc2047(&s))
        }
        _ => None,
    }
}

pub fn decode_html_body(buffer: &[u8], code_page: u16) -> Option<String> {
    match code_page {
        20127 => {
            let buffer: Vec<_> = buffer.iter().map(|&b| u16::from(b)).collect();
            Some(String::from_utf16_lossy(&buffer))
        }
        _ => {
            let coding = codepage_strings::Coding::new(code_page).ok()?;
            Some(coding.decode(buffer).ok()?.to_string())
        }
    }
}

pub fn decode_rtf_compressed(buffer: &[u8]) -> Option<String> {
    decompress_rtf(buffer).ok()
}
