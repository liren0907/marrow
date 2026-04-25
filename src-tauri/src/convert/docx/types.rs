use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ListKind {
    Bullet,
    Ordered,
}

/// Per-level format inside an abstract numbering definition. Indexed by
/// `(numId, ilvl)` after `numId → abstractNumId` resolution.
pub(super) type NumberingMap = HashMap<(String, usize), ListKind>;

pub(super) struct Ctx {
    pub(super) rels: HashMap<String, String>,
    pub(super) styles: HashMap<String, usize>, // style_id → heading level (1..=6)
    pub(super) numbering: NumberingMap,
    /// rId → final attachment filename (e.g. `image1.png`). Populated up
    /// front so the walker can emit `![alt](attachments/...)` without
    /// touching the zip a second time.
    pub(super) image_rels: HashMap<String, String>,
    /// id → "Author Name". Empty when there are no comments.
    pub(super) comment_authors: HashMap<String, String>,
    /// id → flattened comment text (single line, paragraph breaks → spaces).
    pub(super) comment_texts: HashMap<String, String>,
    /// id → footnote markdown body (already rendered, paragraph break → space).
    pub(super) footnotes: HashMap<String, String>,
    /// id → endnote markdown body.
    pub(super) endnotes: HashMap<String, String>,
}

#[derive(Default, Clone)]
pub(super) struct Run {
    pub(super) text: String,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) strike: bool,
}

#[derive(Default)]
pub(super) struct Paragraph {
    pub(super) style_id: Option<String>,
    pub(super) num_id: Option<String>,
    pub(super) ilvl: usize,
    pub(super) runs: Vec<Run>,
    /// Comments whose range ends at (or covers) this paragraph. Rendered
    /// as `> 💬 **Author**: text` blockquotes after the paragraph body.
    pub(super) comment_ids: Vec<String>,
}
