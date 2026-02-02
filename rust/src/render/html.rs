//! HTML renderer for resolved documents.

use crate::ast::{
    Alignment, BibEntry, Block, Citation, CitationStyle, DescriptionItem, EnvironmentKind,
    FootnoteKind, Inline, ResolvedDocument,
};
use crate::error::Result;
use crate::render::math::{create_renderer, MathBackend, MathRenderer};
use crate::resolve::citations::get_citation_order;
use crate::resolve::references::label_to_id;

/// Configuration for HTML rendering.
#[derive(Debug, Clone)]
pub struct HtmlConfig {
    /// Math rendering backend.
    pub math_backend: MathBackend,
    /// Whether to generate a complete HTML document or just the body content.
    pub standalone: bool,
    /// Document title (for standalone mode).
    pub title: Option<String>,
    /// Additional CSS to include.
    pub custom_css: Option<String>,
    /// Whether to include a table of contents.
    pub include_toc: bool,
    /// CSS class prefix for styling.
    pub class_prefix: String,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            math_backend: MathBackend::KaTeX,
            standalone: false,
            title: None,
            custom_css: None,
            include_toc: true,
            class_prefix: "mda".to_string(),
        }
    }
}

/// Render a resolved document to HTML.
pub fn render_html(doc: &ResolvedDocument, config: &HtmlConfig) -> Result<String> {
    let mut renderer = HtmlRenderer::new(doc, config);
    renderer.render()
}

struct HtmlRenderer<'a> {
    doc: &'a ResolvedDocument,
    config: &'a HtmlConfig,
    math: Box<dyn MathRenderer>,
    output: String,
    footnote_counter: u32,
}

impl<'a> HtmlRenderer<'a> {
    fn new(doc: &'a ResolvedDocument, config: &'a HtmlConfig) -> Self {
        Self {
            doc,
            config,
            math: create_renderer(config.math_backend),
            output: String::new(),
            footnote_counter: 0,
        }
    }

    fn render(&mut self) -> Result<String> {
        if self.config.standalone {
            self.render_standalone()
        } else {
            self.render_body()
        }
    }

    fn render_standalone(&mut self) -> Result<String> {
        let title = self
            .config
            .title
            .clone()
            .or_else(|| self.doc.document.metadata.title.clone())
            .unwrap_or_else(|| "Document".to_string());

        self.output.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        self.output.push_str("<meta charset=\"UTF-8\">\n");
        self.output.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        self.output.push_str(&format!("<title>{}</title>\n", escape_html(&title)));

        // Math head content
        if let Some(head) = self.math.head_content() {
            self.output.push_str(&head);
            self.output.push('\n');
        }

        // Default styles
        self.output.push_str(&self.default_styles());

        // Custom CSS
        if let Some(ref css) = self.config.custom_css {
            self.output.push_str("<style>\n");
            self.output.push_str(css);
            self.output.push_str("\n</style>\n");
        }

        self.output.push_str("</head>\n<body>\n");
        self.output.push_str("<article class=\"mdlatex-document\">\n");

        self.render_body_content()?;

        self.output.push_str("</article>\n");
        self.output.push_str("</body>\n</html>");

        Ok(std::mem::take(&mut self.output))
    }

    fn render_body(&mut self) -> Result<String> {
        self.render_body_content()?;
        Ok(std::mem::take(&mut self.output))
    }

    fn render_body_content(&mut self) -> Result<()> {
        for block in &self.doc.document.blocks {
            self.render_block(block)?;
        }

        // Render footnotes section if any
        if !self.doc.footnotes.is_empty() {
            self.render_footnotes_section()?;
        }

        // Render bibliography if citations exist
        if !self.doc.citations.is_empty() {
            self.render_bibliography()?;
        }

        Ok(())
    }

