"""
Core functionality for markdown-academic Python bindings.

This module provides Python bindings to the markdown-academic Rust library
via ctypes FFI.
"""

from __future__ import annotations

import ctypes
import os
import platform
import sys
from ctypes import POINTER, Structure, c_char_p, c_int, c_size_t, c_uint8, c_void_p
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Optional, Union


class MarkdownAcademicError(Exception):
    """Base exception for markdown-academic errors."""

    pass


class ParseError(MarkdownAcademicError):
    """Raised when document parsing fails."""

    pass


class RenderError(MarkdownAcademicError):
    """Raised when HTML rendering fails."""

    pass


class PdfError(MarkdownAcademicError):
    """Raised when PDF generation fails."""

    pass


class MathBackend(IntEnum):
    """Math rendering backend options."""

    KATEX = 0
    """KaTeX - fast client-side rendering (default)."""

    MATHJAX = 1
    """MathJax - comprehensive LaTeX support."""

    MATHML = 2
    """MathML - native browser rendering, no JavaScript."""


class PaperSize(IntEnum):
    """Paper size options for PDF generation."""

    LETTER = 0
    """US Letter (8.5" x 11")."""

    A4 = 1
    """ISO A4 (210mm x 297mm)."""


@dataclass
class RenderConfig:
    """Configuration options for rendering.

    Attributes:
        math_backend: Math rendering backend (default: KaTeX).
        standalone: If True, generate complete HTML document with DOCTYPE,
            head, and body. If False, generate just the content fragment.
        base_path: Base path for resolving relative file paths (e.g., bibliography).
    """

    math_backend: MathBackend = MathBackend.KATEX
    standalone: bool = False
    base_path: Optional[str] = None


@dataclass
class PdfConfig:
    """Configuration options for PDF generation.

    Attributes:
        paper_size: Paper size (default: Letter).
        font_size: Font size in points (default: 11).
        title_page: If True, include a title page.
        page_numbers: If True, include page numbers.
        title: Document title for PDF metadata.
        base_path: Base path for resolving relative file paths.
    """

    paper_size: PaperSize = PaperSize.LETTER
    font_size: int = 11
    title_page: bool = False
    page_numbers: bool = True
    title: Optional[str] = None
    base_path: Optional[str] = None


class _MdAcademicConfig(Structure):
    """C struct for configuration."""

    _fields_ = [
        ("math_backend", c_int),
        ("standalone", c_int),
        ("base_path", c_char_p),
    ]


class _MdAcademicResult(Structure):
    """C struct for results."""

    _fields_ = [
        ("data", c_char_p),
        ("error", c_char_p),
    ]


class _MdAcademicPdfConfig(Structure):
    """C struct for PDF configuration."""

    _fields_ = [
        ("paper_size", c_int),
        ("font_size", c_int),
        ("title_page", c_int),
        ("page_numbers", c_int),
        ("title", c_char_p),
        ("base_path", c_char_p),
    ]


class _MdAcademicPdfResult(Structure):
    """C struct for PDF results."""

    _fields_ = [
        ("data", POINTER(c_uint8)),
        ("len", c_size_t),
        ("error", c_char_p),
    ]


def _get_library_path() -> Path:
    """Find the markdown-academic shared library.

    Searches in order:
    1. MARKDOWN_ACADEMIC_LIB environment variable
    2. Package directory
    3. Adjacent to the package (for development)
    4. System library paths
    """
    # Check environment variable
    env_path = os.environ.get("MARKDOWN_ACADEMIC_LIB")
    if env_path:
        path = Path(env_path)
        if path.exists():
            return path

    # Determine library filename based on platform
    system = platform.system()
    if system == "Linux":
        lib_name = "libmarkdown_academic.so"
    elif system == "Darwin":
        lib_name = "libmarkdown_academic.dylib"
    elif system == "Windows":
        lib_name = "markdown_academic.dll"
    else:
        raise MarkdownAcademicError(f"Unsupported platform: {system}")

    # Check package directory
    package_dir = Path(__file__).parent
    lib_path = package_dir / lib_name
    if lib_path.exists():
        return lib_path

    # Check for development build (../rust/target/release/)
    dev_path = package_dir.parent.parent / "rust" / "target" / "release" / lib_name
    if dev_path.exists():
        return dev_path

    # Check debug build
    debug_path = package_dir.parent.parent / "rust" / "target" / "debug" / lib_name
    if debug_path.exists():
        return debug_path

    # Try loading from system path
    return Path(lib_name)


