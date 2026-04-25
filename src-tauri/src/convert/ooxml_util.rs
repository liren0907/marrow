use std::collections::{HashMap, HashSet};
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

/// Read a file from the zip as raw bytes. Returns `Ok(None)` when the
/// entry does not exist (treat optional parts the same way as
/// [`read_zip_text`]).
pub fn read_zip_bytes(zip: &mut Zip, path: &str) -> Result<Option<Vec<u8>>, ConvertError> {
    let mut file = match zip.by_name(path) {
        Ok(f) => f,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(ConvertError::Zip(format!("{e}"))),
    };
    let mut buf = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buf)?;
    Ok(Some(buf))
}

/// List zip entry names matching a prefix. Useful for enumerating
/// `ppt/slides/slide*.xml`. Returns names in insertion order.
pub fn list_zip_names(zip: &Zip, prefix: &str) -> Vec<String> {
    zip.file_names()
        .filter(|n| n.starts_with(prefix))
        .map(|n| n.to_string())
        .collect()
}

/// Collect every `Target` whose `Relationship` has a `Type` attribute
/// ending with `type_suffix`. Use when a part can have multiple sibling
/// relationships of the same kind (e.g. modern PPTX comments — one rel
/// per comment file).
pub fn find_rel_targets(xml: &str, type_suffix: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let mut ty = None;
                let mut target = None;
                for attr in e.attributes().flatten() {
                    match attr.key.local_name().as_ref() {
                        b"Type" => ty = attr.unescape_value().ok().map(|c| c.into_owned()),
                        b"Target" => target = attr.unescape_value().ok().map(|c| c.into_owned()),
                        _ => {}
                    }
                }
                if let (Some(ty), Some(target)) = (ty, target) {
                    if ty.ends_with(type_suffix) {
                        out.push(target);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Find the `Target` of the first `Relationship` whose `Type` attribute
/// ends with `type_suffix`. Used to locate optional sibling parts (notes
/// slide, comments, etc.) without enumerating every relationship type.
pub fn find_rel_target(xml: &str, type_suffix: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let mut ty = None;
                let mut target = None;
                for attr in e.attributes().flatten() {
                    match attr.key.local_name().as_ref() {
                        b"Type" => ty = attr.unescape_value().ok().map(|c| c.into_owned()),
                        b"Target" => target = attr.unescape_value().ok().map(|c| c.into_owned()),
                        _ => {}
                    }
                }
                if ty.as_deref().map(|t| t.ends_with(type_suffix)).unwrap_or(false) {
                    return target;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Resolve a relationship `Target` (which is relative to the part that
/// declares it) against the source part's path.
///
/// Example: source `ppt/slides/slide5.xml`, target `../notesSlides/notesSlide5.xml`
/// resolves to `ppt/notesSlides/notesSlide5.xml`.
pub fn resolve_rel_path(source_part: &str, target: &str) -> String {
    let mut parts: Vec<&str> = source_part
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    for seg in target.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            other => parts.push(other),
        }
    }
    parts.join("/")
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

/// Wrap a run's text with bold / italic / underline / strikethrough
/// markers in a stable order. Like [`wrap_run`] but covers the full DML /
/// WordprocessingML run vocabulary. Shared by docx and pptx.
pub fn wrap_run_full(
    text: &str,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
) -> String {
    if text.is_empty() {
        return String::new();
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }
    if !bold && !italic && !underline && !strike {
        return text.to_string();
    }
    let leading_len = text.len() - text.trim_start().len();
    let trailing_len = text.len() - text.trim_end().len();
    let leading = &text[..leading_len];
    let trailing = &text[text.len() - trailing_len..];
    let mut out = String::with_capacity(text.len() + 12);
    out.push_str(leading);
    if bold {
        out.push_str("**");
    }
    if italic {
        out.push('*');
    }
    if underline {
        out.push_str("<u>");
    }
    if strike {
        out.push_str("~~");
    }
    out.push_str(trimmed);
    if strike {
        out.push_str("~~");
    }
    if underline {
        out.push_str("</u>");
    }
    if italic {
        out.push('*');
    }
    if bold {
        out.push_str("**");
    }
    out.push_str(trailing);
    out
}

/// Pick a non-conflicting filename for a sidecar asset. If `base` is
/// unused, returned verbatim; otherwise we append `-2`, `-3`, … before
/// the extension until a free name is found.
pub fn unique_asset_name(base: &str, used: &HashSet<String>) -> String {
    if !used.contains(base) {
        return base.to_string();
    }
    let (stem, ext) = match base.rfind('.') {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, ""),
    };
    let mut n = 2usize;
    loop {
        let candidate = format!("{stem}-{n}{ext}");
        if !used.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
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
