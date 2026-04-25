use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::ConvertError;
use crate::ooxml::util::{Zip, read_zip_text};

use super::attrs::attr_val;
use super::types::{Ctx, Paragraph};

/// Load `word/comments.xml` if present. Returns `(authors, texts)`,
/// each keyed by comment id. Text is flattened: paragraphs collapse to
/// space-joined runs, formatting is dropped.
pub(super) fn load_comments(
    zip: &mut Zip,
) -> Result<(HashMap<String, String>, HashMap<String, String>), ConvertError> {
    let mut authors = HashMap::new();
    let mut texts = HashMap::new();
    let xml = match read_zip_text(zip, "word/comments.xml")? {
        Some(s) => s,
        None => return Ok((authors, texts)),
    };
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut cur_id: Option<String> = None;
    let mut cur_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.local_name().as_ref() == b"comment" => {
                cur_id = attr_val(&e, b"id");
                if let Some(author) = attr_val(&e, b"author") {
                    if let Some(id) = cur_id.as_deref() {
                        authors.insert(id.to_string(), author);
                    }
                }
                cur_text.clear();
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"comment" => {
                if let Some(id) = cur_id.take() {
                    texts.insert(id, cur_text.trim().to_string());
                }
                cur_text.clear();
            }
            Ok(Event::Text(t)) => {
                if cur_id.is_some() {
                    if let Ok(raw) = t.decode() {
                        if !cur_text.is_empty() && !cur_text.ends_with(' ') {
                            cur_text.push(' ');
                        }
                        cur_text.push_str(raw.trim());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((authors, texts))
}

/// Render the comment blockquotes attached to `p`. Returns "" when there
/// are no comments. Each comment becomes a `> 💬 **Author**: text` line.
pub(super) fn render_comments_for(p: &Paragraph, ctx: &Ctx) -> String {
    if p.comment_ids.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for id in &p.comment_ids {
        let author = ctx
            .comment_authors
            .get(id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let text = ctx.comment_texts.get(id).cloned().unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        out.push_str(&format!("> 💬 **{author}**: {text}\n"));
    }
    out
}
