use std::collections::HashMap;

use crate::ooxml::dml::{BulletKind, Paragraph};

/// Which `<p:txStyles>` block in the slide master a shape's text inherits
/// its default bullet styling from.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum StyleCategory {
    Title,
    Body,
    Other,
}

pub(super) type MasterStyles = HashMap<(StyleCategory, usize), BulletKind>;

pub(super) enum ShapeKind {
    /// Title or centerTitle placeholder — emit as heading.
    Title(String),
    /// Body text: ordered list of paragraphs. `category` selects which
    /// master `<p:txStyles>` block we fall back to for bullet defaults.
    Body {
        category: StyleCategory,
        paras: Vec<Paragraph>,
    },
    /// Table: pre-rendered cell strings.
    Table(Vec<Vec<String>>),
    /// Picture: `rel_id` references a `ppt/media/imageN.*` entry via the
    /// slide's rels. Resolved to a sidecar asset by the orchestrator.
    Picture { rel_id: String, alt: String },
}

pub(super) struct Shape {
    pub(super) y: i64,
    pub(super) x: i64,
    pub(super) kind: ShapeKind,
}

pub(super) struct Slide {
    /// `<p:sld show="0">` — slide marked hidden in the deck.
    pub(super) hidden: bool,
    pub(super) shapes: Vec<Shape>,
}
