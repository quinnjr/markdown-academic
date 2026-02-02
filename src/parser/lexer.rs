//! Lexer for tokenizing Markdown source.

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_until, take_while, take_while1},
    character::complete::{char, line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{map, opt, peek, recognize, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

/// A token from the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    // Block-level tokens
    Heading(u8, &'a str),           // Level, content
    FencedCodeStart(&'a str),       // Language
    FencedCodeEnd,
    CodeContent(&'a str),
    ThematicBreak,
    BlockQuoteMarker,
    ListItemMarker(ListMarker),
    EnvironmentStart(&'a str, Option<&'a str>),  // Kind, label
    EnvironmentEnd,
    TableOfContents,
    BlankLine,

    // Inline tokens
    Text(&'a str),
    Emphasis(&'a str),              // * or _
    Strong(&'a str),                // ** or __
    InlineCode(&'a str),
    InlineMath(&'a str),
    DisplayMath(&'a str),
    Citation(Vec<CitationToken<'a>>),
    Reference(&'a str),             // @label
    FootnoteInline(&'a str),        // ^[content]
    FootnoteRef(&'a str),           // [^id]
    Link(&'a str, &'a str, Option<&'a str>), // text, url, title
    Image(&'a str, &'a str, Option<&'a str>), // alt, url, title
    Label(&'a str),                 // {#label}
    SoftBreak,
    HardBreak,
    RawHtml(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CitationToken<'a> {
    pub key: &'a str,
    pub locator: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListMarker {
    Unordered,
    Ordered(u32),
    Checkbox(bool),
}

/// Check if a line is blank.
pub fn is_blank_line(input: &str) -> IResult<&str, ()> {
    value((), pair(space0, line_ending))(input)
}

/// Parse a heading (ATX style: # Heading).
pub fn heading(input: &str) -> IResult<&str, Token> {
    let (input, hashes) = take_while1(|c| c == '#')(input)?;
    let level = hashes.len().min(6) as u8;
    let (input, _) = space1(input)?;
    let (input, content) = not_line_ending(input)?;
    // Trim trailing # and spaces
    let content = content.trim_end_matches(|c| c == '#' || c == ' ');
    Ok((input, Token::Heading(level, content)))
}

/// Parse a thematic break (---, ***, ___).
pub fn thematic_break(input: &str) -> IResult<&str, Token> {
    let (input, _) = alt((
        recognize(tuple((tag("-"), tag("-"), tag("-"), many0(char('-'))))),
        recognize(tuple((tag("*"), tag("*"), tag("*"), many0(char('*'))))),
        recognize(tuple((tag("_"), tag("_"), tag("_"), many0(char('_'))))),
    ))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, Token::ThematicBreak))
}

/// Parse a fenced code block start.
pub fn fenced_code_start(input: &str) -> IResult<&str, Token> {
    let (input, _) = alt((tag("```"), tag("~~~")))(input)?;
    let (input, lang) = opt(take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_'))(input)?;
    let (input, _) = not_line_ending(input)?;
    Ok((input, Token::FencedCodeStart(lang.unwrap_or(""))))
}

/// Parse a fenced code block end.
pub fn fenced_code_end(input: &str) -> IResult<&str, Token> {
    let (input, _) = alt((tag("```"), tag("~~~")))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, Token::FencedCodeEnd))
}

/// Parse an environment start (:::).
pub fn environment_start(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag(":::")(input)?;
    let (input, _) = space0(input)?;
    let (input, kind) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;
    let (input, _) = space0(input)?;
    let (input, label) = opt(delimited(tag("{#"), take_while1(|c: char| c != '}'), tag("}")))(input)?;
    let (input, _) = not_line_ending(input)?;
    Ok((input, Token::EnvironmentStart(kind, label)))
}

/// Parse an environment end.
pub fn environment_end(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag(":::")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = peek(alt((line_ending, recognize(nom::combinator::eof))))(input)?;
    Ok((input, Token::EnvironmentEnd))
}

/// Parse a table of contents marker.
pub fn table_of_contents(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("[[toc]]")(input)?;
    Ok((input, Token::TableOfContents))
}

/// Parse a block quote marker.
pub fn block_quote_marker(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('>')(input)?;
    let (input, _) = opt(char(' '))(input)?;
    Ok((input, Token::BlockQuoteMarker))
}

/// Parse a list item marker.
pub fn list_item_marker(input: &str) -> IResult<&str, Token> {
    alt((
        // Checkbox
        map(
            tuple((
                alt((char('-'), char('*'), char('+'))),
                space1,
                char('['),
                alt((value(true, char('x')), value(true, char('X')), value(false, char(' ')))),
                char(']'),
                space0,
            )),
            |(_, _, _, checked, _, _)| Token::ListItemMarker(ListMarker::Checkbox(checked)),
        ),
        // Unordered
        map(
            tuple((alt((char('-'), char('*'), char('+'))), space1)),
            |_| Token::ListItemMarker(ListMarker::Unordered),
        ),
        // Ordered
        map(
            tuple((
                take_while1(|c: char| c.is_ascii_digit()),
                alt((char('.'), char(')'))),
                space1,
            )),
            |(num, _, _): (&str, _, _)| {
                Token::ListItemMarker(ListMarker::Ordered(num.parse().unwrap_or(1)))
            },
        ),
    ))(input)
}

/// Parse inline math ($...$).
pub fn inline_math(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('$')(input)?;
    let (input, _) = peek(nom::combinator::not(char('$')))(input)?;  // Not display math
    let (input, content) = take_until("$")(input)?;
    let (input, _) = char('$')(input)?;
    Ok((input, Token::InlineMath(content)))
}

/// Parse display math ($$...$$).
pub fn display_math(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("$$")(input)?;
    let (input, content) = take_until("$$")(input)?;
    let (input, _) = tag("$$")(input)?;
    Ok((input, Token::DisplayMath(content)))
}

/// Parse a citation ([@key] or [@key, p. 42]).
pub fn citation(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("[@")(input)?;
    let (input, content) = take_until("]")(input)?;
    let (input, _) = char(']')(input)?;

    // Parse citation content: key1; key2, locator
    let citations: Vec<CitationToken> = content
        .split(';')
        .map(|part| {
            let part = part.trim();
            if let Some((key, locator)) = part.split_once(',') {
                CitationToken {
                    key: key.trim(),
                    locator: Some(locator.trim()),
                }
            } else {
                CitationToken {
                    key: part,
                    locator: None,
                }
            }
        })
        .collect();

    Ok((input, Token::Citation(citations)))
}

/// Parse a cross-reference (@label).
pub fn reference(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('@')(input)?;
    // Ensure it's not a citation
    let (input, _) = peek(nom::combinator::not(char('[')))(input)?;
    let (input, label) = take_while1(|c: char| c.is_alphanumeric() || c == ':' || c == '-' || c == '_')(input)?;
    Ok((input, Token::Reference(label)))
}

/// Parse an inline footnote (^[content]).
pub fn footnote_inline(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("^[")(input)?;
    let (input, content) = take_until("]")(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, Token::FootnoteInline(content)))
}

/// Parse a footnote reference ([^id]).
pub fn footnote_ref(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("[^")(input)?;
    let (input, id) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, Token::FootnoteRef(id)))
}

/// Parse a label ({#label}).
pub fn label(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("{#")(input)?;
    let (input, id) = take_while1(|c: char| c != '}')(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, Token::Label(id)))
}

/// Parse inline code (`code`).
pub fn inline_code(input: &str) -> IResult<&str, Token> {
    let (input, _) = char('`')(input)?;
    let (input, _) = peek(nom::combinator::not(char('`')))(input)?;  // Not fenced code
    let (input, content) = take_until("`")(input)?;
    let (input, _) = char('`')(input)?;
    Ok((input, Token::InlineCode(content)))
}

/// Parse emphasis (*text* or _text_).
pub fn emphasis(input: &str) -> IResult<&str, Token> {
    alt((
        delimited(
            pair(char('*'), peek(nom::combinator::not(char('*')))),
            map(take_until("*"), |s| Token::Emphasis(s)),
            char('*'),
        ),
        delimited(
            pair(char('_'), peek(nom::combinator::not(char('_')))),
            map(take_until("_"), |s| Token::Emphasis(s)),
            char('_'),
        ),
    ))(input)
}

/// Parse strong (**text** or __text__).
pub fn strong(input: &str) -> IResult<&str, Token> {
    alt((
        delimited(
            tag("**"),
            map(take_until("**"), |s| Token::Strong(s)),
            tag("**"),
        ),
        delimited(
            tag("__"),
            map(take_until("__"), |s| Token::Strong(s)),
            tag("__"),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading() {
        assert_eq!(
            heading("# Hello World"),
            Ok(("", Token::Heading(1, "Hello World")))
        );
        assert_eq!(
            heading("### Level 3 ###"),
            Ok(("", Token::Heading(3, "Level 3")))
        );
    }

    #[test]
    fn test_inline_math() {
        assert_eq!(
            inline_math("$E = mc^2$ rest"),
            Ok((" rest", Token::InlineMath("E = mc^2")))
        );
    }

    #[test]
    fn test_display_math() {
        assert_eq!(
            display_math("$$\\int_0^1 x dx$$"),
            Ok(("", Token::DisplayMath("\\int_0^1 x dx")))
        );
    }

    #[test]
    fn test_citation() {
        let result = citation("[@knuth1984]");
        assert!(result.is_ok());
        if let Ok((_, Token::Citation(cites))) = result {
            assert_eq!(cites.len(), 1);
            assert_eq!(cites[0].key, "knuth1984");
        }

        let result = citation("[@knuth1984, p. 42]");
        assert!(result.is_ok());
        if let Ok((_, Token::Citation(cites))) = result {
            assert_eq!(cites[0].locator, Some("p. 42"));
        }
    }

    #[test]
    fn test_reference() {
        assert_eq!(
            reference("@eq:euler"),
            Ok(("", Token::Reference("eq:euler")))
        );
    }

    #[test]
    fn test_environment() {
        assert_eq!(
            environment_start("::: theorem {#thm:main}"),
            Ok(("", Token::EnvironmentStart("theorem", Some("thm:main"))))
        );
    }
}
