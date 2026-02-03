//! mda-preview: A simple egui application for previewing and editing .mda files
//!
//! Run with: cargo run --bin mda-preview --features editor
//! Or: cargo run --bin mda-preview --features editor -- path/to/file.mda

use eframe::egui;
use markdown_academic::{render, HtmlConfig, ResolveConfig};
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("mda-preview"),
        ..Default::default()
    };

    // Check for file argument
    let initial_file = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "mda-preview",
        options,
        Box::new(|cc| Ok(Box::new(MdaPreviewApp::new(cc, initial_file)))),
    )
}

struct MdaPreviewApp {
    /// The source text being edited
    source: String,
    /// The rendered HTML output
    rendered_html: String,
    /// Current file path (if any)
    current_file: Option<PathBuf>,
    /// Whether the document has unsaved changes
    dirty: bool,
    /// Error message to display (if any)
    error_message: Option<String>,
    /// Split ratio between editor and preview
    split_ratio: f32,
    /// Show rendered HTML source instead of parsed preview
    show_html_source: bool,
    /// Font size for editor
    font_size: f32,
    /// Auto-refresh preview on edit
    auto_refresh: bool,
    /// Preview needs refresh
    needs_refresh: bool,
}

impl MdaPreviewApp {
    fn new(_cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        let mut app = Self {
            source: Self::default_content(),
            rendered_html: String::new(),
            current_file: None,
            dirty: false,
            error_message: None,
            split_ratio: 0.5,
            show_html_source: false,
            font_size: 14.0,
            auto_refresh: true,
            needs_refresh: true,
        };

        // Load initial file if provided
        if let Some(path) = initial_file {
            app.load_file(&path);
        }

        // Initial render
        app.refresh_preview();

        app
    }

