use std::collections::HashMap;
use std::io::{Cursor, Read};

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use zip::ZipArchive;
use zip::result::ZipError;

use crate::convert::ConvertError;

pub type Zip<'a> = ZipArchive<Cursor<&'a [u8]>>;

pub fn open_zip(bytes: &[u8]) -> Result<Zip<'_>, ConvertError> {
    ZipArchive::new(Cursor::new(bytes)).map_err(|e| ConvertError::Zip(format!("{e}")))
}

/// Read a file from the zip as UTF-8 text. Returns `Ok(None)` when the entry
/// does not exist (e.g. optional rels / numbering / styles files).
pub fn read_zip_text(zip: &mut Zip, path: &str) -> Result<Option<String>, ConvertError> {
    let mut file = match zip.by_name(path) {
        Ok(f) => f,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(ConvertError::Zip(format!("{e}"))),
    };
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(Some(s))
}

/// List zip entry names matching a prefix. Useful for enumerating
/// `ppt/slides/slide*.xml`. Returns names in insertion order.
pub fn list_zip_names(zip: &Zip, prefix: &str) -> Vec<String> {
    zip.file_names()
        .filter(|n| n.starts_with(prefix))
        .map(|n| n.to_string())
        .collect()
}

/// Parse an OOXML `<Relationships>` XML document → `HashMap<Id, Target>`.
pub fn parse_rels(xml: &str) -> HashMap<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut map = HashMap::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let mut id = None;
                let mut target = None;
                for attr in e.attributes().flatten() {
                    match attr.key.local_name().as_ref() {
                        b"Id" => {
                            id = attr.unescape_value().ok().map(|c| c.into_owned());
                        }
                        b"Target" => {
                            target = attr.unescape_value().ok().map(|c| c.into_owned());
                        }
                        _ => {}
                    }
                }
                if let (Some(i), Some(t)) = (id, target) {
                    map.insert(i, t);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

/// Wrap a text run with bold/italic markers. Handles:
/// - empty input → empty output
/// - pure-whitespace input → passthrough (markers would otherwise swallow whitespace)
/// - preserves leading/trailing whitespace outside the markers
pub fn wrap_run(text: &str, bold: bool, italic: bool) -> String {
    if text.is_empty() {
        return String::new();
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }
    if !bold && !italic {
        return text.to_string();
    }
    let leading_len = text.len() - text.trim_start().len();
    let trailing_len = text.len() - text.trim_end().len();
    let leading = &text[..leading_len];
    let trailing = &text[text.len() - trailing_len..];
    let mut out = String::with_capacity(text.len() + 6);
    out.push_str(leading);
    if bold {
        out.push_str("**");
    }
    if italic {
        out.push('*');
    }
    out.push_str(trimmed);
    if italic {
        out.push('*');
    }
    if bold {
        out.push_str("**");
    }
    out.push_str(trailing);
    out
}

/// Escape Markdown meta-characters in plain text (for run bodies we emit
/// into a paragraph).
pub fn escape_md(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Collapse runs of 3+ newlines to exactly 2, trim, ensure a single
/// trailing newline. Shared post-process for docx / pptx output.
pub fn post_process(md: String) -> String {
    let mut out = String::with_capacity(md.len());
    let mut run = 0usize;
    for c in md.chars() {
        if c == '\n' {
            run += 1;
            if run <= 2 {
                out.push(c);
            }
        } else {
            run = 0;
            out.push(c);
        }
    }
    let mut trimmed = out.trim().to_string();
    trimmed.push('\n');
    trimmed
}
