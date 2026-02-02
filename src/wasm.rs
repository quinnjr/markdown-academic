//! WebAssembly bindings for JavaScript/TypeScript.

#![cfg(feature = "wasm")]

use crate::parser::parse;
use crate::render::{render_html, HtmlConfig, MathBackend};
use crate::resolve::{resolve, ResolveConfig};
use wasm_bindgen::prelude::*;

/// Parse and render Markdown to HTML.
///
/// # Arguments
///
/// * `input` - The Markdown source text.
/// * `options` - Optional configuration object.
///
/// # Returns
///
/// The rendered HTML string.
#[wasm_bindgen(js_name = renderMarkdown)]
pub fn render_markdown(input: &str, options: Option<RenderOptions>) -> Result<String, JsError> {
    let doc = parse(input).map_err(|e| JsError::new(&e.to_string()))?;

    let resolve_config = ResolveConfig::default();
    let resolved = resolve(doc, &resolve_config).map_err(|e| JsError::new(&e.to_string()))?;

    let html_config = options
        .map(|o| o.to_html_config())
        .unwrap_or_default();

    render_html(&resolved, &html_config).map_err(|e| JsError::new(&e.to_string()))
}

/// Configuration options for rendering.
#[wasm_bindgen]
pub struct RenderOptions {
    math_backend: String,
    standalone: bool,
    title: Option<String>,
    custom_css: Option<String>,
}

#[wasm_bindgen]
impl RenderOptions {
    /// Create a new options object with defaults.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            math_backend: "katex".to_string(),
            standalone: false,
            title: None,
            custom_css: None,
        }
    }

    /// Set the math rendering backend: "katex", "mathjax", or "mathml".
    #[wasm_bindgen(js_name = setMathBackend)]
    pub fn set_math_backend(&mut self, backend: &str) {
        self.math_backend = backend.to_lowercase();
    }

    /// Set whether to generate a complete HTML document.
    #[wasm_bindgen(js_name = setStandalone)]
    pub fn set_standalone(&mut self, standalone: bool) {
        self.standalone = standalone;
    }

    /// Set the document title (for standalone mode).
    #[wasm_bindgen(js_name = setTitle)]
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    /// Set custom CSS to include.
    #[wasm_bindgen(js_name = setCustomCss)]
    pub fn set_custom_css(&mut self, css: &str) {
        self.custom_css = Some(css.to_string());
    }

    fn to_html_config(&self) -> HtmlConfig {
        HtmlConfig {
            math_backend: match self.math_backend.as_str() {
                "mathjax" => MathBackend::MathJax,
                "mathml" => MathBackend::MathML,
                _ => MathBackend::KaTeX,
            },
            standalone: self.standalone,
            title: self.title.clone(),
            custom_css: self.custom_css.clone(),
            ..Default::default()
        }
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse Markdown and return the AST as JSON.
///
/// Useful for debugging or custom rendering.
#[wasm_bindgen(js_name = parseToJson)]
pub fn parse_to_json(input: &str) -> Result<String, JsError> {
    let doc = parse(input).map_err(|e| JsError::new(&e.to_string()))?;

    // Simple JSON serialization (not using serde_json to avoid extra dependency)
    let json = format!(
        r#"{{"blocks_count": {}, "has_metadata": {}}}"#,
        doc.blocks.len(),
        !doc.metadata.macros.is_empty() || doc.metadata.bibliography_path.is_some()
    );

    Ok(json)
}

/// Get the library version.
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// TypeScript type definitions for documentation
/// ```typescript
/// // markdown_academic.d.ts
/// 
/// /**
///  * Render Markdown to HTML.
///  * @param input - The Markdown source text
///  * @param options - Optional configuration
///  * @returns The rendered HTML string
///  */
/// export function renderMarkdown(input: string, options?: RenderOptions): string;
/// 
/// /**
///  * Parse Markdown and return AST info as JSON.
///  * @param input - The Markdown source text
///  * @returns JSON string with AST information
///  */
/// export function parseToJson(input: string): string;
/// 
/// /**
///  * Get the library version.
///  * @returns Version string
///  */
/// export function getVersion(): string;
/// 
/// /**
///  * Configuration options for rendering.
///  */
/// export class RenderOptions {
///     constructor();
///     setMathBackend(backend: "katex" | "mathjax" | "mathml"): void;
///     setStandalone(standalone: boolean): void;
///     setTitle(title: string): void;
///     setCustomCss(css: string): void;
/// }
/// ```
const _: () = ();
