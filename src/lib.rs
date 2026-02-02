//! # markdown-academic
//!
//! A Markdown parser and renderer with academic writing features: math, citations, cross-references, and more.
//!
//! ## File Extension
//!
//! The recommended file extension for markdown-academic documents is `.mda`.
//!
//! ## Features
//!
//! - **Math rendering**: Inline `$...$` and display `$$...$$` equations with configurable backends
//! - **Citations**: `[@key]` syntax with BibTeX bibliography support
//! - **Cross-references**: Label definitions `{#label}` and references `@label`
//! - **Environments**: Theorem, lemma, proof, figure, and custom environments using `:::` fences
//! - **Table of contents**: Auto-generated from headings with `[[toc]]`
//! - **Footnotes**: Inline `^[text]` and reference `[^id]` style footnotes
//! - **Custom macros**: User-defined LaTeX commands via TOML front matter
//! - **Automatic numbering**: Sections, equations, theorems, figures, and tables
//!
//! ## Quick Start
//!
//! ```rust
//! use markdown_academic::{parse, resolve, render_html, ResolveConfig, HtmlConfig};
//!
//! let input = r#"
//! # Introduction {#sec:intro}
//!
//! The equation $E = mc^2$ is famous. See @sec:intro for more.
//!
//! ::: theorem {#thm:main}
//! Every natural number is interesting.
//! :::
//! "#;
//!
//! // Parse the document
//! let doc = parse(input).unwrap();
//!
//! // Resolve references and citations
//! let resolved = resolve(doc, &ResolveConfig::default()).unwrap();
//!
//! // Render to HTML
//! let html = render_html(&resolved, &HtmlConfig::default()).unwrap();
//! println!("{}", html);
//! ```
//!
//! ## Syntax Reference
//!
//! ### Front Matter (TOML)
//!
//! ```text
//! +++
//! title = "My Document"
//! author = "Jane Doe"
//!
//! [macros]
//! R = "\\mathbb{R}"
//! vec = "\\mathbf{#1}"
//!
//! [bibliography]
//! path = "refs.bib"
//! +++
//! ```
//!
//! ### Math
//!
//! - Inline: `$E = mc^2$`
//! - Display: `$$\int_0^1 x dx$$ {#eq:integral}`
//!
//! ### Citations
//!
//! - Single: `[@knuth1984]`
//! - Multiple: `[@knuth1984; @lamport1994]`
//! - With locator: `[@knuth1984, p. 42]`
//!
//! ### Cross-References
//!
//! - Define label: `# Section {#sec:intro}` or `$$ ... $$ {#eq:euler}`
//! - Reference: `@sec:intro`, `@eq:euler`, `@thm:main`
//!
//! ### Environments
//!
//! ```text
//! ::: theorem {#thm:main}
//! Statement of the theorem.
//! :::
//!
//! ::: proof
//! The proof follows...
//! :::
//! ```
//!
//! Supported environments: theorem, lemma, proposition, corollary, definition,
//! example, remark, proof, figure, table, algorithm.
//!
//! ### Table of Contents
//!
//! Place `[[toc]]` where you want the table of contents to appear.
//!
//! ### Footnotes
//!
//! - Inline: `Some text^[This is a footnote].`
//! - Reference: `Some text[^1].` with `[^1]: Footnote content.` defined later.
//!
//! ## Configuration
//!
//! ### Math Backends
//!
//! - `KaTeX` (default): Client-side rendering with KaTeX
//! - `MathJax`: Client-side rendering with MathJax
//! - `MathML`: Native browser rendering (requires `mathml` feature)
//!
//! ### HTML Output
//!
//! - Fragment mode (default): Just the content, no `<html>` wrapper
//! - Standalone mode: Complete HTML document with styles and scripts
//!
//! ## FFI
//!
//! The library provides a C-compatible FFI for use from Python, JavaScript (via WASM),
//! and other languages. See the `ffi` module documentation for details.
//!
//! ## Features
//!
//! - `mathml`: Enable MathML rendering backend (requires `latex2mathml` crate)
//! - `wasm`: Enable WebAssembly bindings (requires `wasm-bindgen`)

// Re-export main types and functions for public API
pub mod ast;
pub mod bibtex;
pub mod error;
pub mod parser;
pub mod render;
pub mod resolve;

// FFI module (always compiled for cdylib)
pub mod ffi;

// WASM module (only with feature)
#[cfg(feature = "wasm")]
pub mod wasm;

// Convenience re-exports
pub use ast::{Block, Document, Inline, ResolvedDocument};
pub use error::{Error, ParseError, RenderError, ResolutionError, Result};
pub use parser::parse;
pub use render::{render_html, HtmlConfig, MathBackend};
pub use resolve::{resolve, ResolveConfig};

/// Parse, resolve, and render Markdown to HTML in one step.
///
/// This is a convenience function that combines `parse`, `resolve`, and `render_html`.
///
/// # Example
///
/// ```rust
/// use markdown_academic::render;
///
/// let html = render("# Hello *world*", None, None).unwrap();
/// assert!(html.contains("<h1>"));
/// ```
pub fn render(
    input: &str,
    resolve_config: Option<&ResolveConfig>,
    html_config: Option<&HtmlConfig>,
) -> Result<String> {
    let doc = parse(input)?;
    let resolved = resolve(doc, resolve_config.unwrap_or(&ResolveConfig::default()))?;
    render_html(&resolved, html_config.unwrap_or(&HtmlConfig::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        let input = r#"+++
title = "Test Document"

[macros]
R = "\\mathbb{R}"
+++

# Introduction {#sec:intro}

Let $x \in \R$ be a real number. See @sec:intro.

::: theorem {#thm:main}
All numbers are interesting.
:::

As shown in @thm:main, this is true.
"#;

        let html = render(input, None, None).unwrap();

        assert!(html.contains("<h1"));
        assert!(html.contains("Introduction"));
        assert!(html.contains("math inline"));
        assert!(html.contains("theorem"));
    }

    #[test]
    fn test_simple_markdown() {
        let input = "# Hello\n\n**Bold** and *italic* text.";
        let html = render(input, None, None).unwrap();

        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>Bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let html = render(input, None, None).unwrap();

        assert!(html.contains("<pre><code"));
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn test_list() {
        let input = "- Item 1\n- Item 2\n- Item 3";
        let html = render(input, None, None).unwrap();

        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }

    #[test]
    fn test_display_math() {
        let input = "$$\n\\int_0^1 x dx = \\frac{1}{2}\n$$";
        let html = render(input, None, None).unwrap();

        assert!(html.contains("math display"));
    }

    #[test]
    fn test_environment() {
        let input = "::: definition\nA *group* is a set with an operation.\n:::";
        let html = render(input, None, None).unwrap();

        assert!(html.contains("definition"));
        assert!(html.contains("Definition"));
    }

    #[test]
    fn test_table() {
        let input = r#"
| Header 1 | Header 2 |
| -------- | -------- |
| Cell 1   | Cell 2   |
"#;
        let html = render(input, None, None).unwrap();

        assert!(html.contains("<table"));
        assert!(html.contains("<th>"));
        assert!(html.contains("<td>"));
    }
}
