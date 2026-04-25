//! Shared DrawingML text-body parser.
//!
//! Walks the contents of `<a:txBody>` / `<p:txBody>` (plus the run/paragraph
//! grammar that lives inside `<a:tc>`) and produces paragraph + run data
//! ready for markdown rendering. The outer file-format walker (pptx today,
//! xlsx later) drives this parser by forwarding every quick-xml event whose
//! local-name belongs to the DML text grammar:
//! `p`, `pPr`, `buChar`, `buAutoNum`, `buNone`, `r`, `rPr`, `hlinkClick`,
//! `t`, `br`.
//!
//! Each `</p>` finishes a paragraph and appends it to the parser's internal
//! buffer. The outer walker calls [`TextBodyParser::take_paragraphs`] at the
//! enclosing-element boundary (`</tc>`, `</sp>`, `</txBody>`) to drain.

use std::collections::HashMap;

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::reader::Reader;

use crate::convert::ConvertError;
use crate::convert::ooxml_util::escape_md;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum BulletKind {
    #[default]
    None,
    Char,
    AutoNum,
}

#[derive(Default, Clone, Debug)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub hyperlink: Option<String>,
}

#[derive(Default, Debug)]
pub struct Paragraph {
    /// `None` means "no `<a:buXxx>` was seen on this paragraph" — caller
    /// should fall back to the inherited style from layout/master.
    /// `Some(BulletKind::None)` means the paragraph explicitly declared
    /// `<a:buNone>` which suppresses an inherited bullet.
    pub bullet: Option<BulletKind>,
    pub ilvl: usize,
    pub runs: Vec<Run>,
}

#[derive(Default)]
pub struct TextBodyParser {
    cur_p: Option<Paragraph>,
    in_run: bool,
    cur_run: Run,
    pending_hyperlink: Option<String>,
    paragraphs: Vec<Paragraph>,
}

impl TextBodyParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_start(
        &mut self,
        name: &[u8],
        e: &BytesStart,
        rels: &HashMap<String, String>,
    ) {
        match name {
            b"p" => {
                self.cur_p = Some(Paragraph::default());
            }
            b"pPr" => {
                if let Some(p) = self.cur_p.as_mut() {
                    if let Some(v) = attr_val(e, b"lvl") {
                        p.ilvl = v.parse().unwrap_or(0);
                    }
                }
            }
            b"buChar" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::Char);
                }
            }
            b"buAutoNum" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::AutoNum);
                }
            }
            b"buNone" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::None);
                }
            }
            b"r" => {
                self.in_run = true;
                self.cur_run = Run {
                    hyperlink: self.pending_hyperlink.clone(),
                    ..Default::default()
                };
            }
            b"rPr" => {
                self.apply_run_pr(e);
            }
            b"hlinkClick" => {
                if let Some(url) = resolve_hyperlink(e, rels) {
                    self.cur_run.hyperlink = Some(url.clone());
                    self.pending_hyperlink = Some(url);
                }
            }
            _ => {}
        }
    }

    pub fn on_empty(
        &mut self,
        name: &[u8],
        e: &BytesStart,
        rels: &HashMap<String, String>,
    ) {
        match name {
            b"buChar" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::Char);
                }
            }
            b"buAutoNum" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::AutoNum);
                }
            }
            b"buNone" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = Some(BulletKind::None);
                }
            }
            b"rPr" => {
                self.apply_run_pr(e);
            }
            b"br" => {
                if self.in_run {
                    // Markdown hard line break: two trailing spaces + newline.
                    self.cur_run.text.push_str("  \n");
                }
            }
            b"hlinkClick" => {
                if let Some(url) = resolve_hyperlink(e, rels) {
                    self.cur_run.hyperlink = Some(url);
                }
            }
            _ => {}
        }
    }

    pub fn on_text(&mut self, t: &BytesText) {
        if !self.in_run {
            return;
        }
        if let Ok(raw) = t.decode() {
            let un = quick_xml::escape::unescape(&raw)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| raw.into_owned());
            self.cur_run.text.push_str(&un);
        }
    }

    pub fn on_end(&mut self, name: &[u8]) {
        match name {
            b"r" => {
                self.in_run = false;
                if !self.cur_run.text.is_empty() {
                    if let Some(p) = self.cur_p.as_mut() {
                        p.runs.push(std::mem::take(&mut self.cur_run));
                    }
                } else {
                    self.cur_run = Run::default();
                }
            }
            b"hlinkClick" => {
                self.pending_hyperlink = None;
            }
            b"p" => {
                if let Some(p) = self.cur_p.take() {
                    self.paragraphs.push(p);
                }
            }
            _ => {}
        }
    }

    pub fn take_paragraphs(&mut self) -> Vec<Paragraph> {
        std::mem::take(&mut self.paragraphs)
    }

    fn apply_run_pr(&mut self, e: &BytesStart) {
        if let Some(v) = attr_val(e, b"b") {
            if v == "1" || v == "true" {
                self.cur_run.bold = true;
            }
        }
        if let Some(v) = attr_val(e, b"i") {
            if v == "1" || v == "true" {
                self.cur_run.italic = true;
            }
        }
        // `u="none"` is the only "no underline" value; sng / dbl /
        // wavy* / dotted etc. all read as some form of underline.
        if let Some(v) = attr_val(e, b"u") {
            if v != "none" {
                self.cur_run.underline = true;
            }
        }
        // `strike="noStrike"` means "explicitly not struck"; everything
        // else (sngStrike / dblStrike) is a strikethrough.
        if let Some(v) = attr_val(e, b"strike") {
            if v != "noStrike" {
                self.cur_run.strike = true;
            }
        }
    }
}

