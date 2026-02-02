//! Resolution layer for linking references, citations, and expanding macros.

pub mod citations;
mod macros;
pub mod numbering;
pub mod references;

pub use citations::resolve_citations;
pub use macros::expand_macros;
pub use numbering::assign_numbers;
pub use references::resolve_references;

use crate::ast::{BibEntry, Document, LabelInfo, ResolvedDocument};
use crate::bibtex::parse_bibtex;
use crate::error::{ResolutionError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for resolution.
#[derive(Debug, Clone, Default)]
pub struct ResolveConfig {
    /// Base path for resolving relative bibliography paths.
    pub base_path: Option<String>,
    /// Whether to error on unknown citations (default: false, just warn).
    pub strict_citations: bool,
    /// Whether to error on unknown references (default: false).
    pub strict_references: bool,
}

/// Resolve all references, citations, and macros in a document.
pub fn resolve(document: Document, config: &ResolveConfig) -> Result<ResolvedDocument> {
    let mut doc = document;

    // Step 1: Load bibliography if specified
    let citations = if let Some(ref bib_path) = doc.metadata.bibliography_path {
        load_bibliography(bib_path, config)?
    } else {
        HashMap::new()
    };

    // Step 2: Expand macros in math content
    doc = expand_macros(doc)?;

    // Step 3: Assign numbers to sections, environments, equations, etc.
    let (section_numbers, env_numbers) = assign_numbers(&doc);

    // Step 4: Build label registry
    let labels = references::build_label_registry(&doc, &section_numbers, &env_numbers)?;

    // Step 5: Collect footnote definitions
    let footnotes = references::collect_footnotes(&doc)?;

    // Step 6: Resolve references in document
    let doc = resolve_references(doc, &labels, config)?;

    // Step 7: Resolve citations
    let doc = resolve_citations(doc, &citations, config)?;

    Ok(ResolvedDocument {
        document: doc,
        labels,
        citations,
        footnotes,
        section_numbers,
        env_numbers,
    })
}

fn load_bibliography(path: &str, config: &ResolveConfig) -> Result<HashMap<String, BibEntry>> {
    let full_path = if let Some(ref base) = config.base_path {
        Path::new(base).join(path)
    } else {
        Path::new(path).to_path_buf()
    };

    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        ResolutionError::BibliographyRead(format!("{}: {}", full_path.display(), e))
    })?;

    Ok(parse_bibtex(&content).map_err(|e| ResolutionError::BibliographyRead(e.to_string()))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_resolve_simple_document() {
        let input = r#"
# Introduction {#sec:intro}

Some text with a reference to @sec:intro.
"#;

        let doc = parse(input).unwrap();
        let config = ResolveConfig::default();
        let resolved = resolve(doc, &config).unwrap();

        assert!(resolved.labels.contains_key("sec:intro"));
    }
}
