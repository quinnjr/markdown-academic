//! BibTeX parser for bibliography support.

use crate::ast::BibEntry;
use crate::error::{ParseError, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_until, take_while, take_while1},
    character::complete::{char, multispace0, multispace1, none_of, one_of},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

/// Parse a BibTeX file and return a map of citation keys to entries.
pub fn parse_bibtex(input: &str) -> Result<HashMap<String, BibEntry>> {
    let mut entries = HashMap::new();

    // Find all @type{...} entries
    let mut remaining = input;

    while !remaining.is_empty() {
        // Skip whitespace and comments
        remaining = skip_whitespace_and_comments(remaining);

        if remaining.is_empty() {
            break;
        }

        if remaining.starts_with('@') {
            match parse_entry(remaining) {
                Ok((rest, Some(entry))) => {
                    entries.insert(entry.key.clone(), entry);
                    remaining = rest;
                }
                Ok((rest, None)) => {
                    // @comment or @preamble - skip
                    remaining = rest;
                }
                Err(_) => {
                    // Try to recover by finding next @
                    if let Some(pos) = remaining[1..].find('@') {
                        remaining = &remaining[pos + 1..];
                    } else {
                        break;
                    }
                }
            }
        } else {
            // Skip unknown content
            if let Some(pos) = remaining.find('@') {
                remaining = &remaining[pos..];
            } else {
                break;
            }
        }
    }

    Ok(entries)
}

fn skip_whitespace_and_comments(input: &str) -> &str {
    let mut s = input;

    loop {
        s = s.trim_start();

        if s.starts_with('%') {
            // Skip line comment
            if let Some(end) = s.find('\n') {
                s = &s[end + 1..];
            } else {
                return "";
            }
        } else {
            break;
        }
    }

    s
}

fn parse_entry(input: &str) -> IResult<&str, Option<BibEntry>> {
    let (input, _) = char('@')(input)?;
    let (input, entry_type) = take_while1(|c: char| c.is_alphanumeric())(input)?;
    let (input, _) = multispace0(input)?;

    let entry_type_lower = entry_type.to_lowercase();

    // Handle special entries
    if entry_type_lower == "comment" || entry_type_lower == "preamble" || entry_type_lower == "string" {
        // Skip to matching brace
        let (input, _) = skip_braced_content(input)?;
        return Ok((input, None));
    }

    let (input, _) = char('{')(input)?;
    let (input, _) = multispace0(input)?;

    // Parse citation key
    let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(',')(input)?;

    // Parse fields
    let (input, fields) = parse_fields(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = char('}')(input)?;

    let entry = build_entry(key, &entry_type_lower, fields);

    Ok((input, Some(entry)))
}

fn skip_braced_content(input: &str) -> IResult<&str, ()> {
    let (input, _) = char('{')(input)?;
    let mut depth = 1;
    let mut i = 0;

    for (idx, c) in input.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&input[idx + 1..], ()));
                }
            }
            _ => {}
        }
        i = idx;
    }

    // Unmatched brace
    Ok(("", ()))
}

fn parse_fields(input: &str) -> IResult<&str, HashMap<String, String>> {
    let mut fields = HashMap::new();
    let mut remaining = input;

    loop {
        remaining = remaining.trim_start();

        if remaining.starts_with('}') || remaining.is_empty() {
            break;
        }

        match parse_field(remaining) {
            Ok((rest, (name, value))) => {
                fields.insert(name.to_lowercase(), value);
                remaining = rest.trim_start();

                // Optional comma
                if remaining.starts_with(',') {
                    remaining = &remaining[1..];
                }
            }
            Err(_) => break,
        }
    }

    Ok((remaining, fields))
}

fn parse_field(input: &str) -> IResult<&str, (String, String)> {
    let (input, _) = multispace0(input)?;
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value) = parse_value(input)?;

    Ok((input, (name.to_string(), value)))
}

fn parse_value(input: &str) -> IResult<&str, String> {
    alt((
        parse_braced_value,
        parse_quoted_value,
        parse_number_value,
    ))(input)
}

