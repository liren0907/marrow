//! Native DOCX → Markdown conversion.
//!
//! Entry point: [`docx_to_markdown`]. Sub-modules:
//!
//! - [`types`] — `Run`, `Paragraph`, `Ctx`, list/numbering data
//! - [`attrs`] — `attr_val`, `attr_val_with_prefix` quick-xml helpers
//! - [`styles`] / [`numbering`] — parsers for `word/styles.xml` and
//!   `word/numbering.xml` respectively
//! - [`images`] — picture asset collection from `word/media/`
//! - [`comments`] / [`notes`] — `word/comments.xml`,
//!   `word/footnotes.xml`, `word/endnotes.xml` loaders + renderers
//! - [`headers`] — `word/header*.xml` / `word/footer*.xml` flatteners
//! - [`fields`] — old-style HYPERLINK field-code state machine
//! - [`walker`] — the main `word/document.xml` streaming pass
//! - [`render`] — paragraph / table markdown rendering helpers

use crate::ConvertError;
use crate::ooxml::util::{
    open_zip, parse_rels, post_process, read_zip_text,
};

mod attrs;
mod comments;
mod fields;
mod headers;
mod images;
mod notes;
mod numbering;
mod render;
mod styles;
mod types;
mod walker;

use comments::load_comments;
use headers::render_headers_footers;
use images::collect_image_assets;
use notes::{footnote_ids_in, load_notes};
use numbering::parse_numbering;
use styles::parse_styles;
use types::Ctx;
use walker::walk_document;

/// Sidecar asset extracted from the docx (`word/media/*`).
pub struct DocxAsset {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct DocxResult {
    pub markdown: String,
    pub assets: Vec<DocxAsset>,
}

pub fn docx_to_markdown(bytes: &[u8]) -> Result<DocxResult, ConvertError> {
    let mut zip = open_zip(bytes)?;
    let rels = read_zip_text(&mut zip, "word/_rels/document.xml.rels")?
        .map(|s| parse_rels(&s))
        .unwrap_or_default();
    let styles = read_zip_text(&mut zip, "word/styles.xml")?
        .map(|s| parse_styles(&s))
        .unwrap_or_default();
    let numbering = read_zip_text(&mut zip, "word/numbering.xml")?
        .map(|s| parse_numbering(&s))
        .unwrap_or_default();

    // Pictures: walk every Image relationship and pull the bytes once,
    // de-duping by zip path. The walker only needs `image_rels` to
    // resolve r:embed → final attachment filename.
    let (image_rels, assets) = collect_image_assets(&mut zip, &rels)?;

    // Comments: load author/text up front so the walker can refer by id.
    let (comment_authors, comment_texts) = load_comments(&mut zip)?;

    // Footnotes / endnotes: pre-render bodies keyed by id. Inline runs
    // inside the body lose formatting (we flatten to plain markdown text).
    let footnotes = load_notes(&mut zip, "word/footnotes.xml", "footnote")?;
    let endnotes = load_notes(&mut zip, "word/endnotes.xml", "endnote")?;

    let body = read_zip_text(&mut zip, "word/document.xml")?
        .ok_or_else(|| ConvertError::Zip("missing word/document.xml".into()))?;

    let ctx = Ctx {
        rels,
        styles,
        numbering,
        image_rels,
        comment_authors,
        comment_texts,
        footnotes,
        endnotes,
    };
    let mut md = walk_document(&body, &ctx)?;

    // Headers / footers: emit at the very top / bottom of the document
    // wrapped in HTML comments so they don't pollute the main heading
    // outline. Each part is rendered as plain paragraphs separated by
    // newlines.
    let (header_md, footer_md) = render_headers_footers(&mut zip)?;
    if !header_md.is_empty() {
        md = format!("<!-- header -->\n{header_md}\n<!-- /header -->\n\n{md}");
    }
    if !footer_md.is_empty() {
        md.push_str(&format!("\n<!-- footer -->\n{footer_md}\n<!-- /footer -->\n"));
    }

    // Footnote / endnote definitions tail.
    let mut tail = String::new();
    let used_fn = footnote_ids_in(&md, "fn");
    let used_en = footnote_ids_in(&md, "en");
    for id in &used_fn {
        if let Some(body) = ctx.footnotes.get(id) {
            tail.push_str(&format!("[^fn{id}]: {body}\n"));
        }
    }
    for id in &used_en {
        if let Some(body) = ctx.endnotes.get(id) {
            tail.push_str(&format!("[^en{id}]: {body}\n"));
        }
    }
    if !tail.is_empty() {
        md.push_str("\n");
        md.push_str(&tail);
    }

    Ok(DocxResult {
        markdown: post_process(md),
        assets,
    })
}
