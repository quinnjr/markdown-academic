//! Citation resolution.

use crate::ast::{BibEntry, Block, Citation, Document, Inline};
use crate::error::{ResolutionError, Result};
use crate::resolve::ResolveConfig;
use std::collections::HashMap;

/// Resolve all citations in the document.
pub fn resolve_citations(
    mut document: Document,
    bibliography: &HashMap<String, BibEntry>,
    config: &ResolveConfig,
) -> Result<Document> {
    // Validate all citations exist
    let used_keys = collect_citation_keys(&document);

    for key in &used_keys {
        if !bibliography.contains_key(key) && config.strict_citations {
            return Err(ResolutionError::UnknownCitation(key.clone()).into());
        }
    }

    // Note: actual citation formatting happens in the renderer
    // This pass just validates citations exist

    Ok(document)
}

/// Collect all citation keys used in the document.
fn collect_citation_keys(document: &Document) -> Vec<String> {
    let mut keys = Vec::new();

    for block in &document.blocks {
        collect_block_citation_keys(block, &mut keys);
    }

    keys.sort();
    keys.dedup();
    keys
}

fn collect_block_citation_keys(block: &Block, keys: &mut Vec<String>) {
    match block {
        Block::Paragraph(inlines) => collect_inline_citation_keys(inlines, keys),
        Block::Heading { content, .. } => collect_inline_citation_keys(content, keys),
        Block::Environment { content, caption, .. } => {
            for block in content {
                collect_block_citation_keys(block, keys);
            }
            if let Some(caption) = caption {
                collect_inline_citation_keys(caption, keys);
            }
        }
        Block::BlockQuote(blocks) => {
            for block in blocks {
                collect_block_citation_keys(block, keys);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.content {
                    collect_block_citation_keys(block, keys);
                }
            }
        }
        Block::Table { headers, rows, caption, .. } => {
            for header in headers {
                collect_inline_citation_keys(header, keys);
            }
            for row in rows {
                for cell in row {
                    collect_inline_citation_keys(cell, keys);
                }
            }
            if let Some(caption) = caption {
                collect_inline_citation_keys(caption, keys);
            }
        }
        _ => {}
    }
}

fn collect_inline_citation_keys(inlines: &[Inline], keys: &mut Vec<String>) {
    for inline in inlines {
        match inline {
            Inline::Citation(cite) => {
                keys.extend(cite.keys.iter().cloned());
            }
            Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                collect_inline_citation_keys(inner, keys);
            }
            Inline::Link { content, .. } => {
                collect_inline_citation_keys(content, keys);
            }
            _ => {}
        }
    }
}

/// Get the list of citations in order of first appearance (for bibliography generation).
pub fn get_citation_order(document: &Document) -> Vec<String> {
    let mut keys = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for block in &document.blocks {
        collect_block_citation_order(block, &mut keys, &mut seen);
    }

    keys
}

fn collect_block_citation_order(
    block: &Block,
    keys: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    match block {
        Block::Paragraph(inlines) => collect_inline_citation_order(inlines, keys, seen),
        Block::Heading { content, .. } => collect_inline_citation_order(content, keys, seen),
        Block::Environment { content, caption, .. } => {
            for block in content {
                collect_block_citation_order(block, keys, seen);
            }
            if let Some(caption) = caption {
                collect_inline_citation_order(caption, keys, seen);
            }
        }
        Block::BlockQuote(blocks) => {
            for block in blocks {
                collect_block_citation_order(block, keys, seen);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.content {
                    collect_block_citation_order(block, keys, seen);
                }
            }
        }
        _ => {}
    }
}

fn collect_inline_citation_order(
    inlines: &[Inline],
    keys: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    for inline in inlines {
        match inline {
            Inline::Citation(cite) => {
                for key in &cite.keys {
                    if seen.insert(key.clone()) {
                        keys.push(key.clone());
                    }
                }
            }
            Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                collect_inline_citation_order(inner, keys, seen);
            }
            Inline::Link { content, .. } => {
                collect_inline_citation_order(content, keys, seen);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_collect_citation_keys() {
        let input = "This is from [@knuth1984] and [@lamport1994].";
        let doc = parse(input).unwrap();
        let keys = collect_citation_keys(&doc);
        assert_eq!(keys, vec!["knuth1984", "lamport1994"]);
    }
}
