//! Native document → Markdown converters used by the Marrow desktop app.
//!
//! Public surface:
//!
//! - [`html::html_to_markdown`] — chardetng + dom_smoothie + htmd pipeline
//! - [`docx::docx_to_markdown`] — native OOXML walker (zip + quick-xml)
//! - [`pptx::pptx_to_markdown`] — native OOXML walker (zip + quick-xml)
//! - [`ConvertError`] — unified error type returned by every converter
//!
//! The [`ooxml`] module hosts shared OOXML helpers (zip / rels / DML text
//! parser) consumed by both `docx` and `pptx`. It is internal to the
//! crate.

pub mod docx;
pub mod html;
pub mod pptx;

mod error;
pub(crate) mod ooxml;

pub use error::ConvertError;
