//! PDF renderer for markdown-academic documents.
//!
//! This module provides PDF generation capabilities using the `genpdf` crate.
//! Enable with the `pdf` feature flag.

#![cfg(feature = "pdf")]

use crate::ast::{Block, EnvironmentKind, FootnoteKind, Inline, ResolvedDocument};
use crate::error::{RenderError, Result};
use genpdf::elements::{Break, Paragraph};
use genpdf::{Document, Element, SimplePageDecorator};
use std::path::Path;

/// Configuration for PDF rendering.
#[derive(Debug, Clone)]
pub struct PdfConfig {
    /// Document title (appears in PDF metadata and optionally as title page).
    pub title: Option<String>,
    /// Document author(s).
    pub authors: Vec<String>,
    /// Paper size: "letter" or "a4".
    pub paper_size: PaperSize,
    /// Font size in points for body text.
    pub font_size: u8,
    /// Line height multiplier.
    pub line_height: f64,
    /// Page margins in millimeters.
    pub margins: PageMargins,
    /// Whether to include a title page.
    pub title_page: bool,
    /// Whether to include page numbers.
    pub page_numbers: bool,
    /// Whether to include table of contents.
    pub include_toc: bool,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            title: None,
            authors: Vec::new(),
            paper_size: PaperSize::Letter,
            font_size: 11,
            line_height: 1.5,
            margins: PageMargins::default(),
            title_page: false,
            page_numbers: true,
            include_toc: true,
        }
    }
}

/// Paper size options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaperSize {
    #[default]
    Letter,
    A4,
}

impl PaperSize {
    fn dimensions(&self) -> (f64, f64) {
        match self {
            PaperSize::Letter => (215.9, 279.4), // 8.5" x 11" in mm
            PaperSize::A4 => (210.0, 297.0),
        }
    }
}

/// Page margins in millimeters.
#[derive(Debug, Clone, Copy)]
pub struct PageMargins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: 25.4,    // 1 inch
            bottom: 25.4,
            left: 25.4,
            right: 25.4,
        }
    }
}

/// Render a resolved document to PDF bytes.
pub fn render_pdf(doc: &ResolvedDocument, config: &PdfConfig) -> Result<Vec<u8>> {
    let renderer = PdfRenderer::new(doc, config)?;
    renderer.render()
}

/// Render a resolved document to a PDF file.
pub fn render_pdf_to_file(
    doc: &ResolvedDocument,
    config: &PdfConfig,
    path: impl AsRef<Path>,
) -> Result<()> {
    let bytes = render_pdf(doc, config)?;
    std::fs::write(path, bytes).map_err(|e| RenderError::Template(e.to_string()))?;
    Ok(())
}

struct PdfRenderer<'a> {
    doc: &'a ResolvedDocument,
    config: &'a PdfConfig,
    footnotes: Vec<(u32, String)>,
    footnote_counter: u32,
}

impl<'a> PdfRenderer<'a> {
    fn new(doc: &'a ResolvedDocument, config: &'a PdfConfig) -> Result<Self> {
        Ok(Self {
            doc,
            config,
            footnotes: Vec::new(),
            footnote_counter: 0,
        })
    }

    fn render(mut self) -> Result<Vec<u8>> {
        // Try to load fonts from various locations
        let font_family = genpdf::fonts::from_files(
            "/usr/share/fonts/liberation",
            "LiberationSerif",
            None,
        )
        .or_else(|_| {
            genpdf::fonts::from_files(
                "/usr/share/fonts/truetype/liberation",
                "LiberationSerif",
                None,
            )
        })
        .or_else(|_| {
            genpdf::fonts::from_files(
                "/usr/share/fonts/TTF",
                "LiberationSerif", 
                None,
            )
        })
        .map_err(|e| RenderError::Template(format!(
            "Could not load fonts. Please install Liberation fonts. Error: {}", e
        )))?;

        let (width, height) = self.config.paper_size.dimensions();
        
        let mut pdf = Document::new(font_family);
        pdf.set_title(self.config.title.clone().unwrap_or_default());
        pdf.set_paper_size(genpdf::Size::new(width, height));
        
        // Add page decorator with margins
        let mut decorator = SimplePageDecorator::new();
        decorator.set_margins(self.config.margins.top as u32);
        pdf.set_page_decorator(decorator);
        
        pdf.set_font_size(self.config.font_size);
        pdf.set_line_spacing(self.config.line_height);

        // Title page
        if self.config.title_page {
            self.render_title_page(&mut pdf)?;
        }

        // Table of contents
        if self.config.include_toc && self.has_toc_placeholder() {
            self.render_toc(&mut pdf)?;
        }

        // Main content
        for block in &self.doc.document.blocks {
            self.render_block(&mut pdf, block)?;
        }

        // Footnotes section
        if !self.footnotes.is_empty() {
            self.render_footnotes_section(&mut pdf)?;
        }

        // Bibliography
        if !self.doc.citations.is_empty() {
            self.render_bibliography(&mut pdf)?;
        }

        // Render to bytes
        let mut buffer = Vec::new();
        pdf.render(&mut buffer)
            .map_err(|e| RenderError::Template(e.to_string()))?;

        Ok(buffer)
    }