    fn render_block(&mut self, block: &Block) -> Result<()> {
        match block {
            Block::Paragraph(inlines) => {
                self.output.push_str("<p>");
                self.render_inlines(inlines)?;
                self.output.push_str("</p>\n");
            }
            Block::Heading { level, content, label } => {
                let tag = format!("h{}", level);
                let id = label.as_ref().map(|l| label_to_id(l));

                self.output.push('<');
                self.output.push_str(&tag);
                if let Some(ref id) = id {
                    self.output.push_str(&format!(r#" id="{}""#, id));
                }
                self.output.push('>');

                // Add section number if available
                if let Some(ref lbl) = label {
                    if let Some(num) = self.doc.section_numbers.get(lbl) {
                        self.output.push_str(&format!(
                            r#"<span class="{}section-number">{}</span> "#,
                            self.config.class_prefix, num
                        ));
                    }
                }

                self.render_inlines(content)?;

                self.output.push_str("</");
                self.output.push_str(&tag);
                self.output.push_str(">\n");
            }
            Block::CodeBlock { language, content } => {
                self.output.push_str("<pre><code");
                if let Some(lang) = language {
                    self.output.push_str(&format!(r#" class="language-{}""#, lang));
                }
                self.output.push('>');
                self.output.push_str(&escape_html(content));
                self.output.push_str("</code></pre>\n");
            }
            Block::ThematicBreak => {
                self.output.push_str("<hr>\n");
            }
            Block::BlockQuote(blocks) => {
                self.output.push_str("<blockquote>\n");
                for block in blocks {
                    self.render_block(block)?;
                }
                self.output.push_str("</blockquote>\n");
            }
            Block::List { ordered, start, items } => {
                if *ordered {
                    self.output.push_str("<ol");
                    if let Some(start) = start {
                        if *start != 1 {
                            self.output.push_str(&format!(r#" start="{}""#, start));
                        }
                    }
                    self.output.push_str(">\n");
                } else {
                    self.output.push_str("<ul>\n");
                }

                for item in items {
                    self.output.push_str("<li>");
                    if let Some(checked) = item.checked {
                        let checkbox = if checked {
                            r#"<input type="checkbox" checked disabled> "#
                        } else {
                            r#"<input type="checkbox" disabled> "#
                        };
                        self.output.push_str(checkbox);
                    }
                    for (i, block) in item.content.iter().enumerate() {
                        // Inline single paragraphs in list items
                        if item.content.len() == 1 {
                            if let Block::Paragraph(inlines) = block {
                                self.render_inlines(inlines)?;
                                continue;
                            }
                        }
                        self.render_block(block)?;
                    }
                    self.output.push_str("</li>\n");
                }

                if *ordered {
                    self.output.push_str("</ol>\n");
                } else {
                    self.output.push_str("</ul>\n");
                }
            }
            Block::DisplayMath { content, label } => {
                let id = label.as_ref().map(|l| label_to_id(l));

                self.output.push_str(&format!(
                    r#"<div class="{}equation""#,
                    self.config.class_prefix
                ));
                if let Some(ref id) = id {
                    self.output.push_str(&format!(r#" id="{}""#, id));
                }
                self.output.push_str(">\n");

                let rendered = self.math.render_display(content)?;
                self.output.push_str(&rendered);

                // Equation number
                if let Some(ref lbl) = label {
                    if let Some(num) = self.doc.env_numbers.get(lbl) {
                        self.output.push_str(&format!(
                            r#"<span class="{}equation-number">({})</span>"#,
                            self.config.class_prefix, num
                        ));
                    }
                }

                self.output.push_str("\n</div>\n");
            }
            Block::Environment { kind, label, content, caption } => {
                self.render_environment(kind, label.as_deref(), content, caption.as_deref())?;
            }
            Block::TableOfContents => {
                if self.config.include_toc {
                    self.render_toc()?;
                }
            }
            Block::Table { headers, alignments, rows, label, caption } => {
                self.render_table(headers, alignments, rows, label.as_deref(), caption.as_deref())?;
            }
            Block::RawHtml(html) => {
                self.output.push_str(html);
                self.output.push('\n');
            }
            Block::DescriptionList(items) => {
                self.render_description_list(items)?;
            }
            Block::PageBreak => {
                self.output.push_str(&format!(
                    r#"<div class="{}pagebreak" style="page-break-after: always;"></div>"#,
                    self.config.class_prefix
                ));
                self.output.push('\n');
            }
            Block::Abstract(blocks) => {
                self.output.push_str(&format!(
                    r#"<div class="{}abstract">"#,
                    self.config.class_prefix
                ));
                self.output.push_str(&format!(
                    r#"<h2 class="{}abstract-title">Abstract</h2>"#,
                    self.config.class_prefix
                ));
                self.output.push('\n');
                for block in blocks {
                    self.render_block(block)?;
                }
                self.output.push_str("</div>\n");
            }
            Block::AppendixMarker => {
                self.output.push_str(&format!(
                    r#"<div class="{}appendix-marker">"#,
                    self.config.class_prefix
                ));
                self.output.push_str(&format!(
                    r#"<h1 class="{}appendix-title">Appendices</h1>"#,
                    self.config.class_prefix
                ));
                self.output.push_str("</div>\n");
            }
        }

        Ok(())
    }

    fn render_description_list(&mut self, items: &[DescriptionItem]) -> Result<()> {
        self.output.push_str("<dl>\n");
        for item in items {
            self.output.push_str("<dt>");
            self.render_inlines(&item.term)?;
            self.output.push_str("</dt>\n");
            self.output.push_str("<dd>");
            for block in &item.description {
                self.render_block(block)?;
            }
            self.output.push_str("</dd>\n");
        }
        self.output.push_str("</dl>\n");
        Ok(())
    }

    fn render_environment(
        &mut self,
        kind: &EnvironmentKind,
        label: Option<&str>,
        content: &[Block],
        caption: Option<&[Inline]>,
    ) -> Result<()> {
        let id = label.map(label_to_id);
        let class = match kind {
            EnvironmentKind::Proof => "proof",
            EnvironmentKind::Figure => "figure",
            EnvironmentKind::Table => "table",
            _ => "theorem-like",
        };

        // Use figure element for figures
        let tag = if matches!(kind, EnvironmentKind::Figure) {
            "figure"
        } else {
            "div"
        };

        self.output.push_str(&format!(
            r#"<{} class="{}{} {}{}""#,
            tag,
            self.config.class_prefix,
            class,
            self.config.class_prefix,
            kind.display_name().to_lowercase()
        ));
        if let Some(ref id) = id {
            self.output.push_str(&format!(r#" id="{}""#, id));
        }
        self.output.push_str(">\n");

        // Header with name and number
        if kind.is_numbered() {
            self.output.push_str(&format!(
                r#"<span class="{}env-header">"#,
                self.config.class_prefix
            ));
            self.output.push_str(&format!("<strong>{}</strong>", kind.display_name()));
            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    self.output.push_str(&format!(" {}", num));
                }
            }
            self.output.push_str(".</span>\n");
        } else if matches!(kind, EnvironmentKind::Proof) {
            self.output.push_str(&format!(
                r#"<span class="{}env-header"><em>Proof.</em></span>"#,
                self.config.class_prefix
            ));
        }

        // Content
        self.output.push_str(&format!(
            r#"<div class="{}env-content">"#,
            self.config.class_prefix
        ));
        for block in content {
            self.render_block(block)?;
        }
        self.output.push_str("</div>\n");

        // Caption for figures
        if let Some(caption) = caption {
            self.output.push_str("<figcaption>");
            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    self.output.push_str(&format!("<strong>{} {}:</strong> ", kind.display_name(), num));
                }
            }
            self.render_inlines(caption)?;
            self.output.push_str("</figcaption>\n");
        }

        // QED symbol for proofs
        if matches!(kind, EnvironmentKind::Proof) {
            self.output.push_str(&format!(
                r#"<span class="{}qed">∎</span>"#,
                self.config.class_prefix
            ));
        }

        self.output.push_str(&format!("</{}>\n", tag));

        Ok(())
    }

    fn render_table(
        &mut self,
        headers: &[Vec<Inline>],
        alignments: &[Alignment],
        rows: &[Vec<Vec<Inline>>],
        label: Option<&str>,
        caption: Option<&[Inline]>,
    ) -> Result<()> {
        let id = label.map(label_to_id);

        self.output.push_str(&format!(
            r#"<table class="{}table""#,
            self.config.class_prefix
        ));
        if let Some(ref id) = id {
            self.output.push_str(&format!(r#" id="{}""#, id));
        }
        self.output.push_str(">\n");

        // Caption
        if let Some(caption) = caption {
            self.output.push_str("<caption>");
            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    self.output.push_str(&format!("<strong>Table {}:</strong> ", num));
                }
            }
            self.render_inlines(caption)?;
            self.output.push_str("</caption>\n");
        }

        // Header
        self.output.push_str("<thead>\n<tr>\n");
        for (i, cell) in headers.iter().enumerate() {
            let align = alignments.get(i).copied().unwrap_or_default();
            let style = alignment_style(align);
            self.output.push_str(&format!("<th{}>", style));
            self.render_inlines(cell)?;
            self.output.push_str("</th>\n");
        }
        self.output.push_str("</tr>\n</thead>\n");

        // Body
        self.output.push_str("<tbody>\n");
        for row in rows {
            self.output.push_str("<tr>\n");
            for (i, cell) in row.iter().enumerate() {
                let align = alignments.get(i).copied().unwrap_or_default();
                let style = alignment_style(align);
                self.output.push_str(&format!("<td{}>", style));
                self.render_inlines(cell)?;
                self.output.push_str("</td>\n");
            }
            self.output.push_str("</tr>\n");
        }
        self.output.push_str("</tbody>\n");

        self.output.push_str("</table>\n");

        Ok(())
    }

    fn render_toc(&mut self) -> Result<()> {
        self.output.push_str(&format!(
            r#"<nav class="{}toc">"#,
            self.config.class_prefix
        ));
        self.output.push_str("<h2>Table of Contents</h2>\n<ul>\n");

        let mut current_level = 0u8;

        for block in &self.doc.document.blocks {
            if let Block::Heading { level, content, label } = block {
                // Adjust nesting
                while current_level < *level {
                    self.output.push_str("<ul>\n");
                    current_level += 1;
                }
                while current_level > *level {
                    self.output.push_str("</ul>\n");
                    current_level -= 1;
                }

                self.output.push_str("<li>");
                if let Some(lbl) = label {
                    let id = label_to_id(lbl);
                    self.output.push_str(&format!("<a href=\"#{}\">", id));
                    if let Some(num) = self.doc.section_numbers.get(lbl) {
                        self.output.push_str(&format!("{}. ", num));
                    }
                    self.render_inlines(content)?;
                    self.output.push_str("</a>");
                } else {
                    self.render_inlines(content)?;
                }
                self.output.push_str("</li>\n");
            }
        }

        // Close remaining lists
        while current_level > 0 {
            self.output.push_str("</ul>\n");
            current_level -= 1;
        }

        self.output.push_str("</ul>\n</nav>\n");

        Ok(())
    }

    fn render_inlines(&mut self, inlines: &[Inline]) -> Result<()> {
        for inline in inlines {
            self.render_inline(inline)?;
        }
        Ok(())
    }

    fn render_inline(&mut self, inline: &Inline) -> Result<()> {
        match inline {
            Inline::Text(text) => {
                self.output.push_str(&escape_html(text));
            }
            Inline::Emphasis(inlines) => {
                self.output.push_str("<em>");
                self.render_inlines(inlines)?;
                self.output.push_str("</em>");
            }
            Inline::Strong(inlines) => {
                self.output.push_str("<strong>");
                self.render_inlines(inlines)?;
                self.output.push_str("</strong>");
            }
            Inline::Strikethrough(inlines) => {
                self.output.push_str("<del>");
                self.render_inlines(inlines)?;
                self.output.push_str("</del>");
            }
            Inline::Subscript(inlines) => {
                self.output.push_str("<sub>");
                self.render_inlines(inlines)?;
                self.output.push_str("</sub>");
            }
            Inline::Superscript(inlines) => {
                self.output.push_str("<sup>");
                self.render_inlines(inlines)?;
                self.output.push_str("</sup>");
            }
            Inline::SmallCaps(inlines) => {
                self.output.push_str(&format!(
                    r#"<span style="font-variant: small-caps;" class="{}smallcaps">"#,
                    self.config.class_prefix
                ));
                self.render_inlines(inlines)?;
                self.output.push_str("</span>");
            }
            Inline::Code(code) => {
                self.output.push_str("<code>");
                self.output.push_str(&escape_html(code));
                self.output.push_str("</code>");
            }
            Inline::Link { url, title, content } => {
                self.output.push_str(&format!(r#"<a href="{}""#, escape_html(url)));
                if let Some(title) = title {
                    self.output.push_str(&format!(r#" title="{}""#, escape_html(title)));
                }
                self.output.push('>');
                self.render_inlines(content)?;
                self.output.push_str("</a>");
            }
            Inline::Image { url, alt, title } => {
                self.output.push_str(&format!(
                    r#"<img src="{}" alt="{}""#,
                    escape_html(url),
                    escape_html(alt)
                ));
                if let Some(title) = title {
                    self.output.push_str(&format!(r#" title="{}""#, escape_html(title)));
                }
                self.output.push_str(">");
            }
            Inline::InlineMath(latex) => {
                let rendered = self.math.render_inline(latex)?;
                self.output.push_str(&rendered);
            }
            Inline::Citation(cite) => {
                self.render_citation(cite)?;
            }
            Inline::Reference { label, resolved } => {
                let id = label_to_id(label);
                let text = resolved.as_deref().unwrap_or("??");
                self.output.push_str(&format!(
                    "<a href=\"#{}\" class=\"{}ref\">{}</a>",
                    id, self.config.class_prefix, escape_html(text)
                ));
            }
            Inline::Footnote(kind) => {
                self.render_footnote(kind)?;
            }
            Inline::SoftBreak => {
                self.output.push('\n');
            }
            Inline::HardBreak => {
                self.output.push_str("<br>\n");
            }
            Inline::RawHtml(html) => {
                self.output.push_str(html);
            }
        }

        Ok(())
    }

    fn render_citation(&mut self, cite: &Citation) -> Result<()> {
        self.output.push_str(&format!(
            r#"<span class="{}citation">"#,
            self.config.class_prefix
        ));

        match cite.style {
            CitationStyle::Parenthetical => {
                // (Author, Year) or [Author, Year]
                self.output.push('[');
                for (i, key) in cite.keys.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str("; ");
                    }
                    let id = format!("bib-{}", key);
                    if let Some(entry) = self.doc.citations.get(key) {
                        let short = format_short_citation(entry);
                        self.output.push_str(&format!("<a href=\"#{}\">{}</a>", id, escape_html(&short)));
                    } else {
                        self.output.push_str(&format!("<a href=\"#{}\">{}</a>", id, key));
                    }
                }
                if let Some(ref locator) = cite.locator {
                    self.output.push_str(&format!(", {}", escape_html(locator)));
                }
                self.output.push(']');
            }
            CitationStyle::Textual => {
                // Author (Year)
                for (i, key) in cite.keys.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let id = format!("bib-{}", key);
                    if let Some(entry) = self.doc.citations.get(key) {
                        let (author, year) = format_author_year(entry);
                        self.output.push_str(&format!(
                            "{} (<a href=\"#{}\">{}</a>)",
                            escape_html(&author),
                            id,
                            escape_html(&year)
                        ));
                    } else {
                        self.output.push_str(&format!("<a href=\"#{}\">{}</a>", id, key));
                    }
                }
                if let Some(ref locator) = cite.locator {
                    self.output.push_str(&format!(", {}", escape_html(locator)));
                }
            }
            CitationStyle::AuthorOnly => {
                // Just Author
                for (i, key) in cite.keys.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let id = format!("bib-{}", key);
                    if let Some(entry) = self.doc.citations.get(key) {
                        let (author, _) = format_author_year(entry);
                        self.output.push_str(&format!(
                            "<a href=\"#{}\">{}</a>",
                            id,
                            escape_html(&author)
                        ));
                    } else {
                        self.output.push_str(&format!("<a href=\"#{}\">{}</a>", id, key));
                    }
                }
            }
            CitationStyle::YearOnly => {
                // Just (Year)
                self.output.push('(');
                for (i, key) in cite.keys.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let id = format!("bib-{}", key);
                    if let Some(entry) = self.doc.citations.get(key) {
                        let (_, year) = format_author_year(entry);
                        self.output.push_str(&format!(
                            "<a href=\"#{}\">{}</a>",
                            id,
                            escape_html(&year)
                        ));
                    } else {
                        self.output.push_str(&format!("<a href=\"#{}\">{}</a>", id, key));
                    }
                }
                if let Some(ref locator) = cite.locator {
                    self.output.push_str(&format!(", {}", escape_html(locator)));
                }
                self.output.push(')');
            }
        }

        self.output.push_str("</span>");

        Ok(())
    }

    fn render_footnote(&mut self, kind: &FootnoteKind) -> Result<()> {
        self.footnote_counter += 1;
        let num = self.footnote_counter;
        let id = format!("fn-{}", num);
        let back_id = format!("fnref-{}", num);

        self.output.push_str(&format!(
            "<sup id=\"{}\" class=\"{}footnote-ref\"><a href=\"#{}\">[{}]</a></sup>",
            back_id, self.config.class_prefix, id, num
        ));

        Ok(())
    }

    fn render_footnotes_section(&mut self) -> Result<()> {
        self.output.push_str(&format!(
            r#"<section class="{}footnotes">"#,
            self.config.class_prefix
        ));
        self.output.push_str("<hr>\n<ol>\n");

        let mut counter = 0u32;
        for block in &self.doc.document.blocks {
            self.render_block_footnotes(block, &mut counter)?;
        }

        self.output.push_str("</ol>\n</section>\n");

        Ok(())
    }

    fn render_block_footnotes(&mut self, block: &Block, counter: &mut u32) -> Result<()> {
        match block {
            Block::Paragraph(inlines) => self.render_inline_footnotes(inlines, counter)?,
            Block::Heading { content, .. } => self.render_inline_footnotes(content, counter)?,
            Block::Environment { content, caption, .. } => {
                for b in content {
                    self.render_block_footnotes(b, counter)?;
                }
                if let Some(c) = caption {
                    self.render_inline_footnotes(c, counter)?;
                }
            }
            Block::BlockQuote(blocks) => {
                for b in blocks {
                    self.render_block_footnotes(b, counter)?;
                }
            }
            Block::List { items, .. } => {
                for item in items {
                    for b in &item.content {
                        self.render_block_footnotes(b, counter)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn render_inline_footnotes(&mut self, inlines: &[Inline], counter: &mut u32) -> Result<()> {
        for inline in inlines {
            match inline {
                Inline::Footnote(FootnoteKind::Inline(content)) => {
                    *counter += 1;
                    let id = format!("fn-{}", counter);
                    let back_id = format!("fnref-{}", counter);

                    self.output.push_str(&format!("<li id=\"{}\">", id));
                    self.render_inlines(content)?;
                    self.output.push_str(&format!(
                        " <a href=\"#{}\" class=\"{}footnote-back\">↩</a></li>",
                        back_id, self.config.class_prefix
                    ));
                    self.output.push('\n');
                }
                Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                    self.render_inline_footnotes(inner, counter)?;
                }
                Inline::Link { content, .. } => {
                    self.render_inline_footnotes(content, counter)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render_bibliography(&mut self) -> Result<()> {
        let order = get_citation_order(&self.doc.document);

        if order.is_empty() {
            return Ok(());
        }

        self.output.push_str(&format!(
            r#"<section class="{}bibliography">"#,
            self.config.class_prefix
        ));
        self.output.push_str("<h2>References</h2>\n<ol>\n");

        for key in order {
            if let Some(entry) = self.doc.citations.get(&key) {
                let id = format!("bib-{}", key);
                self.output.push_str(&format!(r#"<li id="{}">"#, id));
                self.output.push_str(&format_bibliography_entry(entry));
                self.output.push_str("</li>\n");
            }
        }

        self.output.push_str("</ol>\n</section>\n");

        Ok(())
    }

    fn default_styles(&self) -> String {
        format!(
            r#"<style>
.{p}document {{ max-width: 800px; margin: 0 auto; padding: 2em; font-family: Georgia, serif; line-height: 1.6; }}
.{p}section-number {{ color: #666; margin-right: 0.5em; }}
.{p}equation {{ display: flex; align-items: center; justify-content: space-between; margin: 1em 0; }}
.{p}equation-number {{ color: #666; }}
.{p}theorem-like {{ margin: 1.5em 0; padding: 1em; background: #f8f8f8; border-left: 3px solid #333; }}
.{p}proof {{ margin: 1em 0; padding: 1em; font-style: italic; }}
.{p}qed {{ float: right; }}
.{p}figure {{ margin: 2em 0; text-align: center; }}
.{p}figure img {{ max-width: 100%; }}
.{p}table {{ border-collapse: collapse; margin: 1em auto; }}
.{p}table th, .{p}table td {{ border: 1px solid #ddd; padding: 0.5em 1em; }}
.{p}table th {{ background: #f0f0f0; }}
.{p}toc {{ background: #fafafa; padding: 1em 2em; margin: 2em 0; border-radius: 4px; }}
.{p}toc ul {{ list-style: none; padding-left: 1.5em; }}
.{p}toc > ul {{ padding-left: 0; }}
.{p}citation {{ }}
.{p}ref {{ color: #0066cc; text-decoration: none; }}
.{p}ref:hover {{ text-decoration: underline; }}
.{p}footnotes {{ font-size: 0.9em; color: #666; }}
.{p}footnote-ref {{ font-size: 0.8em; }}
.{p}bibliography {{ margin-top: 3em; }}
.{p}bibliography ol {{ padding-left: 2em; }}
.{p}env-header {{ font-weight: bold; }}
.{p}env-content {{ margin-top: 0.5em; }}
</style>
"#,
            p = self.config.class_prefix
        )
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn alignment_style(align: Alignment) -> &'static str {
    match align {
        Alignment::Left => "",
        Alignment::Center => r#" style="text-align: center""#,
        Alignment::Right => r#" style="text-align: right""#,
    }
}

fn format_short_citation(entry: &BibEntry) -> String {
    let author = entry
        .authors
        .first()
        .map(|a| {
            // Extract last name
            if let Some(comma) = a.find(',') {
                &a[..comma]
            } else if let Some(space) = a.rfind(' ') {
                &a[space + 1..]
            } else {
                a.as_str()
            }
        })
        .unwrap_or("Unknown");

    let year = entry.year.as_deref().unwrap_or("n.d.");

    if entry.authors.len() > 2 {
        format!("{} et al., {}", author, year)
    } else if entry.authors.len() == 2 {
        let author2 = entry.authors.get(1).map(|a| {
            if let Some(comma) = a.find(',') {
                &a[..comma]
            } else if let Some(space) = a.rfind(' ') {
                &a[space + 1..]
            } else {
                a.as_str()
            }
        }).unwrap_or("");
        format!("{} & {}, {}", author, author2, year)
    } else {
        format!("{}, {}", author, year)
    }
}

/// Format author and year separately for textual citations.
fn format_author_year(entry: &BibEntry) -> (String, String) {
    let author = if entry.authors.len() > 2 {
        let first = entry.authors.first().map(|a| {
            if let Some(comma) = a.find(',') {
                &a[..comma]
            } else if let Some(space) = a.rfind(' ') {
                &a[space + 1..]
            } else {
                a.as_str()
            }
        }).unwrap_or("Unknown");
        format!("{} et al.", first)
    } else if entry.authors.len() == 2 {
        let first = entry.authors.first().map(|a| {
            if let Some(comma) = a.find(',') {
                &a[..comma]
            } else if let Some(space) = a.rfind(' ') {
                &a[space + 1..]
            } else {
                a.as_str()
            }
        }).unwrap_or("Unknown");
        let second = entry.authors.get(1).map(|a| {
            if let Some(comma) = a.find(',') {
                &a[..comma]
            } else if let Some(space) = a.rfind(' ') {
                &a[space + 1..]
            } else {
                a.as_str()
            }
        }).unwrap_or("Unknown");
        format!("{} & {}", first, second)
    } else {
        entry.authors.first().map(|a| {
            if let Some(comma) = a.find(',') {
                a[..comma].to_string()
            } else if let Some(space) = a.rfind(' ') {
                a[space + 1..].to_string()
            } else {
                a.to_string()
            }
        }).unwrap_or_else(|| "Unknown".to_string())
    };

    let year = entry.year.as_deref().unwrap_or("n.d.").to_string();

    (author, year)
}

fn format_bibliography_entry(entry: &BibEntry) -> String {
    let mut parts = Vec::new();

    // Authors
    if !entry.authors.is_empty() {
        parts.push(entry.authors.join(", "));
    }

    // Year
    if let Some(ref year) = entry.year {
        parts.push(format!("({})", year));
    }

    // Title
    if let Some(ref title) = entry.title {
        parts.push(format!("<em>{}</em>", escape_html(title)));
    }

    // Journal/Book
    if let Some(ref journal) = entry.journal {
        let mut journal_part = journal.clone();
        if let Some(ref vol) = entry.volume {
            journal_part.push_str(&format!(", {}", vol));
            if let Some(ref num) = entry.number {
                journal_part.push_str(&format!("({})", num));
            }
        }
        if let Some(ref pages) = entry.pages {
            journal_part.push_str(&format!(", {}", pages));
        }
        parts.push(journal_part);
    } else if let Some(ref booktitle) = entry.booktitle {
        parts.push(format!("In <em>{}</em>", escape_html(booktitle)));
    }

    // Publisher
    if let Some(ref publisher) = entry.publisher {
        parts.push(publisher.clone());
    }

    // DOI
    if let Some(ref doi) = entry.doi {
        parts.push(format!(
            r#"<a href="https://doi.org/{}">{}</a>"#,
            doi, doi
        ));
    }

    parts.join(". ") + "."
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::resolve::{resolve, ResolveConfig};

    #[test]
    fn test_render_simple() {
        let input = "# Hello\n\nThis is a paragraph.";
        let doc = parse(input).unwrap();
        let resolved = resolve(doc, &ResolveConfig::default()).unwrap();
        let html = render_html(&resolved, &HtmlConfig::default()).unwrap();

        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_render_math() {
        let input = "Inline $E = mc^2$ math.";
        let doc = parse(input).unwrap();
        let resolved = resolve(doc, &ResolveConfig::default()).unwrap();
        let html = render_html(&resolved, &HtmlConfig::default()).unwrap();

        assert!(html.contains("math inline"));
    }

    #[test]
    fn test_render_standalone() {
        let input = "# Test";
        let doc = parse(input).unwrap();
        let resolved = resolve(doc, &ResolveConfig::default()).unwrap();
        let config = HtmlConfig {
            standalone: true,
            title: Some("Test Doc".to_string()),
            ..Default::default()
        };
        let html = render_html(&resolved, &config).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Doc</title>"));
    }
}
