use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::ConvertError;
use crate::ooxml::util::{Zip, read_zip_text};

/// Render every header*.xml + footer*.xml in the zip. Order is alpha by
/// filename (ZIP-order in practice), which matches the section order. Text
/// runs are flattened (no formatting).
pub(super) fn render_headers_footers(
    zip: &mut Zip,
) -> Result<(String, String), ConvertError> {
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.starts_with("word/header") || n.starts_with("word/footer"))
        .filter(|n| n.ends_with(".xml"))
        .map(|s| s.to_string())
        .collect();
    let mut headers = String::new();
    let mut footers = String::new();
    for name in names {
        let xml = match read_zip_text(zip, &name)? {
            Some(s) => s,
            None => continue,
        };
        let body = flatten_paragraphs(&xml);
        if body.trim().is_empty() {
            continue;
        }
        if name.contains("header") {
            if !headers.is_empty() {
                headers.push('\n');
            }
            headers.push_str(&body);
        } else {
            if !footers.is_empty() {
                footers.push('\n');
            }
            footers.push_str(&body);
        }
    }
    Ok((headers, footers))
}

/// Strip-down walker that emits one line of text per `<w:p>`. Used for
/// headers / footers where we don't care about styles, lists, or rels.
fn flatten_paragraphs(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut cur = String::new();
    let mut in_run = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"r" => {
                in_run = true;
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"r" => {
                in_run = false;
            }
            Ok(Event::Text(t)) if in_run => {
                if let Ok(raw) = t.decode() {
                    cur.push_str(&raw);
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => {
                let line = cur.trim();
                if !line.is_empty() {
                    out.push_str(line);
                    out.push('\n');
                }
                cur.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}
