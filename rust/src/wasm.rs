//! WebAssembly bindings for JavaScript/TypeScript.
//!
//! This module provides WASM bindings that work in both Node.js and browser environments.
//!
//! # Usage (JavaScript/TypeScript)
//!
//! ```javascript
//! import init, { renderMarkdown, parseDocument, RenderOptions } from '@markdown-academic/wasm';
//!
//! // Initialize the WASM module (required before any other calls)
//! await init();
//!
//! // Simple rendering
//! const html = renderMarkdown('# Hello $E=mc^2$');
//!
//! // With options
//! const options = new RenderOptions();
//! options.setStandalone(true);
//! options.setMathBackend('katex');
//! const fullHtml = renderMarkdown(source, options);
//!
//! // Parse and get document info
//! const doc = parseDocument(source);
//! console.log(doc.metadata.title);
//! ```

#![cfg(feature = "wasm")]

use crate::ast::{Block, Document, EnvironmentKind, Inline};
use crate::parser::parse;
use crate::render::{render_html, HtmlConfig, MathBackend};
use crate::resolve::{resolve, ResolveConfig};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Main API Functions
// ============================================================================

/// Parse and render Markdown to HTML.
///
/// This is the primary function for converting markdown-academic source to HTML.
///
/// # Arguments
///
/// * `input` - The Markdown source text.
/// * `options` - Optional configuration object.
///
/// # Returns
///
/// The rendered HTML string.
///
/// # Errors
///
/// Returns an error if parsing or rendering fails.
#[wasm_bindgen(js_name = renderMarkdown)]
pub fn render_markdown(input: &str, options: Option<RenderOptions>) -> Result<String, JsError> {
    let doc = parse(input).map_err(|e| JsError::new(&format!("Parse error: {}", e)))?;

    let resolve_config = ResolveConfig::default();
    let resolved =
        resolve(doc, &resolve_config).map_err(|e| JsError::new(&format!("Resolution error: {}", e)))?;

    let html_config = options.map(|o| o.to_html_config()).unwrap_or_default();

    render_html(&resolved, &html_config).map_err(|e| JsError::new(&format!("Render error: {}", e)))
}

