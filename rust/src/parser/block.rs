//! Block-level parsing for Markdown.

use crate::ast::{Alignment, Block, DescriptionItem, EnvironmentKind, ListItem};
use crate::error::Result;
use crate::parser::inline::parse_inlines;
use crate::parser::lexer::{
    block_quote_marker, environment_end, environment_start, fenced_code_end, fenced_code_start,
    heading, is_blank_line, list_item_marker, table_of_contents, thematic_break, ListMarker, Token,
};
use nom::combinator::opt;

/// Parse all blocks from content.
pub fn parse_blocks(input: &str) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Skip blank lines
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Try parsing different block types
        if let Some((block, consumed)) = try_parse_heading(line)? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_page_break(line)? {
            // Check page breaks before thematic breaks (---pagebreak--- vs ---)
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_appendix_marker(line)? {
            // Check appendix markers before thematic breaks
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_thematic_break(line)? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_toc(line)? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_fenced_code(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_display_math(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_environment(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_block_quote(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_list(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_table(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else if let Some((block, consumed)) = try_parse_description_list(&lines[i..])? {
            blocks.push(block);
            i += consumed;
        } else {
            // Default: paragraph
            let (block, consumed) = parse_paragraph(&lines[i..])?;
            blocks.push(block);
            i += consumed;
        }
    }

    Ok(blocks)
}

fn try_parse_heading(line: &str) -> Result<Option<(Block, usize)>> {
    if !line.trim_start().starts_with('#') {
        return Ok(None);
    }

    match heading(line.trim_start()) {
        Ok((rest, Token::Heading(level, content))) => {
            // Check for label at end
            let (content, label) = extract_label(content);
            let inlines = parse_inlines(content)?;
            Ok(Some((
                Block::Heading {
                    level,
                    content: inlines,
                    label,
                },
                1,
            )))
        }
        _ => Ok(None),
    }
}

fn try_parse_thematic_break(line: &str) -> Result<Option<(Block, usize)>> {
    let trimmed = line.trim_start();
    if thematic_break(trimmed).is_ok() {
        Ok(Some((Block::ThematicBreak, 1)))
    } else {
        Ok(None)
    }
}

fn try_parse_toc(line: &str) -> Result<Option<(Block, usize)>> {
    let trimmed = line.trim();
    if trimmed == "[[toc]]" {
        Ok(Some((Block::TableOfContents, 1)))
    } else {
        Ok(None)
    }
}

fn try_parse_fenced_code(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    let first = lines[0].trim_start();
    
    if !first.starts_with("```") && !first.starts_with("~~~") {
        return Ok(None);
    }

    let fence_char = first.chars().next().unwrap();
    let fence = if first.starts_with("```") { "```" } else { "~~~" };
    
    match fenced_code_start(first) {
        Ok((_, Token::FencedCodeStart(lang))) => {
            let mut content = String::new();
            let mut i = 1;

            while i < lines.len() {
                let line = lines[i];
                if line.trim_start().starts_with(fence) {
                    return Ok(Some((
                        Block::CodeBlock {
                            language: if lang.is_empty() { None } else { Some(lang.to_string()) },
                            content,
                        },
                        i + 1,
                    )));
                }
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(line);
                i += 1;
            }

            // Unclosed fence - treat rest as code
            Ok(Some((
                Block::CodeBlock {
                    language: if lang.is_empty() { None } else { Some(lang.to_string()) },
                    content,
                },
                lines.len(),
            )))
        }
        _ => Ok(None),
    }
}

fn try_parse_display_math(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    let first = lines[0].trim_start();
    
    if !first.starts_with("$$") {
        return Ok(None);
    }

    // Check for single-line display math
    let after_open = &first[2..];
    if let Some(end_pos) = after_open.find("$$") {
        let content = after_open[..end_pos].to_string();
        let rest = &after_open[end_pos + 2..];
        let label = extract_label(rest).1;
        return Ok(Some((
            Block::DisplayMath { content, label },
            1,
        )));
    }

    // Multi-line display math
    let mut content = String::from(after_open);
    let mut i = 1;

    while i < lines.len() {
        let line = lines[i];
        if let Some(end_pos) = line.find("$$") {
            content.push('\n');
            content.push_str(&line[..end_pos]);
            let rest = &line[end_pos + 2..];
            let label = extract_label(rest).1;
            return Ok(Some((
                Block::DisplayMath {
                    content: content.trim().to_string(),
                    label,
                },
                i + 1,
            )));
        }
        content.push('\n');
        content.push_str(line);
        i += 1;
    }

    // Unclosed math
    Ok(Some((
        Block::DisplayMath {
            content: content.trim().to_string(),
            label: None,
        },
        lines.len(),
    )))
}

fn try_parse_environment(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    let first = lines[0].trim_start();
    
    if !first.starts_with(":::") {
        return Ok(None);
    }

    // Check for environment start (not just :::)
    match environment_start(first) {
        Ok((_, Token::EnvironmentStart(kind, label))) => {
            let env_kind = EnvironmentKind::from_str(kind);
            let mut inner_lines = Vec::new();
            let mut i = 1;
            let mut depth = 1;

            while i < lines.len() {
                let line = lines[i];
                let trimmed = line.trim_start();

                if trimmed == ":::" {
                    depth -= 1;
                    if depth == 0 {
                        let inner_content = inner_lines.join("\n");
                        let (content, caption) = parse_environment_content(&inner_content, &env_kind)?;
                        return Ok(Some((
                            Block::Environment {
                                kind: env_kind,
                                label: label.map(String::from),
                                content,
                                caption,
                            },
                            i + 1,
                        )));
                    }
                } else if trimmed.starts_with("::: ") {
                    depth += 1;
                }

                inner_lines.push(line);
                i += 1;
            }

            // Unclosed environment
            let inner_content = inner_lines.join("\n");
            let (content, caption) = parse_environment_content(&inner_content, &env_kind)?;
            Ok(Some((
                Block::Environment {
                    kind: env_kind,
                    label: label.map(String::from),
                    content,
                    caption,
                },
                lines.len(),
            )))
        }
        _ => Ok(None),
    }
}

fn parse_environment_content(
    content: &str,
    kind: &EnvironmentKind,
) -> Result<(Vec<Block>, Option<Vec<crate::ast::Inline>>)> {
    // For figures/tables, look for a caption at the end
    let blocks = parse_blocks(content)?;
    
    if matches!(kind, EnvironmentKind::Figure | EnvironmentKind::Table) {
        // Check if last block is a paragraph that looks like a caption
        if let Some(Block::Paragraph(inlines)) = blocks.last() {
            if blocks.len() > 1 {
                let caption = inlines.clone();
                let content_blocks = blocks[..blocks.len() - 1].to_vec();
                return Ok((content_blocks, Some(caption)));
            }
        }
    }

    Ok((blocks, None))
}

fn try_parse_block_quote(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    let first = lines[0].trim_start();
    
    if !first.starts_with('>') {
        return Ok(None);
    }

    let mut quote_lines = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if trimmed.starts_with('>') {
            // Remove the > prefix
            let content = if trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
                &trimmed[2..]
            } else {
                &trimmed[1..]
            };
            quote_lines.push(content);
            i += 1;
        } else if trimmed.is_empty() && i + 1 < lines.len() && lines[i + 1].trim_start().starts_with('>') {
            // Blank line within quote
            quote_lines.push("");
            i += 1;
        } else {
            break;
        }
    }

    let inner_content = quote_lines.join("\n");
    let inner_blocks = parse_blocks(&inner_content)?;

    Ok(Some((Block::BlockQuote(inner_blocks), i)))
}

fn try_parse_list(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    let first = lines[0];
    let trimmed = first.trim_start();
    let indent = first.len() - trimmed.len();

    let marker_result = list_item_marker(trimmed);
    if marker_result.is_err() {
        return Ok(None);
    }

    let (rest, marker) = marker_result.unwrap();
    let Token::ListItemMarker(marker_type) = marker else {
        return Ok(None);
    };

    let ordered = matches!(marker_type, ListMarker::Ordered(_));
    let start = if let ListMarker::Ordered(n) = marker_type {
        Some(n)
    } else {
        None
    };

    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        let current_indent = line.len() - trimmed.len();

        // Check for list item at same or lesser indent
        if let Ok((rest, Token::ListItemMarker(m))) = list_item_marker(trimmed) {
            // Check if same list type
            let same_type = matches!(
                (&marker_type, &m),
                (ListMarker::Ordered(_), ListMarker::Ordered(_))
                    | (ListMarker::Unordered, ListMarker::Unordered)
                    | (ListMarker::Unordered, ListMarker::Checkbox(_))
                    | (ListMarker::Checkbox(_), ListMarker::Unordered)
                    | (ListMarker::Checkbox(_), ListMarker::Checkbox(_))
            );

            if current_indent <= indent && same_type {
                // Collect item content
                let mut item_lines = vec![rest];
                i += 1;

                while i < lines.len() {
                    let next_line = lines[i];
                    let next_trimmed = next_line.trim_start();
                    let next_indent = next_line.len() - next_trimmed.len();

                    // Check for new list item
                    if let Ok((_, Token::ListItemMarker(_))) = list_item_marker(next_trimmed) {
                        if next_indent <= indent {
                            break;
                        }
                    }

                    if next_trimmed.is_empty() {
                        // Check if next non-blank continues the item
                        let mut j = i + 1;
                        while j < lines.len() && lines[j].trim().is_empty() {
                            j += 1;
                        }
                        if j < lines.len() {
                            let future_indent = lines[j].len() - lines[j].trim_start().len();
                            if future_indent <= indent {
                                break;
                            }
                        }
                    }

                    // Content belongs to this item
                    item_lines.push(next_trimmed);
                    i += 1;
                }

                let content = item_lines.join("\n");
                let content_blocks = parse_blocks(&content)?;
                let checked = if let ListMarker::Checkbox(c) = m {
                    Some(c)
                } else {
                    None
                };

                items.push(ListItem {
                    content: content_blocks,
                    checked,
                });
            } else {
                break;
            }
        } else if current_indent > indent || trimmed.is_empty() {
            // Continuation of previous item
            i += 1;
        } else {
            break;
        }
    }

    if items.is_empty() {
        return Ok(None);
    }

    Ok(Some((
        Block::List {
            ordered,
            start,
            items,
        },
        i,
    )))
}

fn try_parse_table(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    // Check for pipe table
    let first = lines[0];
    if !first.contains('|') {
        return Ok(None);
    }

    // Need at least header row and delimiter row
    if lines.len() < 2 {
        return Ok(None);
    }

    // Check for delimiter row
    let second = lines[1];
    if !is_table_delimiter(second) {
        return Ok(None);
    }

    // Parse header
    let headers = parse_table_row(first)?;
    let alignments = parse_alignments(second);

    // Parse body rows
    let mut rows = Vec::new();
    let mut i = 2;

    while i < lines.len() {
        let line = lines[i];
        if !line.contains('|') || line.trim().is_empty() {
            break;
        }
        rows.push(parse_table_row(line)?);
        i += 1;
    }

    // Check for caption and label after table
    let (caption, label, extra_consumed) = if i < lines.len() {
        let next = lines[i].trim();
        if next.starts_with("Table:") || next.starts_with("Caption:") {
            let caption_text = next.split_once(':').map(|(_, t)| t.trim()).unwrap_or("");
            let (caption_text, label) = extract_label(caption_text);
            let caption_inlines = parse_inlines(caption_text)?;
            (Some(caption_inlines), label, 1)
        } else {
            (None, None, 0)
        }
    } else {
        (None, None, 0)
    };

    Ok(Some((
        Block::Table {
            headers,
            alignments,
            rows,
            label,
            caption,
        },
        i + extra_consumed,
    )))
}

fn is_table_delimiter(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return false;
    }

    // Remove leading/trailing pipes
    let inner = trimmed.trim_matches('|');
    
    // Check each cell is a valid delimiter
    for cell in inner.split('|') {
        let cell = cell.trim();
        if cell.is_empty() {
            continue;
        }
        
        let valid = cell.chars().all(|c| c == '-' || c == ':');
        if !valid || cell.chars().filter(|&c| c == '-').count() < 1 {
            return false;
        }
    }

    true
}

fn parse_alignments(line: &str) -> Vec<Alignment> {
    let trimmed = line.trim().trim_matches('|');
    trimmed
        .split('|')
        .map(|cell| {
            let cell = cell.trim();
            let left = cell.starts_with(':');
            let right = cell.ends_with(':');
            match (left, right) {
                (true, true) => Alignment::Center,
                (false, true) => Alignment::Right,
                _ => Alignment::Left,
            }
        })
        .collect()
}

fn parse_table_row(line: &str) -> Result<Vec<Vec<crate::ast::Inline>>> {
    let trimmed = line.trim().trim_matches('|');
    trimmed
        .split('|')
        .map(|cell| parse_inlines(cell.trim()))
        .collect()
}

fn parse_paragraph(lines: &[&str]) -> Result<(Block, usize)> {
    let mut para_lines = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // End paragraph on blank line
        if trimmed.is_empty() {
            break;
        }

        // End paragraph on block-level element
        if trimmed.starts_with('#')
            || trimmed.starts_with("```")
            || trimmed.starts_with("~~~")
            || trimmed.starts_with(":::")
            || trimmed.starts_with("$$")
            || trimmed.starts_with('>')
            || trimmed == "---"
            || trimmed == "***"
            || trimmed == "___"
            || trimmed == "[[toc]]"
        {
            break;
        }

        // Check for list markers
        if list_item_marker(trimmed).is_ok() {
            break;
        }

        para_lines.push(line);
        i += 1;
    }

    let content = para_lines.join("\n");
    let inlines = parse_inlines(&content)?;

    Ok((Block::Paragraph(inlines), i.max(1)))
}

/// Extract a label from the end of a string ({#label}).
fn extract_label(s: &str) -> (&str, Option<String>) {
    let trimmed = s.trim_end();
    if let Some(start) = trimmed.rfind("{#") {
        if let Some(end) = trimmed[start..].find('}') {
            let label = &trimmed[start + 2..start + end];
            let content = trimmed[..start].trim_end();
            return (content, Some(label.to_string()));
        }
    }
    (s, None)
}

/// Parse a description list (term : definition).
///
/// Syntax:
/// ```text
/// Term 1
/// : Definition of term 1
///
/// Term 2
/// : Definition of term 2
/// : Additional paragraph for term 2
/// ```
fn try_parse_description_list(lines: &[&str]) -> Result<Option<(Block, usize)>> {
    // Look ahead for a term followed by a definition line starting with ':'
    if lines.len() < 2 {
        return Ok(None);
    }

    let first = lines[0].trim();
    let second = lines[1].trim();

    // First line must not start with ':' and second line must start with ':'
    if first.starts_with(':') || !second.starts_with(':') {
        return Ok(None);
    }

    let mut items = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let term_line = lines[i].trim();

        // Check if this looks like a term (non-empty, doesn't start with ':')
        if term_line.is_empty() {
            i += 1;
            continue;
        }

        if term_line.starts_with(':') {
            // Definition without term - end the list
            break;
        }

        // Check if next line is a definition
        if i + 1 >= lines.len() || !lines[i + 1].trim().starts_with(':') {
            break;
        }

        // Parse the term
        let term = parse_inlines(term_line)?;
        i += 1;

        // Collect all definition lines
        let mut def_lines = Vec::new();
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with(':') {
                // Remove the ':' prefix
                let content = line[1..].trim();
                def_lines.push(content);
                i += 1;
            } else if line.is_empty() {
                // Blank line might continue the definition
                if i + 1 < lines.len() && lines[i + 1].trim().starts_with(':') {
                    def_lines.push("");
                    i += 1;
                } else {
                    break;
                }
            } else {
                // New term or end of list
                break;
            }
        }

        let def_content = def_lines.join("\n");
        let description = parse_blocks(&def_content)?;

        items.push(DescriptionItem { term, description });
    }

    if items.is_empty() {
        return Ok(None);
    }

    Ok(Some((Block::DescriptionList(items), i)))
}