class _Library:
    """Wrapper for the markdown-academic shared library."""

    _instance: Optional[_Library] = None
    _lib: Optional[ctypes.CDLL] = None

    def __new__(cls) -> _Library:
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance._load_library()
        return cls._instance

    def _load_library(self) -> None:
        """Load the shared library and set up function signatures."""
        lib_path = _get_library_path()

        try:
            self._lib = ctypes.CDLL(str(lib_path))
        except OSError as e:
            raise MarkdownAcademicError(
                f"Failed to load markdown-academic library from {lib_path}. "
                f"Make sure the Rust library is built: cd rust && cargo build --release\n"
                f"Original error: {e}"
            ) from e

        # mdacademic_parse_and_render
        self._lib.mdacademic_parse_and_render.argtypes = [
            c_char_p,
            POINTER(_MdAcademicConfig),
        ]
        self._lib.mdacademic_parse_and_render.restype = _MdAcademicResult

        # mdacademic_parse
        self._lib.mdacademic_parse.argtypes = [c_char_p]
        self._lib.mdacademic_parse.restype = c_void_p

        # mdacademic_parse_with_config
        self._lib.mdacademic_parse_with_config.argtypes = [
            c_char_p,
            POINTER(_MdAcademicConfig),
        ]
        self._lib.mdacademic_parse_with_config.restype = c_void_p

        # mdacademic_render_html
        self._lib.mdacademic_render_html.argtypes = [
            c_void_p,
            POINTER(_MdAcademicConfig),
        ]
        self._lib.mdacademic_render_html.restype = _MdAcademicResult

        # mdacademic_free_string
        self._lib.mdacademic_free_string.argtypes = [c_char_p]
        self._lib.mdacademic_free_string.restype = None

        # mdacademic_free_document
        self._lib.mdacademic_free_document.argtypes = [c_void_p]
        self._lib.mdacademic_free_document.restype = None

        # mdacademic_free_result
        self._lib.mdacademic_free_result.argtypes = [_MdAcademicResult]
        self._lib.mdacademic_free_result.restype = None

        # mdacademic_version
        self._lib.mdacademic_version.argtypes = []
        self._lib.mdacademic_version.restype = c_char_p

        # PDF functions (may not be available if compiled without pdf feature)
        try:
            # mdacademic_render_pdf
            self._lib.mdacademic_render_pdf.argtypes = [
                c_char_p,
                POINTER(_MdAcademicPdfConfig),
            ]
            self._lib.mdacademic_render_pdf.restype = _MdAcademicPdfResult

            # mdacademic_render_pdf_to_file
            self._lib.mdacademic_render_pdf_to_file.argtypes = [
                c_char_p,
                POINTER(_MdAcademicPdfConfig),
                c_char_p,
            ]
            self._lib.mdacademic_render_pdf_to_file.restype = c_int

            # mdacademic_free_pdf_data
            self._lib.mdacademic_free_pdf_data.argtypes = [POINTER(c_uint8), c_size_t]
            self._lib.mdacademic_free_pdf_data.restype = None

            self._has_pdf = True
        except AttributeError:
            self._has_pdf = False

    @property
    def has_pdf(self) -> bool:
        """Check if PDF support is available."""
        return self._has_pdf

    @property
    def lib(self) -> ctypes.CDLL:
        """Get the loaded library."""
        if self._lib is None:
            raise MarkdownAcademicError("Library not loaded")
        return self._lib


def _make_config(config: Optional[RenderConfig]) -> _MdAcademicConfig:
    """Create a C config struct from Python config."""
    if config is None:
        config = RenderConfig()

    c_config = _MdAcademicConfig()
    c_config.math_backend = int(config.math_backend)
    c_config.standalone = 1 if config.standalone else 0
    c_config.base_path = (
        config.base_path.encode("utf-8") if config.base_path else None
    )

    return c_config


def _handle_result(result: _MdAcademicResult, lib: ctypes.CDLL) -> str:
    """Handle a result struct, raising exceptions on error."""
    try:
        if result.error:
            error_msg = result.error.decode("utf-8")
            if "Parse error" in error_msg:
                raise ParseError(error_msg)
            elif "Render error" in error_msg:
                raise RenderError(error_msg)
            else:
                raise MarkdownAcademicError(error_msg)

        if result.data is None:
            raise MarkdownAcademicError("No data returned")

        return result.data.decode("utf-8")
    finally:
        lib.mdacademic_free_result(result)


