use crate::ConvertError;
use crate::ooxml::dml::{extract_paragraphs, render_paragraph_with_bullet};
use crate::ooxml::util::{Zip, parse_rels, read_zip_text};

use super::types::{MasterStyles, StyleCategory};

/// Read `notes_path` from `zip` and render its paragraphs as a Markdown
/// blockquote. Returns `Ok(None)` when the notes part is missing or
/// contains no rendered text.
pub(super) fn render_speaker_notes(
    zip: &mut Zip<'_>,
    notes_path: &str,
    master_styles: &MasterStyles,
) -> Result<Option<String>, ConvertError> {
    let xml = match read_zip_text(zip, notes_path)? {
        Some(s) => s,
        None => return Ok(None),
    };
    // Notes can carry their own hyperlinks; the rels file is optional.
    let notes_rels_path = notes_path
        .rsplit_once('/')
        .map(|(dir, file)| format!("{dir}/_rels/{file}.rels"))
        .unwrap_or_default();
    let rels = read_zip_text(zip, &notes_rels_path)?
        .as_deref()
        .map(parse_rels)
        .unwrap_or_default();

    let paragraphs = extract_paragraphs(&xml, &rels)?;
    let body: Vec<String> = paragraphs
        .iter()
        .map(|p| {
            let inherited = master_styles.get(&(StyleCategory::Body, p.ilvl));
            render_paragraph_with_bullet(p, inherited)
        })
        .filter(|s| !s.trim().is_empty())
        .collect();
    if body.is_empty() {
        return Ok(None);
    }

    let mut block = String::from("> **Notes:**\n>\n");
    for (i, p) in body.iter().enumerate() {
        if i > 0 {
            block.push_str(">\n");
        }
        for line in p.lines() {
            if line.is_empty() {
                block.push_str(">\n");
            } else {
                block.push_str("> ");
                block.push_str(line);
                block.push('\n');
            }
        }
    }
    Ok(Some(block))
}
