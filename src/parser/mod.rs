//! Parser for extended Markdown with LaTeX-style features.

mod block;
mod inline;
mod lexer;

pub use block::parse_blocks;
pub use inline::parse_inlines;

use crate::ast::{Document, Macro, Metadata};
use crate::error::{ParseError, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Parse a complete document from source text.
pub fn parse(input: &str) -> Result<Document> {
    let (metadata, content) = parse_front_matter(input)?;
    let blocks = parse_blocks(content)?;

    Ok(Document { metadata, blocks })
}

/// Parse TOML front matter delimited by `+++`.
fn parse_front_matter(input: &str) -> Result<(Metadata, &str)> {
    let trimmed = input.trim_start();

    if !trimmed.starts_with("+++") {
        return Ok((Metadata::default(), input));
    }

    let after_open = &trimmed[3..];
    let close_pos = after_open
        .find("\n+++")
        .ok_or_else(|| ParseError::FrontMatter("Unclosed front matter (missing closing +++)".into()))?;

    let front_matter_str = &after_open[..close_pos];
    let content_start = 3 + close_pos + 4; // "+++" + content + "\n+++"
    let content = trimmed[content_start..].trim_start_matches('\n');

    let raw: RawFrontMatter = toml::from_str(front_matter_str)
        .map_err(|e| ParseError::FrontMatter(format!("Invalid TOML: {}", e)))?;

    let metadata = convert_front_matter(raw);

    Ok((metadata, content))
}

/// Raw front matter structure for deserialization.
#[derive(Debug, Deserialize, Default)]
struct RawFrontMatter {
    title: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    author: Option<String>,
    date: Option<String>,
    #[serde(default)]
    macros: HashMap<String, String>,
    bibliography: Option<BibliographyConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BibliographyConfig {
    Path(String),
    Config { path: String },
}

/// Convert raw front matter to metadata.
fn convert_front_matter(raw: RawFrontMatter) -> Metadata {
    let macros = raw
        .macros
        .into_iter()
        .map(|(name, template)| {
            let arg_count = count_macro_args(&template);
            (name, Macro { arg_count, template })
        })
        .collect();

    let mut authors = raw.authors;
    if let Some(author) = raw.author {
        if authors.is_empty() {
            authors.push(author);
        }
    }

    let bibliography_path = raw.bibliography.map(|b| match b {
        BibliographyConfig::Path(p) => p,
        BibliographyConfig::Config { path } => path,
    });

    Metadata {
        macros,
        bibliography_path,
        title: raw.title,
        authors,
        date: raw.date,
    }
}

/// Count the number of macro arguments (#1, #2, etc.) in a template.
fn count_macro_args(template: &str) -> usize {
    let mut max_arg = 0;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '#' {
            if let Some(&digit) = chars.peek() {
                if let Some(n) = digit.to_digit(10) {
                    max_arg = max_arg.max(n as usize);
                }
            }
        }
    }

    max_arg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_front_matter() {
        let input = "# Hello\n\nSome text.";
        let (meta, content) = parse_front_matter(input).unwrap();
        assert!(meta.macros.is_empty());
        assert_eq!(content, input);
    }

    #[test]
    fn test_with_front_matter() {
        let input = r#"+++
title = "My Document"
author = "Jane Doe"

[macros]
R = "\\mathbb{R}"
vec = "\\mathbf{#1}"
+++

# Hello

Some text."#;

        let (meta, content) = parse_front_matter(input).unwrap();
        assert_eq!(meta.title, Some("My Document".to_string()));
        assert_eq!(meta.authors, vec!["Jane Doe".to_string()]);
        assert_eq!(meta.macros.get("R").unwrap().template, "\\mathbb{R}");
        assert_eq!(meta.macros.get("R").unwrap().arg_count, 0);
        assert_eq!(meta.macros.get("vec").unwrap().arg_count, 1);
        assert!(content.starts_with("# Hello"));
    }

    #[test]
    fn test_count_macro_args() {
        assert_eq!(count_macro_args("\\mathbb{R}"), 0);
        assert_eq!(count_macro_args("\\mathbf{#1}"), 1);
        assert_eq!(count_macro_args("\\frac{#1}{#2}"), 2);
        assert_eq!(count_macro_args("#1 + #2 + #3"), 3);
    }
}
