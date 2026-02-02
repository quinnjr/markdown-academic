//! KaTeX/MathJax passthrough renderer.

use super::MathRenderer;
use crate::error::Result;

/// Renderer that outputs raw LaTeX for client-side rendering.
pub struct KaTeXRenderer {
    use_mathjax: bool,
}

impl KaTeXRenderer {
    /// Create a new KaTeX renderer.
    pub fn new() -> Self {
        Self { use_mathjax: false }
    }

    /// Create a renderer configured for MathJax.
    pub fn new_mathjax() -> Self {
        Self { use_mathjax: true }
    }
}

impl Default for KaTeXRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MathRenderer for KaTeXRenderer {
    fn render_inline(&self, latex: &str) -> Result<String> {
        // Escape HTML entities in the LaTeX
        let escaped = escape_html(latex);

        if self.use_mathjax {
            Ok(format!(r#"<span class="math inline">\({}\)</span>"#, escaped))
        } else {
            Ok(format!(r#"<span class="math inline">\({}\)</span>"#, escaped))
        }
    }

    fn render_display(&self, latex: &str) -> Result<String> {
        let escaped = escape_html(latex);

        if self.use_mathjax {
            Ok(format!(
                r#"<div class="math display">\[{}\]</div>"#,
                escaped
            ))
        } else {
            Ok(format!(
                r#"<div class="math display">\[{}\]</div>"#,
                escaped
            ))
        }
    }

    fn head_content(&self) -> Option<String> {
        if self.use_mathjax {
            Some(MATHJAX_HEAD.to_string())
        } else {
            Some(KATEX_HEAD.to_string())
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const KATEX_HEAD: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" crossorigin="anonymous">
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js" crossorigin="anonymous"></script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js" crossorigin="anonymous"
    onload="renderMathInElement(document.body, {
        delimiters: [
            {left: '\\[', right: '\\]', display: true},
            {left: '\\(', right: '\\)', display: false}
        ]
    });"></script>"#;

const MATHJAX_HEAD: &str = r#"<script>
MathJax = {
    tex: {
        inlineMath: [['\\(', '\\)']],
        displayMath: [['\\[', '\\]']]
    }
};
</script>
<script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let renderer = KaTeXRenderer::new();
        let result = renderer.render_inline("E = mc^2").unwrap();
        assert!(result.contains("E = mc^2"));
        assert!(result.contains("math inline"));
    }

    #[test]
    fn test_display_math() {
        let renderer = KaTeXRenderer::new();
        let result = renderer.render_display("\\int_0^1 x dx").unwrap();
        assert!(result.contains("math display"));
    }

    #[test]
    fn test_escaping() {
        let renderer = KaTeXRenderer::new();
        let result = renderer.render_inline("a < b").unwrap();
        assert!(result.contains("&lt;"));
    }
}