/// Resolve a `<a:hlinkClick>` element to its final URL.
///
/// `r:id` looks up a target in `rels`. When the element also carries
/// `action="ppaction://hlinksldjump"` the target is a sibling slide path
/// (e.g. `../slides/slide3.xml`), which we collapse to an in-document
/// anchor `#slide-3` so it does not show up as a broken external link.
/// Other actions (e.g. `hlinkpres` jump-to-show) currently fall through
/// to the raw target.
fn resolve_hyperlink(
    e: &BytesStart,
    rels: &HashMap<String, String>,
) -> Option<String> {
    let id = attr_val(e, b"id")?;
    let target = rels.get(&id)?;
    if attr_val(e, b"action")
        .map(|a| a.starts_with("ppaction://hlinksldjump"))
        .unwrap_or(false)
    {
        let stem = target.rsplit('/').next().unwrap_or(target.as_str());
        if let Some(n) = stem
            .strip_prefix("slide")
            .and_then(|s| s.strip_suffix(".xml"))
        {
            return Some(format!("#slide-{n}"));
        }
    }
    Some(target.clone())
}

fn attr_val(e: &BytesStart, key: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == key {
            return attr.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

pub fn merge_adjacent_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for r in runs {
        if let Some(last) = out.last_mut() {
            if last.bold == r.bold
                && last.italic == r.italic
                && last.underline == r.underline
                && last.strike == r.strike
                && last.hyperlink == r.hyperlink
            {
                last.text.push_str(&r.text);
                continue;
            }
        }
        out.push(r.clone());
    }
    out
}

/// Wrap a run's text with bold / italic / underline / strikethrough
/// markers in a stable order (`**…**` outside, `*…*`, then `<u>…</u>`,
/// then `~~…~~`). Mirrors [`crate::convert::ooxml_util::wrap_run`] but
/// covers the full DML run vocabulary.
fn wrap_run_full(text: &str, bold: bool, italic: bool, underline: bool, strike: bool) -> String {
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

/// Walk an arbitrary DML/PML XML document and collect every `<a:p>` it
/// contains. Use for parts where the outer file-format walker has no
/// shape-level concerns to track — e.g. notes slides, comments, charts.
pub fn extract_paragraphs(
    xml: &str,
    rels: &HashMap<String, String>,
) -> Result<Vec<Paragraph>, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut parser = TextBodyParser::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let n = e.local_name().as_ref().to_vec();
                parser.on_start(&n, &e, rels);
            }
            Ok(Event::Empty(e)) => {
                let n = e.local_name().as_ref().to_vec();
                parser.on_empty(&n, &e, rels);
            }
            Ok(Event::Text(t)) => parser.on_text(&t),
            Ok(Event::End(e)) => {
                let n = e.local_name().as_ref().to_vec();
                parser.on_end(&n);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConvertError::Xml(format!("{e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(parser.take_paragraphs())
}

pub fn render_paragraph(p: &Paragraph) -> String {
    render_paragraph_with_bullet(p, None)
}

/// Render a paragraph with an optional `inherited` bullet style used when
/// the paragraph itself did not declare one (`p.bullet == None`). An
/// explicit `Some(BulletKind::None)` on the paragraph still suppresses
/// any inherited bullet — that's the whole point of `<a:buNone/>`.
pub fn render_paragraph_with_bullet(
    p: &Paragraph,
    inherited: Option<&BulletKind>,
) -> String {
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        let text_escaped = escape_md(&r.text);
        let wrapped =
            wrap_run_full(&text_escaped, r.bold, r.italic, r.underline, r.strike);
        if let Some(url) = &r.hyperlink {
            body.push_str(&format!("[{}]({})", wrapped, url));
        } else {
            body.push_str(&wrapped);
        }
    }
    let body = body.trim().to_string();

    if body.is_empty() {
        return String::new();
    }

    let effective = match &p.bullet {
        Some(k) => Some(k),
        None => inherited,
    };
    match effective {
        None | Some(BulletKind::None) => body,
        Some(BulletKind::Char) => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}- {body}")
        }
        Some(BulletKind::AutoNum) => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}1. {body}")
        }
    }
}
