//! Cross-reference resolution.

use crate::ast::{Block, Document, EnvironmentKind, FootnoteKind, Inline, LabelInfo};
use crate::error::{ResolutionError, Result};
use crate::resolve::ResolveConfig;
use std::collections::HashMap;

/// Build a registry of all labels in the document.
pub fn build_label_registry(
    document: &Document,
    section_numbers: &HashMap<String, String>,
    env_numbers: &HashMap<String, u32>,
) -> Result<HashMap<String, LabelInfo>> {
    let mut labels = HashMap::new();

    for block in &document.blocks {
        collect_block_labels(block, &mut labels, section_numbers, env_numbers)?;
    }

    Ok(labels)
}

fn collect_block_labels(
    block: &Block,
    labels: &mut HashMap<String, LabelInfo>,
    section_numbers: &HashMap<String, String>,
    env_numbers: &HashMap<String, u32>,
) -> Result<()> {
    match block {
        Block::Heading { level, label, content, .. } => {
            if let Some(lbl) = label {
                let display = if let Some(num) = section_numbers.get(lbl) {
                    format!("Section {}", num)
                } else {
                    // Use heading text
                    inlines_to_text(content)
                };

                let html_id = label_to_id(lbl);

                if labels.contains_key(lbl) {
                    return Err(ResolutionError::DuplicateLabel(lbl.clone()).into());
                }

                labels.insert(
                    lbl.clone(),
                    LabelInfo { display, html_id },
                );
            }
        }
        Block::DisplayMath { label, .. } => {
            if let Some(lbl) = label {
                let display = if let Some(num) = env_numbers.get(lbl) {
                    format!("({})", num)
                } else {
                    "(?)".to_string()
                };

                let html_id = label_to_id(lbl);

                if labels.contains_key(lbl) {
                    return Err(ResolutionError::DuplicateLabel(lbl.clone()).into());
                }

                labels.insert(
                    lbl.clone(),
                    LabelInfo { display, html_id },
                );
            }
        }
        Block::Environment { kind, label, .. } => {
            if let Some(lbl) = label {
                let display = if let Some(num) = env_numbers.get(lbl) {
                    format!("{} {}", kind.display_name(), num)
                } else {
                    kind.display_name().to_string()
                };

                let html_id = label_to_id(lbl);

                if labels.contains_key(lbl) {
                    return Err(ResolutionError::DuplicateLabel(lbl.clone()).into());
                }

                labels.insert(
                    lbl.clone(),
                    LabelInfo { display, html_id },
                );
            }
        }
        Block::Table { label, .. } => {
            if let Some(lbl) = label {
                let display = if let Some(num) = env_numbers.get(lbl) {
                    format!("Table {}", num)
                } else {
                    "Table".to_string()
                };

                let html_id = label_to_id(lbl);

                if labels.contains_key(lbl) {
                    return Err(ResolutionError::DuplicateLabel(lbl.clone()).into());
                }

                labels.insert(
                    lbl.clone(),
                    LabelInfo { display, html_id },
                );
            }
        }
        Block::BlockQuote(blocks) | Block::Environment { content: blocks, .. } => {
            for block in blocks {
                collect_block_labels(block, labels, section_numbers, env_numbers)?;
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.content {
                    collect_block_labels(block, labels, section_numbers, env_numbers)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Collect footnote definitions from the document.
pub fn collect_footnotes(document: &Document) -> Result<HashMap<String, Vec<Inline>>> {
    let mut footnotes = HashMap::new();
    let mut counter = 1;

    for block in &document.blocks {
        collect_block_footnotes(block, &mut footnotes, &mut counter)?;
    }

    Ok(footnotes)
}

fn collect_block_footnotes(
    block: &Block,
    footnotes: &mut HashMap<String, Vec<Inline>>,
    counter: &mut u32,
) -> Result<()> {
    match block {
        Block::Paragraph(inlines) => {
            collect_inline_footnotes(inlines, footnotes, counter)?;
        }
        Block::Heading { content, .. } => {
            collect_inline_footnotes(content, footnotes, counter)?;
        }
        Block::Environment { content, caption, .. } => {
            for block in content {
                collect_block_footnotes(block, footnotes, counter)?;
            }
            if let Some(caption) = caption {
                collect_inline_footnotes(caption, footnotes, counter)?;
            }
        }
        Block::BlockQuote(blocks) => {
            for block in blocks {
                collect_block_footnotes(block, footnotes, counter)?;
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.content {
                    collect_block_footnotes(block, footnotes, counter)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn collect_inline_footnotes(
    inlines: &[Inline],
    footnotes: &mut HashMap<String, Vec<Inline>>,
    counter: &mut u32,
) -> Result<()> {
    for inline in inlines {
        match inline {
            Inline::Footnote(FootnoteKind::Inline(content)) => {
                let id = format!("fn-{}", counter);
                footnotes.insert(id, content.clone());
                *counter += 1;
            }
            Inline::Emphasis(inlines) | Inline::Strong(inlines) | Inline::Strikethrough(inlines) => {
                collect_inline_footnotes(inlines, footnotes, counter)?;
            }
            Inline::Link { content, .. } => {
                collect_inline_footnotes(content, footnotes, counter)?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Resolve all references in the document.
pub fn resolve_references(
    mut document: Document,
    labels: &HashMap<String, LabelInfo>,
    config: &ResolveConfig,
) -> Result<Document> {
    document.blocks = document
        .blocks
        .into_iter()
        .map(|block| resolve_block_references(block, labels, config))
        .collect::<Result<Vec<_>>>()?;

    Ok(document)
}

fn resolve_block_references(
    block: Block,
    labels: &HashMap<String, LabelInfo>,
    config: &ResolveConfig,
) -> Result<Block> {
    match block {
        Block::Paragraph(inlines) => {
            Ok(Block::Paragraph(resolve_inlines_references(inlines, labels, config)?))
        }
        Block::Heading { level, content, label } => Ok(Block::Heading {
            level,
            content: resolve_inlines_references(content, labels, config)?,
            label,
        }),
        Block::Environment { kind, label, content, caption } => Ok(Block::Environment {
            kind,
            label,
            content: content
                .into_iter()
                .map(|b| resolve_block_references(b, labels, config))
                .collect::<Result<Vec<_>>>()?,
            caption: caption
                .map(|c| resolve_inlines_references(c, labels, config))
                .transpose()?,
        }),
        Block::BlockQuote(blocks) => Ok(Block::BlockQuote(
            blocks
                .into_iter()
                .map(|b| resolve_block_references(b, labels, config))
                .collect::<Result<Vec<_>>>()?,
        )),
        Block::List { ordered, start, items } => Ok(Block::List {
            ordered,
            start,
            items: items
                .into_iter()
                .map(|item| {
                    Ok(crate::ast::ListItem {
                        content: item
                            .content
                            .into_iter()
                            .map(|b| resolve_block_references(b, labels, config))
                            .collect::<Result<Vec<_>>>()?,
                        checked: item.checked,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        }),
        Block::Table { headers, alignments, rows, label, caption } => Ok(Block::Table {
            headers: headers
                .into_iter()
                .map(|h| resolve_inlines_references(h, labels, config))
                .collect::<Result<Vec<_>>>()?,
            alignments,
            rows: rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|cell| resolve_inlines_references(cell, labels, config))
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?,
            label,
            caption: caption
                .map(|c| resolve_inlines_references(c, labels, config))
                .transpose()?,
        }),
        other => Ok(other),
    }
}

fn resolve_inlines_references(
    inlines: Vec<Inline>,
    labels: &HashMap<String, LabelInfo>,
    config: &ResolveConfig,
) -> Result<Vec<Inline>> {
    inlines
        .into_iter()
        .map(|inline| resolve_inline_references(inline, labels, config))
        .collect()
}

fn resolve_inline_references(
    inline: Inline,
    labels: &HashMap<String, LabelInfo>,
    config: &ResolveConfig,
) -> Result<Inline> {
    match inline {
        Inline::Reference { label, .. } => {
            let resolved = if let Some(info) = labels.get(&label) {
                Some(info.display.clone())
            } else {
                if config.strict_references {
                    return Err(ResolutionError::UnknownReference(label.clone()).into());
                }
                // Leave as unresolved marker
                Some(format!("??{}", label))
            };

            Ok(Inline::Reference { label, resolved })
        }
        Inline::Emphasis(inlines) => Ok(Inline::Emphasis(
            resolve_inlines_references(inlines, labels, config)?,
        )),
        Inline::Strong(inlines) => Ok(Inline::Strong(
            resolve_inlines_references(inlines, labels, config)?,
        )),
        Inline::Strikethrough(inlines) => Ok(Inline::Strikethrough(
            resolve_inlines_references(inlines, labels, config)?,
        )),
        Inline::Link { url, title, content } => Ok(Inline::Link {
            url,
            title,
            content: resolve_inlines_references(content, labels, config)?,
        }),
        other => Ok(other),
    }
}

/// Convert a label to a valid HTML id.
pub fn label_to_id(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Convert inline elements to plain text.
fn inlines_to_text(inlines: &[Inline]) -> String {
    let mut result = String::new();

    for inline in inlines {
        match inline {
            Inline::Text(t) => result.push_str(t),
            Inline::Code(t) => result.push_str(t),
            Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                result.push_str(&inlines_to_text(inner));
            }
            Inline::Link { content, .. } => {
                result.push_str(&inlines_to_text(content));
            }
            Inline::InlineMath(m) => {
                result.push_str(m);
            }
            Inline::SoftBreak | Inline::HardBreak => result.push(' '),
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_to_id() {
        assert_eq!(label_to_id("sec:intro"), "sec-intro");
        assert_eq!(label_to_id("eq:euler"), "eq-euler");
        assert_eq!(label_to_id("fig-1"), "fig-1");
    }
}