    fn default_content() -> String {
        r#"+++
title = "Untitled Document"
author = "Author Name"
date = "2026-02-03"

[macros]
R = "\\mathbb{R}"
+++

# Introduction {#sec:intro}

Welcome to **markdown-academic**! This is a live preview editor for `.mda` files.

## Mathematics {#sec:math}

Inline math works like this: $E = mc^2$

Display math with labels:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$ {#eq:gaussian}

Reference equations: see @eq:gaussian.

## Environments {#sec:env}

::: theorem {#thm:example}
For all $\epsilon > 0$, there exists $\delta > 0$ such that the desired property holds.
:::

::: proof
By construction. Choose $\delta = \epsilon / 2$.
:::

## Features

- **Bold** and *italic* text
- `inline code`
- Cross-references like @sec:intro and @thm:example
- Citations like [@knuth1984]
- Footnotes^[This is an inline footnote]

| Column 1 | Column 2 | Column 3 |
|----------|:--------:|---------:|
| Left     | Center   | Right    |
| Data     | Data     | Data     |

Table: A sample table. {#tab:sample}

See @tab:sample for the table.
"#
        .to_string()
    }

    fn refresh_preview(&mut self) {
        let config = HtmlConfig {
            standalone: false,
            ..Default::default()
        };

        match render(&self.source, Some(&ResolveConfig::default()), Some(&config)) {
            Ok(html) => {
                self.rendered_html = html;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Render error: {}", e));
            }
        }
        self.needs_refresh = false;
    }

    fn load_file(&mut self, path: &PathBuf) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.source = content;
                self.current_file = Some(path.clone());
                self.dirty = false;
                self.needs_refresh = true;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load file: {}", e));
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = &self.current_file {
            match std::fs::write(path, &self.source) {
                Ok(_) => {
                    self.dirty = false;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save file: {}", e));
                }
            }
        } else {
            self.save_file_as();
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown Academic", &["mda"])
            .add_filter("Markdown", &["md"])
            .add_filter("All files", &["*"])
            .set_file_name("document.mda")
            .save_file()
        {
            self.current_file = Some(path);
            self.save_file();
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown Academic", &["mda"])
            .add_filter("Markdown", &["md"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.load_file(&path);
        }
    }

    fn new_file(&mut self) {
        self.source = Self::default_content();
        self.current_file = None;
        self.dirty = false;
        self.needs_refresh = true;
    }

    fn export_html(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("HTML", &["html"])
            .set_file_name("document.html")
            .save_file()
        {
            let config = HtmlConfig {
                standalone: true,
                title: self.current_file.as_ref().map(|p| {
                    p.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                }),
                ..Default::default()
            };

            match render(&self.source, Some(&ResolveConfig::default()), Some(&config)) {
                Ok(html) => match std::fs::write(&path, html) {
                    Ok(_) => {
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to export: {}", e));
                    }
                },
                Err(e) => {
                    self.error_message = Some(format!("Render error: {}", e));
                }
            }
        }
    }

    fn window_title(&self) -> String {
        let file_name = self
            .current_file
            .as_ref()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let dirty_marker = if self.dirty { " •" } else { "" };

        format!("{}{} - mda-preview", file_name, dirty_marker)
    }
}

impl eframe::App for MdaPreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update window title
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
            self.new_file();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            self.open_file();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_file_as();
            } else {
                self.save_file();
            }
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::E)) {
            self.export_html();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.needs_refresh = true;
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                        .clicked()
                    {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Open...").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .add(egui::Button::new("Export HTML...").shortcut_text("Ctrl+E"))
                        .clicked()
                    {
                        self.export_html();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.auto_refresh, "Auto-refresh preview");
                    if ui
                        .add(egui::Button::new("Refresh Now").shortcut_text("F5"))
                        .clicked()
                    {
                        self.needs_refresh = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.show_html_source, "Show HTML source");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Font size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0).suffix("px"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Split:");
                        ui.add(egui::Slider::new(&mut self.split_ratio, 0.2..=0.8));
                    });
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // Could show an about dialog
                        ui.close_menu();
                    }
                });

                // Right-aligned status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(path) = &self.current_file {
                        ui.label(
                            egui::RichText::new(path.to_string_lossy())
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
            });
        });

        // Error message panel
        if self.error_message.is_some() {
            let error = self.error_message.clone().unwrap();
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠").color(egui::Color32::YELLOW));
                    ui.label(egui::RichText::new(&error).color(egui::Color32::LIGHT_RED));
                    if ui.button("✕").clicked() {
                        self.error_message = None;
                    }
                });
            });
        }

        // Main content area with split panels
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let editor_width = available_width * self.split_ratio;
            let preview_width = available_width * (1.0 - self.split_ratio);

            ui.horizontal(|ui| {
                // Editor panel
                ui.vertical(|ui| {
                    ui.set_width(editor_width - 8.0);

                    ui.horizontal(|ui| {
                        ui.heading("Editor");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let lines = self.source.lines().count();
                            let chars = self.source.len();
                            ui.label(
                                egui::RichText::new(format!("{} lines, {} chars", lines, chars))
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );
                        });
                    });

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .show(ui, |ui| {
                            let response = ui.add(
                                egui::TextEdit::multiline(&mut self.source)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(30)
                                    .lock_focus(true),
                            );

                            if response.changed() {
                                self.dirty = true;
                                if self.auto_refresh {
                                    self.needs_refresh = true;
                                }
                            }
                        });
                });

                // Splitter
                ui.separator();

                // Preview panel
                ui.vertical(|ui| {
                    ui.set_width(preview_width - 8.0);

                    ui.horizontal(|ui| {
                        ui.heading("Preview");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(if self.show_html_source {
                                    "📄 Show Preview"
                                } else {
                                    "🔧 Show HTML"
                                })
                                .clicked()
                            {
                                self.show_html_source = !self.show_html_source;
                            }
                            if ui.button("🔄 Refresh").clicked() {
                                self.needs_refresh = true;
                            }
                        });
                    });

                    ui.separator();

                    // Refresh preview if needed
                    if self.needs_refresh {
                        self.refresh_preview();
                    }

                    egui::ScrollArea::vertical()
                        .id_salt("preview_scroll")
                        .show(ui, |ui| {
                            if self.show_html_source {
                                // Show raw HTML source
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.rendered_html.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY),
                                );
                            } else {
                                // Show simple rendered preview
                                // Note: egui doesn't have a full HTML renderer, so we'll show
                                // a simplified markdown-style preview
                                render_preview(ui, &self.rendered_html);
                            }
                        });
                });
            });
        });
    }
}

