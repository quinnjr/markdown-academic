//! Inline-level parsing for Markdown.

use crate::ast::{Citation, FootnoteKind, Inline};
use crate::error::{ParseError, Result};
use crate::parser::lexer::{
    citation, display_math, emphasis, footnote_inline, footnote_ref, inline_code, inline_math,
    label, reference, strong, CitationToken, Token,
};

/// Parse inline content from a string.
pub fn parse_inlines(input: &str) -> Result<Vec<Inline>> {
    let mut inlines = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        // Try to parse special inline elements
        if let Some((inline, rest)) = try_parse_inline(remaining)? {
            // Skip empty text nodes from labels
            if !matches!(&inline, Inline::Text(t) if t.is_empty()) {
                inlines.push(inline);
            }
            remaining = rest;
        } else {
            // Consume plain text until the next special character or end
            let (text, rest) = consume_text(remaining);
            if !text.is_empty() {
                // Handle line breaks in text
                if text.contains('\n') {
                    let parts: Vec<&str> = text.split('\n').collect();
                    for (i, part) in parts.iter().enumerate() {
                        if !part.is_empty() {
                            inlines.push(Inline::Text(part.to_string()));
                        }
                        if i < parts.len() - 1 {
                            // Check for hard break (two trailing spaces or backslash)
                            if part.ends_with("  ") || part.ends_with('\\') {
                                inlines.push(Inline::HardBreak);
                            } else {
                                inlines.push(Inline::SoftBreak);
                            }
                        }
                    }
                } else {
                    inlines.push(Inline::Text(text.to_string()));
                }
                remaining = rest;
            } else if rest == remaining {
                // No progress made - consume one character to avoid infinite loop
                let c = remaining.chars().next().unwrap();
                inlines.push(Inline::Text(c.to_string()));
                remaining = &remaining[c.len_utf8()..];
            } else {
                remaining = rest;
            }
        }
    }

    Ok(inlines)
}

fn try_parse_inline(input: &str) -> Result<Option<(Inline, &str)>> {
    // Order matters - try more specific patterns first

    // Display math ($$...$$)
    if input.starts_with("$$") {
        if let Ok((rest, Token::DisplayMath(content))) = display_math(input) {
            // Display math in inline context - treat as inline math
            return Ok(Some((Inline::InlineMath(content.to_string()), rest)));
        }
    }

    // Inline math ($...$)
    if input.starts_with('$') && !input.starts_with("$$") {
        if let Ok((rest, Token::InlineMath(content))) = inline_math(input) {
            return Ok(Some((Inline::InlineMath(content.to_string()), rest)));
        }
    }

    // Strong (**...** or __...__)
    if input.starts_with("**") || input.starts_with("__") {
        if let Ok((rest, Token::Strong(content))) = strong(input) {
            let inner = parse_inlines(content)?;
            return Ok(Some((Inline::Strong(inner), rest)));
        }
    }

    // Emphasis (*...* or _..._)
    if (input.starts_with('*') && !input.starts_with("**"))
        || (input.starts_with('_') && !input.starts_with("__"))
    {
        if let Ok((rest, Token::Emphasis(content))) = emphasis(input) {
            let inner = parse_inlines(content)?;
            return Ok(Some((Inline::Emphasis(inner), rest)));
        }
    }

    // Strikethrough (~~...~~)
    if input.starts_with("~~") {
        if let Some(end) = input[2..].find("~~") {
            let content = &input[2..2 + end];
            let rest = &input[2 + end + 2..];
            let inner = parse_inlines(content)?;
            return Ok(Some((Inline::Strikethrough(inner), rest)));
        }
    }

    // Inline code (`...`)
    if input.starts_with('`') && !input.starts_with("```") {
        if let Ok((rest, Token::InlineCode(content))) = inline_code(input) {
            return Ok(Some((Inline::Code(content.to_string()), rest)));
        }
    }

    // Citation ([@key])
    if input.starts_with("[@") {
        if let Ok((rest, Token::Citation(cites))) = citation(input) {
            let cite = Citation {
                keys: cites.iter().map(|c| c.key.to_string()).collect(),
                prefix: None,
                locator: cites.first().and_then(|c| c.locator.map(String::from)),
            };
            return Ok(Some((Inline::Citation(cite), rest)));
        }
    }

    // Footnote inline (^[...])
    if input.starts_with("^[") {
        if let Ok((rest, Token::FootnoteInline(content))) = footnote_inline(input) {
            let inner = parse_inlines(content)?;
            return Ok(Some((
                Inline::Footnote(FootnoteKind::Inline(inner)),
                rest,
            )));
        }
    }

    // Footnote reference ([^...])
    if input.starts_with("[^") {
        if let Ok((rest, Token::FootnoteRef(id))) = footnote_ref(input) {
            return Ok(Some((
                Inline::Footnote(FootnoteKind::Reference(id.to_string())),
                rest,
            )));
        }
    }

    // Cross-reference (@label)
    if input.starts_with('@') && !input.starts_with("[@") {
        if let Ok((rest, Token::Reference(lbl))) = reference(input) {
            return Ok(Some((
                Inline::Reference {
                    label: lbl.to_string(),
                    resolved: None,
                },
                rest,
            )));
        }
    }

    // Label ({#...})
    if input.starts_with("{#") {
        if let Ok((rest, Token::Label(_))) = label(input) {
            // Labels are metadata, not rendered inline - skip them
            return Ok(Some((Inline::Text(String::new()), rest)));
        }
    }

    // Link ([text](url "title"))
    if input.starts_with('[') && !input.starts_with("[^") && !input.starts_with("[@") {
        if let Some((inline, rest)) = try_parse_link(input)? {
            return Ok(Some((inline, rest)));
        }
    }

    // Image (![alt](url "title"))
    if input.starts_with("![") {
        if let Some((inline, rest)) = try_parse_image(input)? {
            return Ok(Some((inline, rest)));
        }
    }

    // Raw HTML (<tag>)
    if input.starts_with('<') {
        if let Some((inline, rest)) = try_parse_raw_html(input)? {
            return Ok(Some((inline, rest)));
        }
    }

    Ok(None)
}

