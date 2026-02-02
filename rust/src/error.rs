//! Error types for the markdown-latex library.

use thiserror::Error;

/// Result type alias for this library.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the library.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Resolution error: {0}")]
    Resolution(#[from] ResolutionError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid front matter: {0}")]
    FrontMatter(String),

    #[error("Invalid BibTeX: {0}")]
    BibTeX(String),

    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Parse error: {0}")]
    Other(String),
}

/// Errors that occur during resolution.
#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("Unknown citation key: {0}")]
    UnknownCitation(String),

    #[error("Unknown reference label: {0}")]
    UnknownReference(String),

    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),

    #[error("Undefined footnote: {0}")]
    UndefinedFootnote(String),

    #[error("Circular macro reference: {0}")]
    CircularMacro(String),

    #[error("Failed to read bibliography file: {0}")]
    BibliographyRead(String),
}

/// Errors that occur during rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Math rendering error: {0}")]
    Math(String),

    #[error("Template error: {0}")]
    Template(String),
}