/// Parse a page break marker.
///
/// Syntax: `---pagebreak---` or `\\pagebreak` or `\\newpage`
fn try_parse_page_break(line: &str) -> Result<Option<(Block, usize)>> {
    let trimmed = line.trim();

    if trimmed == "---pagebreak---"
        || trimmed == "\\pagebreak"
        || trimmed == "\\newpage"
        || trimmed == "<!-- pagebreak -->"
        || trimmed == "<!-- newpage -->"
    {
        return Ok(Some((Block::PageBreak, 1)));
    }

    Ok(None)
}

/// Parse an appendix marker.
///
/// Syntax: `\\appendix` or `---appendix---`
fn try_parse_appendix_marker(line: &str) -> Result<Option<(Block, usize)>> {
    let trimmed = line.trim();

    if trimmed == "\\appendix"
        || trimmed == "---appendix---"
        || trimmed == "<!-- appendix -->"
    {
        return Ok(Some((Block::AppendixMarker, 1)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let blocks = parse_blocks("# Hello World").unwrap();
        assert_eq!(blocks.len(), 1);
        if let Block::Heading { level, label, .. } = &blocks[0] {
            assert_eq!(*level, 1);
            assert_eq!(*label, None);
        } else {
            panic!("Expected heading");
        }
    }

    #[test]
    fn test_parse_heading_with_label() {
        let blocks = parse_blocks("## Introduction {#sec:intro}").unwrap();
        if let Block::Heading { level, label, .. } = &blocks[0] {
            assert_eq!(*level, 2);
            assert_eq!(label.as_deref(), Some("sec:intro"));
        } else {
            panic!("Expected heading");
        }
    }

    #[test]
    fn test_parse_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let blocks = parse_blocks(input).unwrap();
        if let Block::CodeBlock { language, content } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(content, "fn main() {}");
        } else {
            panic!("Expected code block");
        }
    }

    #[test]
    fn test_parse_display_math() {
        let input = "$$\n\\int_0^1 x dx\n$$";
        let blocks = parse_blocks(input).unwrap();
        if let Block::DisplayMath { content, label } = &blocks[0] {
            assert!(content.contains("\\int"));
            assert_eq!(*label, None);
        } else {
            panic!("Expected display math");
        }
    }

    #[test]
    fn test_parse_environment() {
        let input = "::: theorem {#thm:main}\nStatement here.\n:::";
        let blocks = parse_blocks(input).unwrap();
        if let Block::Environment { kind, label, .. } = &blocks[0] {
            assert_eq!(*kind, EnvironmentKind::Theorem);
            assert_eq!(label.as_deref(), Some("thm:main"));
        } else {
            panic!("Expected environment");
        }
    }

    #[test]
    fn test_table_delimiter() {
        assert!(is_table_delimiter("| --- | :---: | ---: |"));
        assert!(is_table_delimiter("|---|:---:|---:|"));
        assert!(!is_table_delimiter("| not | a | delimiter |"));
    }
}