/// Parse a Markdown document and return structured information.
///
/// Returns a JavaScript object with the document's metadata and structure.
///
/// # Arguments
///
/// * `input` - The Markdown source text.
///
/// # Returns
///
/// A JavaScript object containing document metadata and block information.
#[wasm_bindgen(js_name = parseDocument)]
pub fn parse_document(input: &str) -> Result<JsValue, JsError> {
    let doc = parse(input).map_err(|e| JsError::new(&format!("Parse error: {}", e)))?;

    let resolve_config = ResolveConfig::default();
    let resolved =
        resolve(doc, &resolve_config).map_err(|e| JsError::new(&format!("Resolution error: {}", e)))?;

    let info = DocumentInfo::from_resolved(&resolved.document);

    serde_wasm_bindgen::to_value(&info).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Parse Markdown and return the full AST as JSON.
///
/// Useful for debugging or implementing custom renderers.
///
/// # Arguments
///
/// * `input` - The Markdown source text.
///
/// # Returns
///
/// JSON string representation of the parsed document.
#[wasm_bindgen(js_name = parseToJson)]
pub fn parse_to_json(input: &str) -> Result<String, JsError> {
    let doc = parse(input).map_err(|e| JsError::new(&format!("Parse error: {}", e)))?;

    let resolve_config = ResolveConfig::default();
    let resolved =
        resolve(doc, &resolve_config).map_err(|e| JsError::new(&format!("Resolution error: {}", e)))?;

    let info = DocumentInfo::from_resolved(&resolved.document);

    serde_json::to_string_pretty(&info).map_err(|e| JsError::new(&format!("JSON error: {}", e)))
}

/// Validate a Markdown document without rendering.
///
/// Checks for syntax errors, unresolved references, and other issues.
///
/// # Arguments
///
/// * `input` - The Markdown source text.
///
/// # Returns
///
/// A validation result object.
#[wasm_bindgen(js_name = validateDocument)]
pub fn validate_document(input: &str) -> Result<JsValue, JsError> {
    let mut result = ValidationResult {
        valid: true,
        errors: vec![],
        warnings: vec![],
    };

    // Try to parse
    let doc = match parse(input) {
        Ok(d) => d,
        Err(e) => {
            result.valid = false;
            result.errors.push(format!("Parse error: {}", e));
            return serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsError::new(&format!("Serialization error: {}", e)));
        }
    };

    // Try to resolve
    let resolve_config = ResolveConfig::default();
    if let Err(e) = resolve(doc, &resolve_config) {
        result.valid = false;
        result.errors.push(format!("Resolution error: {}", e));
    }

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Get the library version.
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if a feature is supported.
#[wasm_bindgen(js_name = hasFeature)]
pub fn has_feature(feature: &str) -> bool {
    match feature {
        "math" => true,
        "citations" => true,
        "crossref" => true,
        "environments" => true,
        "footnotes" => true,
        "toc" => true,
        "mathml" => cfg!(feature = "mathml"),
        _ => false,
    }
}

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration options for rendering.
#[wasm_bindgen]
#[derive(Clone)]
pub struct RenderOptions {
    math_backend: String,
    standalone: bool,
    title: Option<String>,
    custom_css: Option<String>,
    include_toc: bool,
    class_prefix: String,
    strict_mode: bool,
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
            include_toc: true,
            class_prefix: "mda".to_string(),
            strict_mode: false,
        }
    }

    /// Set the math rendering backend: "katex", "mathjax", or "mathml".
    #[wasm_bindgen(js_name = setMathBackend)]
    pub fn set_math_backend(&mut self, backend: &str) {
        self.math_backend = backend.to_lowercase();
    }

    /// Get the current math backend.
    #[wasm_bindgen(js_name = getMathBackend)]
    pub fn get_math_backend(&self) -> String {
        self.math_backend.clone()
    }

    /// Set whether to generate a complete HTML document.
    #[wasm_bindgen(js_name = setStandalone)]
    pub fn set_standalone(&mut self, standalone: bool) {
        self.standalone = standalone;
    }

    /// Get standalone setting.
    #[wasm_bindgen(js_name = getStandalone)]
    pub fn get_standalone(&self) -> bool {
        self.standalone
    }

    /// Set the document title (for standalone mode).
    #[wasm_bindgen(js_name = setTitle)]
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    /// Get the document title.
    #[wasm_bindgen(js_name = getTitle)]
    pub fn get_title(&self) -> Option<String> {
        self.title.clone()
    }

    /// Set custom CSS to include.
    #[wasm_bindgen(js_name = setCustomCss)]
    pub fn set_custom_css(&mut self, css: &str) {
        self.custom_css = Some(css.to_string());
    }

    /// Set whether to include table of contents.
    #[wasm_bindgen(js_name = setIncludeToc)]
    pub fn set_include_toc(&mut self, include: bool) {
        self.include_toc = include;
    }

    /// Set the CSS class prefix.
    #[wasm_bindgen(js_name = setClassPrefix)]
    pub fn set_class_prefix(&mut self, prefix: &str) {
        self.class_prefix = prefix.to_string();
    }

    /// Enable or disable strict mode (errors on unresolved refs).
    #[wasm_bindgen(js_name = setStrictMode)]
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
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
            include_toc: self.include_toc,
            class_prefix: self.class_prefix.clone(),
        }
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Serializable Types for JS Interop
// ============================================================================

#[derive(Serialize, Deserialize)]
struct DocumentInfo {
    metadata: MetadataInfo,
    blocks: Vec<BlockInfo>,
    labels: Vec<LabelInfo>,
    statistics: DocumentStats,
}

