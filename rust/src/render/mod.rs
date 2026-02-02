//! Rendering layer for converting resolved documents to output formats.

pub mod html;
pub mod math;

#[cfg(feature = "pdf")]
pub mod pdf;

pub use html::{render_html, HtmlConfig};
pub use math::{MathBackend, MathRenderer};

#[cfg(feature = "pdf")]
pub use pdf::{render_pdf, render_pdf_to_file, PageMargins, PaperSize, PdfConfig};

use crate::ast::ResolvedDocument;
use crate::error::Result;

/// Render a resolved document to HTML.
pub fn render(document: &ResolvedDocument, config: &HtmlConfig) -> Result<String> {
    render_html(document, config)
}
