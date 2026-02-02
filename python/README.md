# markdown-academic (Python)

Python bindings for [markdown-academic](https://github.com/quinnjr/markdown-academic), a Markdown parser and renderer for academic writing.

## Features

- **Math rendering** - Inline `$...$` and display `$$...$$` equations
- **Citations** - `[@key]` syntax with BibTeX support
- **Cross-references** - Label definitions `{#label}` and references `@label`
- **Environments** - Theorem, proof, definition, figure, and more
- **Automatic numbering** - Sections, equations, theorems, figures
- **Table of contents** - Generated from headings

## Installation

### From PyPI (when published)

```bash
pip install markdown-academic
```

### From Source

First, build the Rust library:

```bash
cd rust
cargo build --release
```

Then install the Python package:

```bash
cd python
pip install -e .
```

## Quick Start

```python
import markdown_academic as mda

# Simple rendering
html = mda.render("""
# Introduction {#sec:intro}

The equation $E = mc^2$ is famous. See @sec:intro.

::: theorem {#thm:main}
All natural numbers are interesting.
:::
""")

print(html)
```

## API Reference

### `render(text, *, math_backend, standalone, base_path)`

Render markdown-academic text to HTML.

```python
import markdown_academic as mda

# Basic usage
html = mda.render("# Hello\n\n$E=mc^2$")

# With options
html = mda.render(
    "# My Document",
    math_backend=mda.MathBackend.MATHJAX,
    standalone=True,  # Complete HTML document
    base_path="/path/to/document",  # For bibliography resolution
)
```

**Parameters:**
- `text` (str): The markdown-academic source text
- `math_backend` (MathBackend): KaTeX (default), MathJax, or MathML
- `standalone` (bool): Generate complete HTML document (default: False)
- `base_path` (str, optional): Base path for resolving relative paths

**Returns:** HTML string

### `Document` class

For rendering the same document multiple times with different options:

```python
import markdown_academic as mda

# Parse once
doc = mda.Document("# Hello\n\n$E=mc^2$")

# Render multiple times
html_fragment = doc.render()
html_full = doc.render(standalone=True)
html_mathml = doc.render(math_backend=mda.MathBackend.MATHML)

# Can also use as context manager
with mda.Document("# Test") as doc:
    html = doc.render()
```

### `MathBackend` enum

```python
from markdown_academic import MathBackend

MathBackend.KATEX   # Fast client-side rendering (default)
MathBackend.MATHJAX # Comprehensive LaTeX support
MathBackend.MATHML  # Native browser rendering
```

### `RenderConfig` dataclass

```python
from markdown_academic import RenderConfig, MathBackend

config = RenderConfig(
    math_backend=MathBackend.KATEX,
    standalone=False,
    base_path=None,
)
```

## Syntax Reference

### Front Matter (TOML)

```markdown
+++
title = "My Paper"
author = "Jane Doe"

[macros]
R = "\\mathbb{R}"

[bibliography]
path = "refs.bib"
+++
```

### Math

```markdown
Inline: $E = mc^2$

Display:
$$
\int_0^1 x\,dx = \frac{1}{2}
$$ {#eq:integral}

Reference: See @eq:integral.
```

### Citations

```markdown
As shown by [@knuth1984].
Multiple: [@knuth1984; @lamport1994]
With page: [@knuth1984, p. 42]
```

### Environments

```markdown
::: theorem {#thm:main}
Statement of the theorem.
:::

::: proof
The proof...
:::

::: definition
A **group** is...
:::
```

### Cross-References

```markdown
# Introduction {#sec:intro}

See @sec:intro, @eq:euler, @thm:main, @fig:chart.
```

## Error Handling

```python
import markdown_academic as mda
from markdown_academic import ParseError, RenderError, MarkdownAcademicError

try:
    html = mda.render(invalid_input)
except ParseError as e:
    print(f"Parse error: {e}")
except RenderError as e:
    print(f"Render error: {e}")
except MarkdownAcademicError as e:
    print(f"Other error: {e}")
```

## Environment Variable

Set `MARKDOWN_ACADEMIC_LIB` to specify the library path:

```bash
export MARKDOWN_ACADEMIC_LIB=/path/to/libmarkdown_academic.so
```

## License

MIT License - see [LICENSE](../LICENSE)
