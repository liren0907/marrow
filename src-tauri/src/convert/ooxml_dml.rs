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

use quick_xml::events::{BytesStart, BytesText};

use crate::convert::ooxml_util::{escape_md, wrap_run};

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
    pub hyperlink: Option<String>,
}

#[derive(Default, Debug)]
pub struct Paragraph {
    pub bullet: BulletKind,
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
                    p.bullet = BulletKind::Char;
                }
            }
            b"buAutoNum" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = BulletKind::AutoNum;
                }
            }
            b"buNone" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = BulletKind::None;
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
                    p.bullet = BulletKind::Char;
                }
            }
            b"buAutoNum" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = BulletKind::AutoNum;
                }
            }
            b"buNone" => {
                if let Some(p) = self.cur_p.as_mut() {
                    p.bullet = BulletKind::None;
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

pub fn render_paragraph(p: &Paragraph) -> String {
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        let text_escaped = escape_md(&r.text);
        let wrapped = wrap_run(&text_escaped, r.bold, r.italic);
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

    match p.bullet {
        BulletKind::None => body,
        BulletKind::Char => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}- {body}")
        }
        BulletKind::AutoNum => {
            let indent = "  ".repeat(p.ilvl);
            format!("{indent}1. {body}")
        }
    }
}
