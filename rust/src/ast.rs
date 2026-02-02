//! Abstract Syntax Tree definitions for the extended Markdown language.

use std::collections::HashMap;

/// A complete parsed document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Front matter metadata
    pub metadata: Metadata,
    /// Document content as a sequence of blocks
    pub blocks: Vec<Block>,
}

/// Document metadata from TOML front matter.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metadata {
    /// User-defined LaTeX macros
    pub macros: HashMap<String, Macro>,
    /// Path to bibliography file
    pub bibliography_path: Option<String>,
    /// Document title
    pub title: Option<String>,
    /// Document subtitle
    pub subtitle: Option<String>,
    /// Document author(s)
    pub authors: Vec<String>,
    /// Document date
    pub date: Option<String>,
    /// Document abstract
    pub document_abstract: Option<String>,
    /// Keywords for the document
    pub keywords: Vec<String>,
    /// Institution (for academic documents)
    pub institution: Option<String>,
    /// Department
    pub department: Option<String>,
    /// Advisor/supervisor
    pub advisor: Option<String>,
    /// Document language
    pub lang: Option<String>,
}

/// A user-defined macro.
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    /// Number of arguments (0 for simple substitution)
    pub arg_count: usize,
    /// Replacement template (use #1, #2, etc. for args)
    pub template: String,
}

/// Block-level elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A paragraph of inline content
    Paragraph(Vec<Inline>),

    /// A heading with level (1-6), content, and optional label
    Heading {
        level: u8,
        content: Vec<Inline>,
        label: Option<String>,
    },

    /// A fenced code block
    CodeBlock {
        language: Option<String>,
        content: String,
    },

    /// A block quote
    BlockQuote(Vec<Block>),

    /// An ordered or unordered list
    List {
        ordered: bool,
        start: Option<u32>,
        items: Vec<ListItem>,
    },

    /// A thematic break (horizontal rule)
    ThematicBreak,

    /// Display math block
    DisplayMath {
        content: String,
        label: Option<String>,
    },

    /// A custom environment (theorem, proof, figure, etc.)
    Environment {
        kind: EnvironmentKind,
        label: Option<String>,
        content: Vec<Block>,
        caption: Option<Vec<Inline>>,
    },

    /// Table of contents placeholder
    TableOfContents,

    /// Raw HTML passthrough
    RawHtml(String),

    /// A table
    Table {
        headers: Vec<Vec<Inline>>,
        alignments: Vec<Alignment>,
        rows: Vec<Vec<Vec<Inline>>>,
        label: Option<String>,
        caption: Option<Vec<Inline>>,
    },

    /// A description list (definition list)
    DescriptionList(Vec<DescriptionItem>),

    /// A page break / section break
    PageBreak,

    /// An abstract section
    Abstract(Vec<Block>),

    /// An appendix marker (changes section numbering to letters)
    AppendixMarker,
}

/// List item containing blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub content: Vec<Block>,
    pub checked: Option<bool>,
}

/// A description list item (term and definition).
#[derive(Debug, Clone, PartialEq)]
pub struct DescriptionItem {
    /// The term being defined
    pub term: Vec<Inline>,
    /// The definition/description
    pub description: Vec<Block>,
}

/// Environment types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentKind {
    Theorem,
    Lemma,
    Proposition,
    Corollary,
    Definition,
    Example,
    Remark,
    Proof,
    Figure,
    Table,
    Algorithm,
    /// Abstract environment
    Abstract,
    /// Note environment
    Note,
    /// Warning/caution environment
    Warning,
    /// Quote environment (extended block quote)
    Quote,
    /// Conjecture
    Conjecture,
    /// Axiom
    Axiom,
    /// Exercise
    Exercise,
    /// Solution
    Solution,
    /// Case (for proof cases)
    Case,
    /// Custom environment with user-defined name
    Custom(String),
}