def render(
    text: str,
    *,
    math_backend: MathBackend = MathBackend.KATEX,
    standalone: bool = False,
    base_path: Optional[str] = None,
) -> str:
    """Render markdown-academic text to HTML.

    This is the main entry point for converting markdown-academic
    documents to HTML.

    Args:
        text: The markdown-academic source text.
        math_backend: Math rendering backend (default: KaTeX).
        standalone: If True, generate a complete HTML document.
        base_path: Base path for resolving relative paths.

    Returns:
        The rendered HTML string.

    Raises:
        ParseError: If the document cannot be parsed.
        RenderError: If rendering fails.
        MarkdownAcademicError: For other errors.

    Example:
        >>> html = render("# Hello\\n\\nThe equation $E=mc^2$ is famous.")
        >>> print(html)
        <h1>Hello</h1>
        <p>The equation <span class="math inline">...</span> is famous.</p>

        >>> html = render("# Title", standalone=True)
        >>> html.startswith("<!DOCTYPE html>")
        True
    """
    config = RenderConfig(
        math_backend=math_backend,
        standalone=standalone,
        base_path=base_path,
    )
    return parse_and_render(text, config)


def parse_and_render(
    text: str,
    config: Optional[RenderConfig] = None,
) -> str:
    """Parse and render markdown-academic text with configuration.

    Args:
        text: The markdown-academic source text.
        config: Rendering configuration options.

    Returns:
        The rendered HTML string.

    Raises:
        ParseError: If the document cannot be parsed.
        RenderError: If rendering fails.
        MarkdownAcademicError: For other errors.
    """
    library = _Library()
    c_config = _make_config(config)

    result = library.lib.mdacademic_parse_and_render(
        text.encode("utf-8"),
        ctypes.byref(c_config),
    )

    return _handle_result(result, library.lib)


