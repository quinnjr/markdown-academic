"""
markdown-academic: Academic writing with the simplicity of Markdown.

A Python wrapper for the markdown-academic Rust library, providing
math rendering, citations, cross-references, and theorem environments.

Example:
    >>> import markdown_academic as mda
    >>> html = mda.render("# Hello\n\nThe equation $E=mc^2$ is famous.")
    >>> print(html)

    # PDF generation (requires library built with pdf feature)
    >>> if mda.has_pdf_support():
    ...     pdf = mda.render_pdf("# Hello World")
    ...     with open("output.pdf", "wb") as f:
    ...         f.write(pdf)
"""

from .core import (
    render,
    parse_and_render,
    render_pdf,
    render_pdf_to_file,
    has_pdf_support,
    Document,
    MathBackend,
    PaperSize,
    RenderConfig,
    PdfConfig,
    MarkdownAcademicError,
    ParseError,
    RenderError,
    PdfError,
    get_library_version,
)
from .version import __version__

__all__ = [
    # HTML rendering
    "render",
    "parse_and_render",
    "Document",
    "MathBackend",
    "RenderConfig",
    # PDF rendering
    "render_pdf",
    "render_pdf_to_file",
    "has_pdf_support",
    "PaperSize",
    "PdfConfig",
    # Errors
    "MarkdownAcademicError",
    "ParseError",
    "RenderError",
    "PdfError",
    # Utilities
    "get_library_version",
    "__version__",
]