/// Simple preview renderer that displays HTML with basic formatting
fn render_preview(ui: &mut egui::Ui, html: &str) {
    // This is a very simplified HTML renderer - egui doesn't have native HTML support
    // We'll parse the HTML and render it with egui widgets

    let mut in_pre = false;
    let mut code_buffer = String::new();

    for line in html.lines() {
        let trimmed = line.trim();

        // Handle code blocks
        if trimmed.starts_with("<pre><code") {
            in_pre = true;
            code_buffer.clear();
            continue;
        }
        if trimmed.contains("</code></pre>") {
            in_pre = false;
            // Render the code block
            let code = code_buffer.trim();
            if !code.is_empty() {
                egui::Frame::none()
                    .fill(egui::Color32::from_gray(30))
                    .inner_margin(8.0)
                    .outer_margin(4.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(code).monospace().color(egui::Color32::LIGHT_GRAY));
                    });
            }
            code_buffer.clear();
            continue;
        }
        if in_pre {
            code_buffer.push_str(line);
            code_buffer.push('\n');
            continue;
        }

        // Handle headings
        if let Some(content) = trimmed.strip_prefix("<h1") {
            if let Some(text) = extract_tag_content(content, "h1") {
                ui.add_space(12.0);
                ui.heading(egui::RichText::new(strip_html(&text)).size(28.0).strong());
                ui.add_space(8.0);
                continue;
            }
        }
        if let Some(content) = trimmed.strip_prefix("<h2") {
            if let Some(text) = extract_tag_content(content, "h2") {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new(strip_html(&text)).size(22.0).strong());
                ui.add_space(6.0);
                continue;
            }
        }
        if let Some(content) = trimmed.strip_prefix("<h3") {
            if let Some(text) = extract_tag_content(content, "h3") {
                ui.add_space(8.0);
                ui.heading(egui::RichText::new(strip_html(&text)).size(18.0).strong());
                ui.add_space(4.0);
                continue;
            }
        }

        // Handle paragraphs
        if trimmed.starts_with("<p>") {
            let text = trimmed
                .strip_prefix("<p>")
                .unwrap_or(trimmed)
                .strip_suffix("</p>")
                .unwrap_or(trimmed);
            if !text.is_empty() {
                ui.label(strip_html(text));
                ui.add_space(4.0);
            }
            continue;
        }

        // Handle list items
        if trimmed.starts_with("<li>") {
            let text = trimmed
                .strip_prefix("<li>")
                .unwrap_or(trimmed)
                .strip_suffix("</li>")
                .unwrap_or(trimmed);
            ui.horizontal(|ui| {
                ui.label("•");
                ui.label(strip_html(text));
            });
            continue;
        }

        // Handle horizontal rules
        if trimmed == "<hr>" || trimmed == "<hr/>" || trimmed == "<hr />" {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            continue;
        }

        // Handle theorem-like environments
        if trimmed.contains("theorem-like") || trimmed.contains("mda-theorem") {
            // Start of a theorem block
            continue;
        }

        // Handle blockquotes
        if trimmed.starts_with("<blockquote>") {
            continue;
        }

        // Handle table cells (simplified)
        if trimmed.starts_with("<th>") || trimmed.starts_with("<td>") {
            let text = strip_html(trimmed);
            if !text.is_empty() {
                ui.label(text);
            }
            continue;
        }

        // Skip pure HTML tags
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            continue;
        }

        // Handle div content - skip the tag, content will be on next lines
        if trimmed.starts_with("<div") {
            continue;
        }

        // Handle any remaining content
        let text = strip_html(trimmed);
        if !text.is_empty() && !text.chars().all(|c| c.is_whitespace()) {
            ui.label(&text);
        }
    }
}

/// Extract content between a tag
fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let end_tag = format!("</{}>", tag);
    if let Some(start) = html.find('>') {
        let content = &html[start + 1..];
        if let Some(end) = content.find(&end_tag) {
            return Some(content[..end].to_string());
        }
        // Tag might end on different line
        return Some(content.to_string());
    }
    None
}

/// Strip HTML tags from text, preserving content
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
