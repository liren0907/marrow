//! Shared OOXML helpers consumed by both `docx` and `pptx`.
//!
//! - [`util`] — zip / rels / path / markdown plumbing
//! - [`dml`] — DrawingML text-body parser (`<a:p>` / `<a:r>` grammar)

pub(crate) mod dml;
pub(crate) mod util;