fn try_parse_link(input: &str) -> Result<Option<(Inline, &str)>> {
    // [text](url "title")
    if !input.starts_with('[') {
        return Ok(None);
    }

    let mut depth = 0;
    let mut text_end = None;

    for (i, c) in input.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    text_end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let text_end = match text_end {
        Some(e) => e,
        None => return Ok(None),
    };

    let text = &input[1..text_end];
    let after_text = &input[text_end + 1..];

    if !after_text.starts_with('(') {
        return Ok(None);
    }

    // Find closing paren, handling nested parens
    let mut depth = 0;
    let mut url_end = None;

    for (i, c) in after_text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    url_end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let url_end = match url_end {
        Some(e) => e,
        None => return Ok(None),
    };

    let url_part = &after_text[1..url_end];
    let rest = &after_text[url_end + 1..];

    // Parse URL and optional title
    let (url, title) = parse_url_and_title(url_part);

    let content = parse_inlines(text)?;

    Ok(Some((
        Inline::Link {
            url: url.to_string(),
            title: title.map(String::from),
            content,
        },
        rest,
    )))
}

fn try_parse_image(input: &str) -> Result<Option<(Inline, &str)>> {
    // ![alt](url "title")
    if !input.starts_with("![") {
        return Ok(None);
    }

    let close_bracket = match input[2..].find(']') {
        Some(i) => i + 2,
        None => return Ok(None),
    };

    let alt = &input[2..close_bracket];
    let after_alt = &input[close_bracket + 1..];

    if !after_alt.starts_with('(') {
        return Ok(None);
    }

    let close_paren = match after_alt.find(')') {
        Some(i) => i,
        None => return Ok(None),
    };

    let url_part = &after_alt[1..close_paren];
    let rest = &after_alt[close_paren + 1..];

    let (url, title) = parse_url_and_title(url_part);

    Ok(Some((
        Inline::Image {
            url: url.to_string(),
            alt: alt.to_string(),
            title: title.map(String::from),
        },
        rest,
    )))
}

fn parse_url_and_title(input: &str) -> (&str, Option<&str>) {
    let input = input.trim();

    // Check for title in quotes
    if let Some(quote_start) = input.find('"') {
        if let Some(quote_end) = input[quote_start + 1..].find('"') {
            let url = input[..quote_start].trim();
            let title = &input[quote_start + 1..quote_start + 1 + quote_end];
            return (url, Some(title));
        }
    }

    // Check for title in single quotes
    if let Some(quote_start) = input.find('\'') {
        if let Some(quote_end) = input[quote_start + 1..].find('\'') {
            let url = input[..quote_start].trim();
            let title = &input[quote_start + 1..quote_start + 1 + quote_end];
            return (url, Some(title));
        }
    }

    (input, None)
}

