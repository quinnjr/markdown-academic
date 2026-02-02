//! Macro expansion for user-defined LaTeX commands.

use crate::ast::{Block, Document, Inline, Macro};
use crate::error::Result;
use std::collections::HashMap;

/// Expand all user-defined macros in the document.
pub fn expand_macros(mut document: Document) -> Result<Document> {
    let macros = document.metadata.macros.clone();

    if macros.is_empty() {
        return Ok(document);
    }

    // Expand macros in all blocks
    document.blocks = document
        .blocks
        .into_iter()
        .map(|block| expand_block_macros(block, &macros))
        .collect();

    Ok(document)
}

fn expand_block_macros(block: Block, macros: &HashMap<String, Macro>) -> Block {
    match block {
        Block::Paragraph(inlines) => {
            Block::Paragraph(expand_inlines_macros(inlines, macros))
        }
        Block::Heading { level, content, label } => Block::Heading {
            level,
            content: expand_inlines_macros(content, macros),
            label,
        },
        Block::DisplayMath { content, label } => Block::DisplayMath {
            content: expand_math_macros(&content, macros),
            label,
        },
        Block::Environment { kind, label, content, caption } => Block::Environment {
            kind,
            label,
            content: content.into_iter().map(|b| expand_block_macros(b, macros)).collect(),
            caption: caption.map(|c| expand_inlines_macros(c, macros)),
        },
        Block::BlockQuote(blocks) => {
            Block::BlockQuote(blocks.into_iter().map(|b| expand_block_macros(b, macros)).collect())
        }
        Block::List { ordered, start, items } => Block::List {
            ordered,
            start,
            items: items
                .into_iter()
                .map(|item| crate::ast::ListItem {
                    content: item.content.into_iter().map(|b| expand_block_macros(b, macros)).collect(),
                    checked: item.checked,
                })
                .collect(),
        },
        Block::Table { headers, alignments, rows, label, caption } => Block::Table {
            headers: headers.into_iter().map(|h| expand_inlines_macros(h, macros)).collect(),
            alignments,
            rows: rows.into_iter().map(|row| row.into_iter().map(|cell| expand_inlines_macros(cell, macros)).collect()).collect(),
            label,
            caption: caption.map(|c| expand_inlines_macros(c, macros)),
        },
        // Pass through unchanged
        other => other,
    }
}

fn expand_inlines_macros(inlines: Vec<Inline>, macros: &HashMap<String, Macro>) -> Vec<Inline> {
    inlines
        .into_iter()
        .map(|inline| expand_inline_macros(inline, macros))
        .collect()
}

fn expand_inline_macros(inline: Inline, macros: &HashMap<String, Macro>) -> Inline {
    match inline {
        Inline::InlineMath(content) => {
            Inline::InlineMath(expand_math_macros(&content, macros))
        }
        Inline::Emphasis(inlines) => {
            Inline::Emphasis(expand_inlines_macros(inlines, macros))
        }
        Inline::Strong(inlines) => {
            Inline::Strong(expand_inlines_macros(inlines, macros))
        }
        Inline::Strikethrough(inlines) => {
            Inline::Strikethrough(expand_inlines_macros(inlines, macros))
        }
        Inline::Link { url, title, content } => Inline::Link {
            url,
            title,
            content: expand_inlines_macros(content, macros),
        },
        other => other,
    }
}

/// Expand macros in math content.
fn expand_math_macros(content: &str, macros: &HashMap<String, Macro>) -> String {
    let mut result = content.to_string();

    // Expand macros iteratively (to handle nested macros)
    // Limit iterations to prevent infinite loops
    for _ in 0..10 {
        let mut changed = false;

        for (name, macro_def) in macros {
            let expanded = expand_single_macro(&result, name, macro_def);
            if expanded != result {
                result = expanded;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    result
}

fn expand_single_macro(content: &str, name: &str, macro_def: &Macro) -> String {
    let pattern = format!("\\{}", name);
    let mut result = String::new();
    let mut remaining = content;

    while let Some(pos) = remaining.find(&pattern) {
        result.push_str(&remaining[..pos]);

        let after_name = &remaining[pos + pattern.len()..];

        if macro_def.arg_count == 0 {
            // Simple substitution
            result.push_str(&macro_def.template);
            remaining = after_name;
        } else {
            // Parse arguments
            match parse_macro_args(after_name, macro_def.arg_count) {
                Some((args, rest)) => {
                    let expanded = substitute_args(&macro_def.template, &args);
                    result.push_str(&expanded);
                    remaining = rest;
                }
                None => {
                    // Failed to parse args, keep original
                    result.push_str(&pattern);
                    remaining = after_name;
                }
            }
        }
    }

    result.push_str(remaining);
    result
}

fn parse_macro_args(input: &str, count: usize) -> Option<(Vec<String>, &str)> {
    let mut args = Vec::new();
    let mut remaining = input;

    for _ in 0..count {
        remaining = remaining.trim_start();

        if !remaining.starts_with('{') {
            return None;
        }

        // Find matching closing brace
        let mut depth = 0;
        let mut end = 0;

        for (i, c) in remaining.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return None;
        }

        let arg = &remaining[1..end];
        args.push(arg.to_string());
        remaining = &remaining[end + 1..];
    }

    Some((args, remaining))
}

fn substitute_args(template: &str, args: &[String]) -> String {
    let mut result = template.to_string();

    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("#{}", i + 1);
        result = result.replace(&placeholder, arg);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_simple_macro() {
        let mut macros = HashMap::new();
        macros.insert(
            "R".to_string(),
            Macro {
                arg_count: 0,
                template: "\\mathbb{R}".to_string(),
            },
        );

        let result = expand_math_macros("x \\in \\R", &macros);
        assert_eq!(result, "x \\in \\mathbb{R}");
    }

    #[test]
    fn test_expand_macro_with_args() {
        let mut macros = HashMap::new();
        macros.insert(
            "vec".to_string(),
            Macro {
                arg_count: 1,
                template: "\\mathbf{#1}".to_string(),
            },
        );

        let result = expand_math_macros("\\vec{x} + \\vec{y}", &macros);
        assert_eq!(result, "\\mathbf{x} + \\mathbf{y}");
    }

    #[test]
    fn test_expand_macro_with_multiple_args() {
        let mut macros = HashMap::new();
        macros.insert(
            "frac".to_string(),
            Macro {
                arg_count: 2,
                template: "\\dfrac{#1}{#2}".to_string(),
            },
        );

        let result = expand_math_macros("\\frac{a}{b}", &macros);
        assert_eq!(result, "\\dfrac{a}{b}");
    }

    #[test]
    fn test_parse_macro_args() {
        let input = "{x}{y} + z";
        let (args, rest) = parse_macro_args(input, 2).unwrap();
        assert_eq!(args, vec!["x", "y"]);
        assert_eq!(rest, " + z");
    }
}
