use crate::convert::ooxml_util::{escape_md, wrap_run_full};

use super::types::{Ctx, ListKind, Paragraph, Run};

#[derive(Default)]
pub(super) struct TableAcc {
    pub(super) rows: Vec<Vec<String>>,
}

pub(super) fn render_paragraph(p: &Paragraph, ctx: &Ctx) -> String {
    // Merge adjacent runs with identical formatting (cheap dedup before
    // we pay for markdown markers).
    let merged = merge_adjacent_runs(&p.runs);
    let mut body = String::new();
    for r in &merged {
        let escaped = escape_md_preserve_brackets(&r.text);
        body.push_str(&wrap_run_full(
            &escaped,
            r.bold,
            r.italic,
            r.underline,
            r.strike,
        ));
    }
    let body = body.trim().to_string();

    // Heading?
    if let Some(id) = &p.style_id {
        if let Some(level) = ctx.styles.get(id).copied() {
            let hash = "#".repeat(level);
            return if body.is_empty() {
                String::new()
            } else {
                format!("\n{hash} {body}\n")
            };
        }
    }

    // List?
    if let Some(num_id) = &p.num_id {
        let kind = ctx
            .numbering
            .get(&(num_id.clone(), p.ilvl))
            .copied()
            .unwrap_or(ListKind::Bullet);
        let marker = match kind {
            ListKind::Bullet => "-".to_string(),
            ListKind::Ordered => "1.".to_string(),
        };
        let indent = "  ".repeat(p.ilvl);
        return format!("{indent}{marker} {body}");
    }

    if body.is_empty() {
        String::new()
    } else {
        format!("{body}\n")
    }
}

pub(super) fn render_table(t: &TableAcc) -> String {
    if t.rows.is_empty() {
        return String::new();
    }
    let cols = t.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let mut out = String::new();
    let header = normalize_row(&t.rows[0], cols);
    out.push_str(&format_row(&header));
    out.push('\n');
    out.push_str(&format_row(&vec!["---".to_string(); cols]));
    out.push('\n');
    for row in t.rows.iter().skip(1) {
        out.push_str(&format_row(&normalize_row(row, cols)));
        out.push('\n');
    }
    out
}

fn normalize_row(row: &[String], cols: usize) -> Vec<String> {
    let mut v: Vec<String> = row
        .iter()
        .map(|c| c.replace('|', "\\|").replace('\n', " ").trim().to_string())
        .collect();
    while v.len() < cols {
        v.push(String::new());
    }
    v
}

fn format_row(cells: &[String]) -> String {
    let mut s = String::from("|");
    for c in cells {
        s.push(' ');
        s.push_str(c);
        s.push_str(" |");
    }
    s
}

pub(super) fn merge_adjacent_runs(runs: &[Run]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for r in runs {
        if let Some(last) = out.last_mut() {
            if last.bold == r.bold
                && last.italic == r.italic
                && last.underline == r.underline
                && last.strike == r.strike
            {
                last.text.push_str(&r.text);
                continue;
            }
        }
        out.push(r.clone());
    }
    out
}

/// Same as escape_md in ooxml_util but hyperlink runs arrive pre-formatted
/// with `[..](..)` — don't double-escape brackets if the text is clearly
/// already a link form. Heuristic: skip escaping when text matches `[*](*)`.
fn escape_md_preserve_brackets(text: &str) -> String {
    if looks_like_link(text) || looks_like_image(text) || looks_like_footnote(text) {
        return text.to_string();
    }
    escape_md(text)
}

fn looks_like_link(text: &str) -> bool {
    if !text.starts_with('[') {
        return false;
    }
    text.contains("](") && text.ends_with(')')
}

fn looks_like_image(text: &str) -> bool {
    text.starts_with("![") && text.contains("](") && text.ends_with(')')
}

fn looks_like_footnote(text: &str) -> bool {
    // [^fn3] or [^en7]
    text.starts_with("[^") && text.ends_with(']')
}