fn try_parse_raw_html(input: &str) -> Result<Option<(Inline, &str)>> {
    if !input.starts_with('<') {
        return Ok(None);
    }

    // Find the closing >
    let close = match input.find('>') {
        Some(i) => i,
        None => return Ok(None),
    };

    // Check if it looks like a tag
    let tag_content = &input[1..close];
    if tag_content.is_empty() || !tag_content.chars().next().unwrap().is_alphabetic() {
        return Ok(None);
    }

    let html = &input[..=close];
    let rest = &input[close + 1..];

    Ok(Some((Inline::RawHtml(html.to_string()), rest)))
}

fn consume_text(input: &str) -> (&str, &str) {
    // Special characters that might start inline elements
    const SPECIAL: &[char] = &['*', '_', '`', '$', '[', '!', '@', '^', '<', '~', '{', '\n'];

    let mut end = 0;
    let mut chars = input.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if SPECIAL.contains(&c) {
            // Check for escaped character
            if i > 0 && input.as_bytes()[i - 1] == b'\\' {
                end = i + c.len_utf8();
                continue;
            }

            // Special handling for potential inline elements
            if c == '*' || c == '_' {
                // Check if followed by non-space (potential emphasis/strong)
                if let Some(&(_, next)) = chars.peek() {
                    if !next.is_whitespace() {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..end.max(i)], &input[end.max(i)..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '~' {
                // Check for strikethrough
                if let Some(&(_, next)) = chars.peek() {
                    if next == '~' {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '{' {
                // Check for label
                if let Some(&(_, next)) = chars.peek() {
                    if next == '#' {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '!' {
                // Check for image (![ )
                if let Some(&(_, next)) = chars.peek() {
                    if next == '[' {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '@' {
                // Check for citation or reference
                if let Some(&(_, next)) = chars.peek() {
                    if next == '[' || next.is_alphanumeric() {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '^' {
                // Check for footnote (^[ )
                if let Some(&(_, next)) = chars.peek() {
                    if next == '[' {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            if c == '<' {
                // Check for HTML tag
                if let Some(&(_, next)) = chars.peek() {
                    if next.is_alphabetic() || next == '/' {
                        if end == 0 && i == 0 {
                            return ("", input);
                        }
                        return (&input[..i], &input[i..]);
                    }
                }
                end = i + c.len_utf8();
                continue;
            }

            // For remaining special chars ([, $, `, \n), stop here
            if end == 0 && i == 0 {
                return ("", input);
            }
            return (&input[..end.max(i)], &input[end.max(i)..]);
        }

        end = i + c.len_utf8();
    }

    (input, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let inlines = parse_inlines("Hello, world!").unwrap();
        assert_eq!(inlines.len(), 1);
        if let Inline::Text(t) = &inlines[0] {
            assert_eq!(t, "Hello, world!");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_emphasis() {
        let inlines = parse_inlines("Hello *world*!").unwrap();
        assert_eq!(inlines.len(), 3);
        assert!(matches!(&inlines[1], Inline::Emphasis(_)));
    }

    #[test]
    fn test_strong() {
        let inlines = parse_inlines("Hello **world**!").unwrap();
        assert!(matches!(&inlines[1], Inline::Strong(_)));
    }

    #[test]
    fn test_inline_math() {
        let inlines = parse_inlines("The equation $E = mc^2$ is famous.").unwrap();
        let math_count = inlines.iter().filter(|i| matches!(i, Inline::InlineMath(_))).count();
        assert_eq!(math_count, 1);
    }

    #[test]
    fn test_citation() {
        let inlines = parse_inlines("As shown in [@knuth1984].").unwrap();
        let cite_count = inlines.iter().filter(|i| matches!(i, Inline::Citation(_))).count();
        assert_eq!(cite_count, 1);
    }

    #[test]
    fn test_reference() {
        let inlines = parse_inlines("See @eq:euler for details.").unwrap();
        let ref_count = inlines
            .iter()
            .filter(|i| matches!(i, Inline::Reference { .. }))
            .count();
        assert_eq!(ref_count, 1);
    }

    #[test]
    fn test_link() {
        let inlines = parse_inlines("Click [here](https://example.com \"Title\")!").unwrap();
        let link = inlines.iter().find(|i| matches!(i, Inline::Link { .. }));
        assert!(link.is_some());
        if let Some(Inline::Link { url, title, .. }) = link {
            assert_eq!(url, "https://example.com");
            assert_eq!(title.as_deref(), Some("Title"));
        }
    }

    #[test]
    fn test_footnote_inline() {
        let inlines = parse_inlines("Some text^[This is a note].").unwrap();
        let fn_count = inlines.iter().filter(|i| matches!(i, Inline::Footnote(_))).count();
        assert_eq!(fn_count, 1);
    }
}
