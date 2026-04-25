use std::collections::{HashMap, HashSet};

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_util::{Zip, read_zip_text};

use super::attrs::attr_val;

/// Load footnotes / endnotes from `word/footnotes.xml` or
/// `word/endnotes.xml`. `wrapper` is the local-name of the per-note
/// element (`footnote` or `endnote`). Returns id → flattened body. Skips
/// the implicit separator/continuation-separator notes (id 0 / -1) which
/// have `w:type="separator"` and carry no real content.
pub(super) fn load_notes(
    zip: &mut Zip,
    path: &str,
    wrapper: &str,
) -> Result<HashMap<String, String>, ConvertError> {
    let mut out = HashMap::new();
    let xml = match read_zip_text(zip, path)? {
        Some(s) => s,
        None => return Ok(out),
    };
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let wrapper_b = wrapper.as_bytes();
    let mut cur_id: Option<String> = None;
    let mut skip = false;
    let mut cur_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == wrapper_b => {
                cur_id = attr_val(&e, b"id");
                let ty = attr_val(&e, b"type");
                skip = matches!(
                    ty.as_deref(),
                    Some("separator") | Some("continuationSeparator")
                );
                cur_text.clear();
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == wrapper_b => {
                if let Some(id) = cur_id.take() {
                    if !skip {
                        out.insert(id, cur_text.trim().to_string());
                    }
                }
                cur_text.clear();
                skip = false;
            }
            Ok(Event::Text(t)) => {
                if cur_id.is_some() && !skip {
                    if let Ok(raw) = t.decode() {
                        let trimmed = raw.trim();
                        if !trimmed.is_empty() {
                            if !cur_text.is_empty() && !cur_text.ends_with(' ') {
                                cur_text.push(' ');
                            }
                            cur_text.push_str(trimmed);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// Scan rendered markdown for `[^fn123]` / `[^en123]` references and
/// return the unique referenced ids in first-occurrence order. Pads the
/// definitions tail to only what was actually cited.
pub(super) fn footnote_ids_in(md: &str, prefix: &str) -> Vec<String> {
    let needle = format!("[^{prefix}");
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let bytes = md.as_bytes();
    let mut i = 0;
    while let Some(pos) = md[i..].find(&needle) {
        let start = i + pos + needle.len();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start && bytes.get(end) == Some(&b']') {
            let id = &md[start..end];
            if seen.insert(id.to_string()) {
                out.push(id.to_string());
            }
        }
        i = end;
    }
    out
}