#[derive(Serialize, Deserialize)]
struct MetadataInfo {
    title: Option<String>,
    subtitle: Option<String>,
    authors: Vec<String>,
    date: Option<String>,
    keywords: Vec<String>,
    institution: Option<String>,
    macros: Vec<String>,
    bibliography_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BlockInfo {
    #[serde(rename = "type")]
    block_type: String,
    label: Option<String>,
    level: Option<u8>,
    content_preview: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct LabelInfo {
    label: String,
    #[serde(rename = "type")]
    label_type: String,
}

#[derive(Serialize, Deserialize)]
struct DocumentStats {
    block_count: usize,
    heading_count: usize,
    equation_count: usize,
    citation_count: usize,
    figure_count: usize,
    table_count: usize,
    footnote_count: usize,
    word_count: usize,
}

#[derive(Serialize, Deserialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl DocumentInfo {
    fn from_resolved(doc: &Document) -> Self {
        let metadata = MetadataInfo {
            title: doc.metadata.title.clone(),
            subtitle: doc.metadata.subtitle.clone(),
            authors: doc.metadata.authors.clone(),
            date: doc.metadata.date.clone(),
            keywords: doc.metadata.keywords.clone(),
            institution: doc.metadata.institution.clone(),
            macros: doc.metadata.macros.keys().cloned().collect(),
            bibliography_path: doc.metadata.bibliography_path.clone(),
        };

        let mut blocks = Vec::new();
        let mut labels = Vec::new();
        let mut stats = DocumentStats {
            block_count: 0,
            heading_count: 0,
            equation_count: 0,
            citation_count: 0,
            figure_count: 0,
            table_count: 0,
            footnote_count: 0,
            word_count: 0,
        };

        Self::collect_blocks(&doc.blocks, &mut blocks, &mut labels, &mut stats);

        Self {
            metadata,
            blocks,
            labels,
            statistics: stats,
        }
    }

    fn collect_blocks(
        doc_blocks: &[Block],
        blocks: &mut Vec<BlockInfo>,
        labels: &mut Vec<LabelInfo>,
        stats: &mut DocumentStats,
    ) {
        for block in doc_blocks {
            stats.block_count += 1;

            match block {
                Block::Heading {
                    level,
                    label,
                    content,
                    ..
                } => {
                    stats.heading_count += 1;
                    let preview = Self::inline_preview(content);
                    stats.word_count += Self::count_words(&preview);
                    blocks.push(BlockInfo {
                        block_type: "heading".to_string(),
                        label: label.clone(),
                        level: Some(*level),
                        content_preview: Some(preview),
                    });
                    if let Some(l) = label {
                        labels.push(LabelInfo {
                            label: l.clone(),
                            label_type: "section".to_string(),
                        });
                    }
                }
                Block::Paragraph(inlines) => {
                    let preview = Self::inline_preview(inlines);
                    stats.word_count += Self::count_words(&preview);
                    Self::count_inline_elements(inlines, stats);
                    blocks.push(BlockInfo {
                        block_type: "paragraph".to_string(),
                        label: None,
                        level: None,
                        content_preview: Some(Self::truncate(&preview, 100)),
                    });
                }
                Block::DisplayMath { label, .. } => {
                    stats.equation_count += 1;
                    blocks.push(BlockInfo {
                        block_type: "equation".to_string(),
                        label: label.clone(),
                        level: None,
                        content_preview: None,
                    });
                    if let Some(l) = label {
                        labels.push(LabelInfo {
                            label: l.clone(),
                            label_type: "equation".to_string(),
                        });
                    }
                }
                Block::Environment {
                    kind, label, content, ..
                } => {
                    let type_name = match kind {
                        EnvironmentKind::Theorem => "theorem",
                        EnvironmentKind::Lemma => "lemma",
                        EnvironmentKind::Proposition => "proposition",
                        EnvironmentKind::Corollary => "corollary",
                        EnvironmentKind::Definition => "definition",
                        EnvironmentKind::Example => "example",
                        EnvironmentKind::Remark => "remark",
                        EnvironmentKind::Proof => "proof",
                        EnvironmentKind::Figure => {
                            stats.figure_count += 1;
                            "figure"
                        }
                        EnvironmentKind::Table => {
                            stats.table_count += 1;
                            "table"
                        }
                        EnvironmentKind::Algorithm => "algorithm",
                        EnvironmentKind::Abstract => "abstract",
                        EnvironmentKind::Note => "note",
                        EnvironmentKind::Warning => "warning",
                        EnvironmentKind::Quote => "quote",
                        EnvironmentKind::Conjecture => "conjecture",
                        EnvironmentKind::Axiom => "axiom",
                        EnvironmentKind::Exercise => "exercise",
                        EnvironmentKind::Solution => "solution",
                        EnvironmentKind::Case => "case",
                        EnvironmentKind::Custom(name) => {
                            // Return the custom name, but we need to handle lifetimes
                            // For now, just return "custom"
                            let _ = name;
                            "custom"
                        }
                    };
                    blocks.push(BlockInfo {
                        block_type: format!("environment:{}", type_name),
                        label: label.clone(),
                        level: None,
                        content_preview: None,
                    });
                    if let Some(l) = label {
                        labels.push(LabelInfo {
                            label: l.clone(),
                            label_type: type_name.to_string(),
                        });
                    }
                    Self::collect_blocks(content, blocks, labels, stats);
                }
                Block::Table { label, .. } => {
                    stats.table_count += 1;
                    blocks.push(BlockInfo {
                        block_type: "table".to_string(),
                        label: label.clone(),
                        level: None,
                        content_preview: None,
                    });
                    if let Some(l) = label {
                        labels.push(LabelInfo {
                            label: l.clone(),
                            label_type: "table".to_string(),
                        });
                    }
                }
                Block::CodeBlock { language, .. } => {
                    blocks.push(BlockInfo {
                        block_type: "codeblock".to_string(),
                        label: None,
                        level: None,
                        content_preview: language.clone(),
                    });
                }
                Block::BlockQuote(inner) => {
                    blocks.push(BlockInfo {
                        block_type: "blockquote".to_string(),
                        label: None,
                        level: None,
                        content_preview: None,
                    });
                    Self::collect_blocks(inner, blocks, labels, stats);
                }
                Block::List { items, .. } => {
                    blocks.push(BlockInfo {
                        block_type: "list".to_string(),
                        label: None,
                        level: None,
                        content_preview: Some(format!("{} items", items.len())),
                    });
                }
                Block::TableOfContents => {
                    blocks.push(BlockInfo {
                        block_type: "toc".to_string(),
                        label: None,
                        level: None,
                        content_preview: None,
                    });
                }
                Block::ThematicBreak => {
                    blocks.push(BlockInfo {
                        block_type: "hr".to_string(),
                        label: None,
                        level: None,
                        content_preview: None,
                    });
                }
                Block::RawHtml(_) => {
                    blocks.push(BlockInfo {
                        block_type: "html".to_string(),
                        label: None,
                        level: None,
                        content_preview: None,
                    });
                }
                _ => {}
            }
        }
    }

    fn count_inline_elements(inlines: &[Inline], stats: &mut DocumentStats) {
        for inline in inlines {
            match inline {
                Inline::Citation(_) => stats.citation_count += 1,
                Inline::Footnote(_) => stats.footnote_count += 1,
                Inline::Emphasis(inner) | Inline::Strong(inner) => {
                    Self::count_inline_elements(inner, stats);
                }
                _ => {}
            }
        }
    }

    fn inline_preview(inlines: &[Inline]) -> String {
        let mut result = String::new();
        for inline in inlines {
            match inline {
                Inline::Text(s) => result.push_str(s),
                Inline::Code(s) => {
                    result.push('`');
                    result.push_str(s);
                    result.push('`');
                }
                Inline::Emphasis(inner) => {
                    result.push('*');
                    result.push_str(&Self::inline_preview(inner));
                    result.push('*');
                }
                Inline::Strong(inner) => {
                    result.push_str("**");
                    result.push_str(&Self::inline_preview(inner));
                    result.push_str("**");
                }
                Inline::InlineMath(s) => {
                    result.push('$');
                    result.push_str(s);
                    result.push('$');
                }
                Inline::SoftBreak | Inline::HardBreak => result.push(' '),
                _ => {}
            }
        }
        result
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }

    fn count_words(s: &str) -> usize {
        s.split_whitespace().count()
    }
}