class Document:
    """A parsed markdown-academic document.

    Use this class when you need to render the same document multiple
    times with different configurations, avoiding repeated parsing.

    Example:
        >>> doc = Document("# Hello\\n\\n$E=mc^2$")
        >>> html_fragment = doc.render()
        >>> html_full = doc.render(standalone=True)
    """

    def __init__(
        self,
        text: str,
        *,
        base_path: Optional[str] = None,
    ) -> None:
        """Parse a markdown-academic document.

        Args:
            text: The markdown-academic source text.
            base_path: Base path for resolving relative paths.

        Raises:
            ParseError: If the document cannot be parsed.
        """
        self._library = _Library()
        self._doc_ptr: Optional[c_void_p] = None

        if base_path:
            config = _MdAcademicConfig()
            config.math_backend = 0
            config.standalone = 0
            config.base_path = base_path.encode("utf-8")

            self._doc_ptr = self._library.lib.mdacademic_parse_with_config(
                text.encode("utf-8"),
                ctypes.byref(config),
            )
        else:
            self._doc_ptr = self._library.lib.mdacademic_parse(
                text.encode("utf-8"),
            )

        if not self._doc_ptr:
            raise ParseError("Failed to parse document")

    def __del__(self) -> None:
        """Free the document handle."""
        if self._doc_ptr and hasattr(self, "_library"):
            self._library.lib.mdacademic_free_document(self._doc_ptr)
            self._doc_ptr = None

    def render(
        self,
        *,
        math_backend: MathBackend = MathBackend.KATEX,
        standalone: bool = False,
    ) -> str:
        """Render the document to HTML.

        Args:
            math_backend: Math rendering backend.
            standalone: If True, generate a complete HTML document.

        Returns:
            The rendered HTML string.

        Raises:
            RenderError: If rendering fails.
            MarkdownAcademicError: If the document has been freed.
        """
        if not self._doc_ptr:
            raise MarkdownAcademicError("Document has been freed")

        config = _MdAcademicConfig()
        config.math_backend = int(math_backend)
        config.standalone = 1 if standalone else 0
        config.base_path = None

        result = self._library.lib.mdacademic_render_html(
            self._doc_ptr,
            ctypes.byref(config),
        )

        return _handle_result(result, self._library.lib)

    def __enter__(self) -> Document:
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit - free the document."""
        if self._doc_ptr:
            self._library.lib.mdacademic_free_document(self._doc_ptr)
            self._doc_ptr = None


def get_library_version() -> str:
    """Get the version of the underlying Rust library.

    Returns:
        Version string (e.g., "0.1.0").
    """
    library = _Library()
    version = library.lib.mdacademic_version()
    return version.decode("utf-8")


def has_pdf_support() -> bool:
    """Check if PDF support is available.

    Returns:
        True if the library was compiled with PDF support.
    """
    library = _Library()
    return library.has_pdf


def _make_pdf_config(config: Optional[PdfConfig]) -> _MdAcademicPdfConfig:
    """Create a C PDF config struct from Python config."""
    if config is None:
        config = PdfConfig()

    c_config = _MdAcademicPdfConfig()
    c_config.paper_size = int(config.paper_size)
    c_config.font_size = config.font_size
    c_config.title_page = 1 if config.title_page else 0
    c_config.page_numbers = 1 if config.page_numbers else 0
    c_config.title = config.title.encode("utf-8") if config.title else None
    c_config.base_path = config.base_path.encode("utf-8") if config.base_path else None

    return c_config


def render_pdf(
    text: str,
    *,
    paper_size: PaperSize = PaperSize.LETTER,
    font_size: int = 11,
    title_page: bool = False,
    page_numbers: bool = True,
    title: Optional[str] = None,
    base_path: Optional[str] = None,
) -> bytes:
    """Render markdown-academic text to PDF.

    This function converts a markdown-academic document to PDF format.

    Args:
        text: The markdown-academic source text.
        paper_size: Paper size (default: Letter).
        font_size: Font size in points (default: 11).
        title_page: If True, include a title page.
        page_numbers: If True, include page numbers.
        title: Document title for PDF metadata.
        base_path: Base path for resolving relative paths.

    Returns:
        The PDF content as bytes.

    Raises:
        PdfError: If PDF generation fails.
        MarkdownAcademicError: If PDF support is not available.

    Example:
        >>> pdf_bytes = render_pdf("# Hello\\n\\n$E=mc^2$")
        >>> with open("output.pdf", "wb") as f:
        ...     f.write(pdf_bytes)
    """
    library = _Library()

    if not library.has_pdf:
        raise MarkdownAcademicError(
            "PDF support is not available. "
            "Rebuild the Rust library with: cargo build --release --features pdf"
        )

    config = PdfConfig(
        paper_size=paper_size,
        font_size=font_size,
        title_page=title_page,
        page_numbers=page_numbers,
        title=title,
        base_path=base_path,
    )
    c_config = _make_pdf_config(config)

    result = library.lib.mdacademic_render_pdf(
        text.encode("utf-8"),
        ctypes.byref(c_config),
    )

    try:
        if result.error:
            error_msg = result.error.decode("utf-8")
            library.lib.mdacademic_free_string(result.error)
            raise PdfError(error_msg)

        if not result.data or result.len == 0:
            raise PdfError("No PDF data returned")

        # Copy the data before freeing
        pdf_bytes = bytes(result.data[: result.len])
        return pdf_bytes
    finally:
        if result.data and result.len > 0:
            library.lib.mdacademic_free_pdf_data(result.data, result.len)


def render_pdf_to_file(
    text: str,
    output_path: Union[str, Path],
    *,
    paper_size: PaperSize = PaperSize.LETTER,
    font_size: int = 11,
    title_page: bool = False,
    page_numbers: bool = True,
    title: Optional[str] = None,
    base_path: Optional[str] = None,
) -> None:
    """Render markdown-academic text to a PDF file.

    This function converts a markdown-academic document to PDF format
    and writes it directly to a file.

    Args:
        text: The markdown-academic source text.
        output_path: Path to the output PDF file.
        paper_size: Paper size (default: Letter).
        font_size: Font size in points (default: 11).
        title_page: If True, include a title page.
        page_numbers: If True, include page numbers.
        title: Document title for PDF metadata.
        base_path: Base path for resolving relative paths.

    Raises:
        PdfError: If PDF generation fails.
        MarkdownAcademicError: If PDF support is not available.

    Example:
        >>> render_pdf_to_file("# Hello\\n\\n$E=mc^2$", "output.pdf")
    """
    library = _Library()

    if not library.has_pdf:
        raise MarkdownAcademicError(
            "PDF support is not available. "
            "Rebuild the Rust library with: cargo build --release --features pdf"
        )

    config = PdfConfig(
        paper_size=paper_size,
        font_size=font_size,
        title_page=title_page,
        page_numbers=page_numbers,
        title=title,
        base_path=base_path,
    )
    c_config = _make_pdf_config(config)

    result = library.lib.mdacademic_render_pdf_to_file(
        text.encode("utf-8"),
        ctypes.byref(c_config),
        str(output_path).encode("utf-8"),
    )

    if result != 0:
        raise PdfError(f"Failed to write PDF to {output_path}")
