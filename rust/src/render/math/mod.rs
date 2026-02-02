//! Math rendering backends.

mod katex;
mod mathml;

pub use self::katex::KaTeXRenderer;
pub use self::mathml::MathMLRenderer;

use crate::error::Result;

/// Math rendering backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MathBackend {
    /// Output raw LaTeX for KaTeX/MathJax to render client-side.
    #[default]
    KaTeX,
    /// Convert to MathML for native browser rendering.
    MathML,
    /// Output raw LaTeX for MathJax (same as KaTeX but different delimiters).
    MathJax,
}

/// Trait for math renderers.
pub trait MathRenderer {
    /// Render inline math.
    fn render_inline(&self, latex: &str) -> Result<String>;

    /// Render display math.
    fn render_display(&self, latex: &str) -> Result<String>;

    /// Get any required HTML head content (scripts, styles).
    fn head_content(&self) -> Option<String>;
}

/// Create a math renderer for the given backend.
pub fn create_renderer(backend: MathBackend) -> Box<dyn MathRenderer> {
    match backend {
        MathBackend::KaTeX => Box::new(KaTeXRenderer::new()),
        MathBackend::MathJax => Box::new(KaTeXRenderer::new_mathjax()),
        MathBackend::MathML => Box::new(MathMLRenderer::new()),
    }
}