    fn has_toc_placeholder(&self) -> bool {
        self.doc
            .document
            .blocks
            .iter()
            .any(|b| matches!(b, Block::TableOfContents))
    }

    fn render_title_page(&self, pdf: &mut Document) -> Result<()> {
        let title = self
            .config
            .title
            .clone()
            .or_else(|| self.doc.document.metadata.title.clone())
            .unwrap_or_else(|| "Untitled Document".to_string());

        pdf.push(Break::new(3.0));
        pdf.push(Paragraph::new(title));
        pdf.push(Break::new(1.0));

        let authors = if !self.config.authors.is_empty() {
            self.config.authors.clone()
        } else {
            self.doc.document.metadata.authors.clone()
        };

        if !authors.is_empty() {
            pdf.push(Paragraph::new(authors.join(", ")));
        }

        if let Some(ref date) = self.doc.document.metadata.date {
            pdf.push(Break::new(0.5));
            pdf.push(Paragraph::new(date.clone()));
        }

        pdf.push(genpdf::elements::PageBreak::new());
        Ok(())
    }

    fn render_toc(&mut self, pdf: &mut Document) -> Result<()> {
        pdf.push(Paragraph::new("Table of Contents"));
        pdf.push(Break::new(0.5));

        for block in &self.doc.document.blocks.clone() {
            if let Block::Heading {
                level,
                content,
                label,
            } = block
            {
                let text = self.inlines_to_string(content);
                let prefix = if let Some(lbl) = label {
                    if let Some(num) = self.doc.section_numbers.get(lbl) {
                        format!("{}  ", num)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let indent = "  ".repeat((*level as usize).saturating_sub(1));
                pdf.push(Paragraph::new(format!("{}{}{}", indent, prefix, text)));
            }
        }

        pdf.push(Break::new(1.0));
        Ok(())
    }

    fn render_block(&mut self, pdf: &mut Document, block: &Block) -> Result<()> {
        match block {
            Block::Paragraph(inlines) => {
                let text = self.inlines_to_string(inlines);
                pdf.push(Paragraph::new(text));
                pdf.push(Break::new(0.3));
            }
            Block::Heading {
                level,
                content,
                label,
            } => {
                pdf.push(Break::new(0.5));

                let text = self.inlines_to_string(content);
                let mut full_text = String::new();

                if let Some(lbl) = label {
                    if let Some(num) = self.doc.section_numbers.get(lbl) {
                        full_text.push_str(&format!("{} ", num));
                    }
                }
                full_text.push_str(&text);

                // Add heading markers based on level
                let marker = match level {
                    1 => "# ",
                    2 => "## ",
                    3 => "### ",
                    _ => "",
                };

                pdf.push(Paragraph::new(format!("{}{}", marker, full_text)));
                pdf.push(Break::new(0.3));
            }
            Block::CodeBlock { content, .. } => {
                pdf.push(Break::new(0.2));
                for line in content.lines() {
                    pdf.push(Paragraph::new(format!("  {}", line)));
                }
                pdf.push(Break::new(0.3));
            }
            Block::BlockQuote(blocks) => {
                pdf.push(Break::new(0.2));
                for inner in blocks {
                    if let Block::Paragraph(inlines) = inner {
                        let text = self.inlines_to_string(inlines);
                        pdf.push(Paragraph::new(format!("  > {}", text)));
                    } else {
                        self.render_block(pdf, inner)?;
                    }
                }
                pdf.push(Break::new(0.3));
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                pdf.push(Break::new(0.2));
                let start_num = start.unwrap_or(1);

                for (i, item) in items.iter().enumerate() {
                    let marker = if *ordered {
                        format!("{}. ", start_num + i as u32)
                    } else {
                        "* ".to_string()
                    };

                    let marker = if let Some(checked) = item.checked {
                        if checked {
                            "[x] ".to_string()
                        } else {
                            "[ ] ".to_string()
                        }
                    } else {
                        marker
                    };

                    for (j, inner_block) in item.content.iter().enumerate() {
                        if j == 0 {
                            if let Block::Paragraph(inlines) = inner_block {
                                let text = self.inlines_to_string(inlines);
                                pdf.push(Paragraph::new(format!("  {}{}", marker, text)));
                            }
                        } else if let Block::Paragraph(inlines) = inner_block {
                            let text = self.inlines_to_string(inlines);
                            pdf.push(Paragraph::new(format!("    {}", text)));
                        }
                    }
                }
                pdf.push(Break::new(0.3));
            }
            Block::ThematicBreak => {
                pdf.push(Break::new(0.3));
                pdf.push(Paragraph::new("---"));
                pdf.push(Break::new(0.3));
            }
            Block::DisplayMath { content, label } => {
                pdf.push(Break::new(0.3));

                let mut display_text = content.clone();
                if let Some(lbl) = label {
                    if let Some(num) = self.doc.env_numbers.get(lbl) {
                        display_text.push_str(&format!("  ({})", num));
                    }
                }

                pdf.push(Paragraph::new(display_text));
                pdf.push(Break::new(0.3));
            }
            Block::Environment {
                kind,
                label,
                content,
                caption,
            } => {
                self.render_environment(pdf, kind, label.as_deref(), content, caption.as_deref())?;
            }
            Block::TableOfContents => {
                // Already rendered at the beginning
            }
            Block::Table {
                headers,
                rows,
                label,
                caption,
                ..
            } => {
                self.render_table(pdf, headers, rows, label.as_deref(), caption.as_deref())?;
            }
            Block::RawHtml(_) => {
                // Skip raw HTML in PDF
            }
        }

        Ok(())
    }

    fn render_environment(
        &mut self,
        pdf: &mut Document,
        kind: &EnvironmentKind,
        label: Option<&str>,
        content: &[Block],
        caption: Option<&[Inline]>,
    ) -> Result<()> {
        pdf.push(Break::new(0.3));

        // Environment header
        let header = if kind.is_numbered() {
            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    format!("{} {}.", kind.display_name(), num)
                } else {
                    format!("{}.", kind.display_name())
                }
            } else {
                format!("{}.", kind.display_name())
            }
        } else if matches!(kind, EnvironmentKind::Proof) {
            "Proof.".to_string()
        } else {
            String::new()
        };

        if !header.is_empty() {
            pdf.push(Paragraph::new(header));
        }

        for inner_block in content {
            self.render_block(pdf, inner_block)?;
        }

        if let Some(cap) = caption {
            let cap_text = self.inlines_to_string(cap);
            let mut caption_line = String::new();

            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    caption_line.push_str(&format!("{} {}: ", kind.display_name(), num));
                }
            }
            caption_line.push_str(&cap_text);
            pdf.push(Paragraph::new(caption_line));
        }

        if matches!(kind, EnvironmentKind::Proof) {
            pdf.push(Paragraph::new("QED"));
        }

        pdf.push(Break::new(0.3));
        Ok(())
    }

    fn render_table(
        &mut self,
        pdf: &mut Document,
        headers: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
        label: Option<&str>,
        caption: Option<&[Inline]>,
    ) -> Result<()> {
        pdf.push(Break::new(0.3));

        if let Some(cap) = caption {
            let cap_text = self.inlines_to_string(cap);
            let mut caption_line = String::new();

            if let Some(lbl) = label {
                if let Some(num) = self.doc.env_numbers.get(lbl) {
                    caption_line.push_str(&format!("Table {}: ", num));
                }
            }
            caption_line.push_str(&cap_text);
            pdf.push(Paragraph::new(caption_line));
            pdf.push(Break::new(0.2));
        }

        // Header row
        let header_text: Vec<String> = headers.iter().map(|h| self.inlines_to_string(h)).collect();
        pdf.push(Paragraph::new(header_text.join(" | ")));
        pdf.push(Paragraph::new("-".repeat(60)));

        // Data rows
        for row in rows {
            let row_text: Vec<String> = row.iter().map(|c| self.inlines_to_string(c)).collect();
            pdf.push(Paragraph::new(row_text.join(" | ")));
        }

        pdf.push(Break::new(0.3));
        Ok(())
    }

    fn render_footnotes_section(&mut self, pdf: &mut Document) -> Result<()> {
        if self.footnotes.is_empty() {
            return Ok(());
        }

        pdf.push(Break::new(1.0));
        pdf.push(Paragraph::new("-".repeat(30)));
        pdf.push(Break::new(0.2));

        let footnotes = std::mem::take(&mut self.footnotes);
        for (num, content) in footnotes {
            pdf.push(Paragraph::new(format!("[{}] {}", num, content)));
        }

        Ok(())
    }

    fn render_bibliography(&self, pdf: &mut Document) -> Result<()> {
        use crate::resolve::citations::get_citation_order;

        let order = get_citation_order(&self.doc.document);
        if order.is_empty() {
            return Ok(());
        }

        pdf.push(Break::new(1.0));
        pdf.push(Paragraph::new("References"));
        pdf.push(Break::new(0.3));

        for (i, key) in order.iter().enumerate() {
            if let Some(entry) = self.doc.citations.get(key) {
                let mut parts = Vec::new();

                if !entry.authors.is_empty() {
                    parts.push(entry.authors.join(", "));
                }
                if let Some(ref year) = entry.year {
                    parts.push(format!("({})", year));
                }
                if let Some(ref entry_title) = entry.title {
                    parts.push(format!("\"{}\"", entry_title));
                }
                if let Some(ref journal) = entry.journal {
                    parts.push(journal.clone());
                }
                if let Some(ref booktitle) = entry.booktitle {
                    parts.push(format!("In {}", booktitle));
                }

                let entry_text = parts.join(". ");
                pdf.push(Paragraph::new(format!("[{}] {}", i + 1, entry_text)));
            }
        }

        Ok(())
    }

    fn inlines_to_string(&mut self, inlines: &[Inline]) -> String {
        let mut result = String::new();
        for inline in inlines {
            match inline {
                Inline::Text(t) => result.push_str(t),
                Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                    result.push_str(&self.inlines_to_string(inner));
                }
                Inline::Code(c) => {
                    result.push('`');
                    result.push_str(c);
                    result.push('`');
                }
                Inline::Link { content, .. } => {
                    result.push_str(&self.inlines_to_string(content));
                }
                Inline::Image { alt, .. } => {
                    result.push_str(&format!("[Image: {}]", alt));
                }
                Inline::InlineMath(m) => {
                    result.push('$');
                    result.push_str(m);
                    result.push('$');
                }
                Inline::Citation(cite) => {
                    let keys: Vec<String> = cite
                        .keys
                        .iter()
                        .map(|k| {
                            if let Some(entry) = self.doc.citations.get(k) {
                                self.format_short_citation(entry)
                            } else {
                                k.clone()
                            }
                        })
                        .collect();

                    let mut cite_text = format!("[{}]", keys.join("; "));
                    if let Some(ref loc) = cite.locator {
                        cite_text = format!("[{}, {}]", keys.join("; "), loc);
                    }
                    result.push_str(&cite_text);
                }
                Inline::Reference { label, resolved } => {
                    let fallback = format!("??{}", label);
                    let text = resolved.as_deref().unwrap_or(&fallback);
                    result.push_str(text);
                }
                Inline::Footnote(kind) => {
                    self.footnote_counter += 1;
                    let num = self.footnote_counter;
                    result.push_str(&format!("[{}]", num));

                    if let FootnoteKind::Inline(content) = kind {
                        let footnote_text = self.inlines_to_string(content);
                        self.footnotes.push((num, footnote_text));
                    }
                }
                Inline::SoftBreak | Inline::HardBreak => result.push(' '),
                Inline::RawHtml(_) => {}
            }
        }
        result
    }

    fn format_short_citation(&self, entry: &crate::ast::BibEntry) -> String {
        let author = entry
            .authors
            .first()
            .map(|a| {
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
        } else {
            format!("{}, {}", author, year)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_config_default() {
        let config = PdfConfig::default();
        assert_eq!(config.font_size, 11);
        assert_eq!(config.paper_size, PaperSize::Letter);
    }

    #[test]
    fn test_paper_size_dimensions() {
        assert_eq!(PaperSize::Letter.dimensions(), (215.9, 279.4));
        assert_eq!(PaperSize::A4.dimensions(), (210.0, 297.0));
    }
}
