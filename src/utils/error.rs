use thiserror::Error;

#[derive(Error, Debug)]
pub enum PandorustError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported input format: {0}")]
    UnsupportedInputFormat(String),

    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),

    #[error("YAML front matter parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("DOCX generation error: {0}")]
    DocxError(String),
}

pub type Result<T> = std::result::Result<T, PandorustError>;
