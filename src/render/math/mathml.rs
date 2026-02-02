//! MathML renderer.

use super::MathRenderer;
use crate::error::Result;

/// Renderer that converts LaTeX to MathML.
pub struct MathMLRenderer {
    #[cfg(feature = "mathml")]
    _phantom: std::marker::PhantomData<()>,
}

impl MathMLRenderer {
    /// Create a new MathML renderer.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "mathml")]
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for MathMLRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MathRenderer for MathMLRenderer {
    fn render_inline(&self, latex: &str) -> Result<String> {
        #[cfg(feature = "mathml")]
        {
            match latex2mathml::latex_to_mathml(latex, latex2mathml::DisplayStyle::Inline) {
                Ok(mathml) => Ok(mathml),
                Err(_) => {
                    // Fallback to escaped LaTeX
                    Ok(format!(
                        r#"<span class="math inline math-error">{}</span>"#,
                        escape_html(latex)
                    ))
                }
            }
        }

        #[cfg(not(feature = "mathml"))]
        {
            // Without the mathml feature, fall back to escaped LaTeX
            Ok(format!(
                r#"<span class="math inline">\({}\)</span>"#,
                escape_html(latex)
            ))
        }
    }

    fn render_display(&self, latex: &str) -> Result<String> {
        #[cfg(feature = "mathml")]
        {
            match latex2mathml::latex_to_mathml(latex, latex2mathml::DisplayStyle::Block) {
                Ok(mathml) => Ok(format!(r#"<div class="math display">{}</div>"#, mathml)),
                Err(_) => {
                    // Fallback to escaped LaTeX
                    Ok(format!(
                        r#"<div class="math display math-error">{}</div>"#,
                        escape_html(latex)
                    ))
                }
            }
        }

        #[cfg(not(feature = "mathml"))]
        {
            Ok(format!(
                r#"<div class="math display">\[{}\]</div>"#,
                escape_html(latex)
            ))
        }
    }

    fn head_content(&self) -> Option<String> {
        // MathML doesn't require external scripts
        // But we might want some CSS for fallback styling
        Some(MATHML_STYLES.to_string())
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const MATHML_STYLES: &str = r#"<style>
.math-error {
    color: red;
    font-family: monospace;
}
math {
    font-size: 1.1em;
}
</style>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mathml_inline() {
        let renderer = MathMLRenderer::new();
        let result = renderer.render_inline("x^2").unwrap();
        // Should produce some output regardless of feature
        assert!(!result.is_empty());
    }
}
