use thiserror::Error;

#[derive(Error, Debug)]
pub enum LyricsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("XML parse error: {0}")]
    Xml(String),

    #[error("Unsupported format")]
    UnsupportedFormat,
}

pub type Result<T> = std::result::Result<T, LyricsError>;
