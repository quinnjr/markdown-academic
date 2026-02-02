"""
markdown-academic: Academic writing with the simplicity of Markdown.

A Python wrapper for the markdown-academic Rust library, providing
math rendering, citations, cross-references, and theorem environments.

Example:
    >>> import markdown_academic as mda
    >>> html = mda.render("# Hello\n\nThe equation $E=mc^2$ is famous.")
    >>> print(html)
"""

from .core import (
    render,
    parse_and_render,
    Document,
    MathBackend,
    RenderConfig,
    MarkdownAcademicError,
    ParseError,
    RenderError,
)
from .version import __version__

__all__ = [
    "render",
    "parse_and_render",
    "Document",
    "MathBackend",
    "RenderConfig",
    "MarkdownAcademicError",
    "ParseError",
    "RenderError",
    "__version__",
]
