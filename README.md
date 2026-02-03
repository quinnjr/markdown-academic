# markdown-academic

**Academic writing with the simplicity of Markdown.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

markdown-academic is a Markdown parser and renderer for academic and technical writing. It extends standard Markdown with features essential for scholarly documents—mathematical notation, citations, cross-references, theorem environments, and automatic numbering—while preserving the plain-text simplicity that makes Markdown a joy to write.

## Features

- **Math Rendering** — Inline `$...$` and display `$$...$$` equations with KaTeX, MathJax, or MathML backends
- **Citations** — `[@key]` syntax with BibTeX bibliography support
- **Cross-References** — Label anything with `{#label}`, reference with `@label`, automatic numbering
- **Environments** — Theorem, lemma, proof, definition, figure, and custom environments using `:::` fences
- **Table of Contents** — Auto-generated with `[[toc]]`
- **Footnotes** — Inline `^[text]` and reference `[^id]` styles
- **Custom Macros** — User-defined LaTeX commands via TOML front matter
- **Multiple Outputs** — Render to HTML or PDF from the same source

## File Extension

markdown-academic documents use the `.mda` file extension. This distinguishes them from standard Markdown (`.md`) while maintaining compatibility—`.mda` files are valid Markdown and render sensibly in any Markdown viewer.

## Quick Example

```markdown
+++
title = "My Paper"
[macros]
R = "\\mathbb{R}"
+++

# Introduction {#sec:intro}

Let $x \in \R$ be a real number. The Euler identity states:

$$e^{i\pi} + 1 = 0$$ {#eq:euler}

As shown in @eq:euler, this connects five fundamental constants.

::: theorem {#thm:main}
Every natural number greater than 1 is either prime or 
can be factored into primes.
:::

See @thm:main and [@knuth1984] for details.
```

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
markdown-academic = "0.1"
```

### From Source

```bash
git clone https://github.com/quinnjr/markdown-academic.git
cd markdown-academic/rust
cargo build --release
```

### Optional Features

| Feature | Description |
|---------|-------------|
| `mathml` | Enable MathML rendering backend |
| `wasm` | Enable WebAssembly bindings for JavaScript |
| `pdf` | Enable PDF output generation |
| `editor` | Enable the GUI preview application |

## Usage

### Rust

```rust
use markdown_academic::render;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
# Introduction {#sec:intro}

The equation $E = mc^2$ is famous. See @sec:intro.
"#;

    let html = render(input, None, None)?;
    println!("{}", html);
    Ok(())
}
```

### Python

```python
import markdown_academic as mda

# Render to HTML
html = mda.render("# Hello\n\nThe equation $E=mc^2$ is famous.")

# With options
html = mda.render(source, standalone=True, math_backend=mda.MathBackend.KATEX)

# PDF output (if compiled with pdf feature)
if mda.has_pdf_support():
    mda.render_pdf_to_file("# My Document", "output.pdf")
```

### JavaScript (WebAssembly)

```javascript
import init, { renderMarkdown } from './pkg/markdown_academic.js';

await init();
const html = renderMarkdown('# Hello $x^2$');
```

## Preview Application

A GUI application for editing and previewing `.mda` files:

```bash
cargo run --bin mda-preview --features editor
# Or open a specific file
cargo run --bin mda-preview --features editor -- path/to/file.mda
```

## Documentation

- **[Documentation Site](https://quinnjr.github.io/markdown-academic/)** — Full documentation
- **[Syntax Reference](https://quinnjr.github.io/markdown-academic/syntax.html)** — Complete syntax guide
- **[Rust API](https://quinnjr.github.io/markdown-academic/api.html)** — Rust API documentation
- **[FFI / Python](https://quinnjr.github.io/markdown-academic/ffi.html)** — Python and C bindings
- **[Whitepaper](https://quinnjr.github.io/markdown-academic/whitepaper.html)** — Design rationale

## Project Structure

```
markdown-academic/
├── rust/                   # Rust library and CLI
│   ├── src/
│   │   ├── lib.rs          # Library entry point
│   │   ├── parser/         # Markdown parser
│   │   ├── resolve/        # Reference resolution
│   │   ├── render/         # HTML/PDF rendering
│   │   ├── ast.rs          # Abstract syntax tree
│   │   ├── bibtex.rs       # BibTeX parser
│   │   ├── ffi.rs          # C FFI bindings
│   │   └── wasm.rs         # WebAssembly bindings
│   ├── examples/
│   └── Cargo.toml
├── python/                 # Python package
│   ├── markdown_academic/
│   └── pyproject.toml
├── docs/                   # Documentation website
└── WHITEPAPER.mda          # Design whitepaper (in .mda format)
```

## Syntax Overview

| Feature | Syntax | Description |
|---------|--------|-------------|
| Inline math | `$E = mc^2$` | LaTeX math inline |
| Display math | `$$...$$ {#eq:label}` | Numbered equation |
| Citation | `[@knuth1984]` | BibTeX citation |
| Reference | `@sec:intro` | Cross-reference |
| Label | `{#sec:intro}` | Define a label |
| Environment | `::: theorem ... :::` | Theorem-like blocks |
| Footnote | `^[inline note]` | Inline footnote |
| TOC | `[[toc]]` | Table of contents |

See the [Syntax Reference](https://quinnjr.github.io/markdown-academic/syntax.html) for complete documentation.

## Comparison

| Feature | markdown-academic | LaTeX | Pandoc | R Markdown |
|---------|-------------------|-------|--------|------------|
| Plain-text readable | ✓ | ✗ | ✓ | ✓ |
| Math support | ✓ | ✓ | ✓ | ✓ |
| Citations | ✓ | ✓ | ✓ | ✓ |
| Cross-references | ✓ | ✓ | ✓ | ✓ |
| Theorem environments | ✓ | ✓ | Partial | Partial |
| Learning curve | Low | High | Medium | Medium |
| Library embeddable | ✓ | ✗ | ✗ | ✗ |
| WebAssembly | ✓ | ✗ | ✗ | ✗ |

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Joseph R. Quinn**  
Email: quinn.josephr@protonmail.com  
GitHub: [@quinnjr](https://github.com/quinnjr)

---

*markdown-academic: Because academic writing should be about ideas, not markup.*
