pub mod html;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("html parse: {0}")]
    Parse(String),
    #[error("readability extraction failed: {0}")]
    Extract(String),
    #[error("markdown serialization: {0}")]
    Serialize(String),
}