fn parse_braced_value(input: &str) -> IResult<&str, String> {
    let (input, _) = char('{')(input)?;
    let mut depth = 1;
    let mut end = 0;

    for (i, c) in input.char_indices() {
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

    let value = &input[..end];
    let rest = &input[end + 1..];

    Ok((rest, clean_bibtex_value(value)))
}

fn parse_quoted_value(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;

    let mut end = 0;
    let mut escape = false;

    for (i, c) in input.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            '"' => {
                end = i;
                break;
            }
            _ => {}
        }
    }

    let value = &input[..end];
    let rest = &input[end + 1..];

    Ok((rest, clean_bibtex_value(value)))
}

fn parse_number_value(input: &str) -> IResult<&str, String> {
    let (input, value) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    Ok((input, value.to_string()))
}

fn clean_bibtex_value(value: &str) -> String {
    // Remove LaTeX braces used for capitalization preservation
    let mut result = String::with_capacity(value.len());
    let mut depth = 0;
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                // Check if it's a LaTeX command
                if chars.peek() == Some(&'\\') {
                    result.push(c);
                    depth += 1;
                } else {
                    // Skip opening brace
                }
            }
            '}' => {
                if depth > 0 {
                    result.push(c);
                    depth -= 1;
                }
                // Skip closing brace of non-command braces
            }
            _ => result.push(c),
        }
    }

    // Normalize whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn build_entry(key: &str, entry_type: &str, fields: HashMap<String, String>) -> BibEntry {
    let mut entry = BibEntry {
        key: key.to_string(),
        entry_type: entry_type.to_string(),
        ..Default::default()
    };

    // Extract standard fields
    if let Some(v) = fields.get("title") {
        entry.title = Some(v.clone());
    }
    if let Some(v) = fields.get("author") {
        entry.authors = parse_authors(v);
    }
    if let Some(v) = fields.get("year") {
        entry.year = Some(v.clone());
    }
    if let Some(v) = fields.get("journal") {
        entry.journal = Some(v.clone());
    }
    if let Some(v) = fields.get("booktitle") {
        entry.booktitle = Some(v.clone());
    }
    if let Some(v) = fields.get("publisher") {
        entry.publisher = Some(v.clone());
    }
    if let Some(v) = fields.get("volume") {
        entry.volume = Some(v.clone());
    }
    if let Some(v) = fields.get("number") {
        entry.number = Some(v.clone());
    }
    if let Some(v) = fields.get("pages") {
        entry.pages = Some(v.clone());
    }
    if let Some(v) = fields.get("doi") {
        entry.doi = Some(v.clone());
    }
    if let Some(v) = fields.get("url") {
        entry.url = Some(v.clone());
    }

    // Store remaining fields
    for (k, v) in fields {
        if !matches!(
            k.as_str(),
            "title" | "author" | "year" | "journal" | "booktitle" | "publisher" 
            | "volume" | "number" | "pages" | "doi" | "url"
        ) {
            entry.extra.insert(k, v);
        }
    }

    entry
}

fn parse_authors(input: &str) -> Vec<String> {
    // Authors are separated by " and "
    input
        .split(" and ")
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_entry() {
        let input = r#"
@article{knuth1984,
    author = {Donald E. Knuth},
    title = {Literate Programming},
    journal = {The Computer Journal},
    year = {1984},
    volume = {27},
    number = {2},
    pages = {97--111}
}
"#;

        let entries = parse_bibtex(input).unwrap();
        assert_eq!(entries.len(), 1);

        let entry = entries.get("knuth1984").unwrap();
        assert_eq!(entry.entry_type, "article");
        assert_eq!(entry.title.as_deref(), Some("Literate Programming"));
        assert_eq!(entry.authors, vec!["Donald E. Knuth"]);
        assert_eq!(entry.year.as_deref(), Some("1984"));
    }

    #[test]
    fn test_parse_multiple_authors() {
        let input = r#"
@book{dragon2006,
    author = {Alfred V. Aho and Monica S. Lam and Ravi Sethi and Jeffrey D. Ullman},
    title = {Compilers: Principles, Techniques, and Tools},
    year = {2006}
}
"#;

        let entries = parse_bibtex(input).unwrap();
        let entry = entries.get("dragon2006").unwrap();
        assert_eq!(entry.authors.len(), 4);
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
% This is a comment
@article{test,
    title = {Test}
}
% Another comment
"#;

        let entries = parse_bibtex(input).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_clean_bibtex_value() {
        assert_eq!(clean_bibtex_value("{DNA} Sequencing"), "DNA Sequencing");
        assert_eq!(clean_bibtex_value("The {Art} of Programming"), "The Art of Programming");
    }
}