impl EnvironmentKind {
    /// Parse an environment kind from a string.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "theorem" | "thm" => Self::Theorem,
            "lemma" | "lem" => Self::Lemma,
            "proposition" | "prop" => Self::Proposition,
            "corollary" | "cor" => Self::Corollary,
            "definition" | "def" => Self::Definition,
            "example" | "ex" => Self::Example,
            "remark" | "rem" => Self::Remark,
            "proof" | "pf" => Self::Proof,
            "figure" | "fig" => Self::Figure,
            "table" | "tab" => Self::Table,
            "algorithm" | "algo" => Self::Algorithm,
            "abstract" | "abs" => Self::Abstract,
            "note" => Self::Note,
            "warning" | "caution" => Self::Warning,
            "quote" | "blockquote" => Self::Quote,
            "conjecture" | "conj" => Self::Conjecture,
            "axiom" | "ax" => Self::Axiom,
            "exercise" => Self::Exercise,
            "solution" | "sol" => Self::Solution,
            "case" => Self::Case,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Get the display name for this environment.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Theorem => "Theorem",
            Self::Lemma => "Lemma",
            Self::Proposition => "Proposition",
            Self::Corollary => "Corollary",
            Self::Definition => "Definition",
            Self::Example => "Example",
            Self::Remark => "Remark",
            Self::Proof => "Proof",
            Self::Figure => "Figure",
            Self::Table => "Table",
            Self::Algorithm => "Algorithm",
            Self::Abstract => "Abstract",
            Self::Note => "Note",
            Self::Warning => "Warning",
            Self::Quote => "Quote",
            Self::Conjecture => "Conjecture",
            Self::Axiom => "Axiom",
            Self::Exercise => "Exercise",
            Self::Solution => "Solution",
            Self::Case => "Case",
            Self::Custom(name) => name,
        }
    }

    /// Check if this environment should be numbered.
    pub fn is_numbered(&self) -> bool {
        !matches!(
            self,
            Self::Proof | Self::Abstract | Self::Note | Self::Warning | Self::Quote | Self::Case
        )
    }
}

/// Table column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Inline-level elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text
    Text(String),

    /// Emphasized text (italic)
    Emphasis(Vec<Inline>),

    /// Strong text (bold)
    Strong(Vec<Inline>),

    /// Strikethrough text
    Strikethrough(Vec<Inline>),

    /// Subscript text (e.g., H~2~O)
    Subscript(Vec<Inline>),

    /// Superscript text (e.g., x^2^ outside math mode)
    Superscript(Vec<Inline>),

    /// Small caps text
    SmallCaps(Vec<Inline>),

    /// Inline code
    Code(String),

    /// A link
    Link {
        url: String,
        title: Option<String>,
        content: Vec<Inline>,
    },

    /// An image
    Image {
        url: String,
        alt: String,
        title: Option<String>,
    },

    /// Inline math
    InlineMath(String),

    /// A citation reference
    Citation(Citation),

    /// A cross-reference
    Reference {
        label: String,
        /// Resolved text (filled in during resolution)
        resolved: Option<String>,
    },

    /// An inline footnote
    Footnote(FootnoteKind),

    /// A soft line break
    SoftBreak,

    /// A hard line break
    HardBreak,

    /// Raw HTML inline
    RawHtml(String),
}

/// Citation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CitationStyle {
    /// Parenthetical: (Author, Year) - default with [@key]
    #[default]
    Parenthetical,
    /// Textual: Author (Year) - with @key
    Textual,
    /// Author only: Author - with @key-
    AuthorOnly,
    /// Year only: (Year) - with [-@key]
    YearOnly,
}

/// Citation with optional locator.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    /// Citation keys
    pub keys: Vec<String>,
    /// Citation style
    pub style: CitationStyle,
    /// Optional prefix (e.g., "see")
    pub prefix: Option<String>,
    /// Optional suffix/locator (e.g., "p. 42")
    pub locator: Option<String>,
}

/// Footnote variants.
#[derive(Debug, Clone, PartialEq)]
pub enum FootnoteKind {
    /// Inline footnote with direct content
    Inline(Vec<Inline>),
    /// Reference to a footnote defined elsewhere
    Reference(String),
}

/// A resolved document with all references linked.
#[derive(Debug, Clone)]
pub struct ResolvedDocument {
    pub document: Document,
    /// Resolved labels -> (display text, target id)
    pub labels: HashMap<String, LabelInfo>,
    /// Resolved citations
    pub citations: HashMap<String, BibEntry>,
    /// Footnote contents (id -> content)
    pub footnotes: HashMap<String, Vec<Inline>>,
    /// Section numbering
    pub section_numbers: HashMap<String, String>,
    /// Environment numbering (label -> number)
    pub env_numbers: HashMap<String, u32>,
}

/// Information about a label target.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelInfo {
    /// The display text for references (e.g., "Theorem 1", "Figure 2")
    pub display: String,
    /// The HTML id for linking
    pub html_id: String,
}

/// A bibliography entry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: String,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub booktitle: Option<String>,
    pub publisher: Option<String>,
    pub volume: Option<String>,
    pub number: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
    /// All other fields
    pub extra: HashMap<String, String>,
}
