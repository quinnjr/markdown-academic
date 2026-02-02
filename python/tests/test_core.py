"""Tests for core markdown-academic functionality."""

import pytest

import markdown_academic as mda
from markdown_academic import (
    Document,
    MarkdownAcademicError,
    MathBackend,
    ParseError,
    RenderConfig,
)


class TestRender:
    """Tests for the render function."""

    def test_simple_text(self):
        """Test rendering simple text."""
        html = mda.render("Hello, world!")
        assert "<p>" in html
        assert "Hello, world!" in html

    def test_heading(self):
        """Test rendering headings."""
        html = mda.render("# Hello")
        assert "<h1>" in html
        assert "Hello" in html

    def test_emphasis(self):
        """Test rendering emphasis."""
        html = mda.render("This is *italic* and **bold**.")
        assert "<em>italic</em>" in html
        assert "<strong>bold</strong>" in html

    def test_inline_math(self):
        """Test rendering inline math."""
        html = mda.render("The equation $E = mc^2$ is famous.")
        assert "math inline" in html
        assert "E = mc^2" in html

    def test_display_math(self):
        """Test rendering display math."""
        html = mda.render("$$\\int_0^1 x dx$$")
        assert "math display" in html

    def test_code_block(self):
        """Test rendering code blocks."""
        html = mda.render("```python\nprint('hello')\n```")
        assert "<pre><code" in html
        assert "language-python" in html

    def test_list(self):
        """Test rendering lists."""
        html = mda.render("- Item 1\n- Item 2")
        assert "<ul>" in html
        assert "<li>" in html

    def test_standalone(self):
        """Test standalone HTML generation."""
        html = mda.render("# Test", standalone=True)
        assert "<!DOCTYPE html>" in html
        assert "<html" in html
        assert "</html>" in html

    def test_math_backend_katex(self):
        """Test KaTeX math backend."""
        html = mda.render("$x$", math_backend=MathBackend.KATEX)
        assert "math inline" in html

    def test_math_backend_mathjax(self):
        """Test MathJax math backend."""
        html = mda.render("$x$", math_backend=MathBackend.MATHJAX)
        assert "math inline" in html


class TestDocument:
    """Tests for the Document class."""

    def test_parse_and_render(self):
        """Test parsing and rendering a document."""
        doc = Document("# Hello\n\nWorld")
        html = doc.render()
        assert "<h1>" in html
        assert "Hello" in html

    def test_multiple_renders(self):
        """Test rendering the same document multiple times."""
        doc = Document("# Test")
        html1 = doc.render()
        html2 = doc.render(standalone=True)
        
        assert "<!DOCTYPE html>" not in html1
        assert "<!DOCTYPE html>" in html2

    def test_context_manager(self):
        """Test using Document as context manager."""
        with Document("# Test") as doc:
            html = doc.render()
            assert "<h1>" in html

    def test_render_with_different_backends(self):
        """Test rendering with different math backends."""
        doc = Document("$x^2$")
        
        html_katex = doc.render(math_backend=MathBackend.KATEX)
        html_mathjax = doc.render(math_backend=MathBackend.MATHJAX)
        
        assert "math inline" in html_katex
        assert "math inline" in html_mathjax


class TestRenderConfig:
    """Tests for RenderConfig."""

    def test_default_config(self):
        """Test default configuration."""
        config = RenderConfig()
        assert config.math_backend == MathBackend.KATEX
        assert config.standalone is False
        assert config.base_path is None

    def test_custom_config(self):
        """Test custom configuration."""
        config = RenderConfig(
            math_backend=MathBackend.MATHML,
            standalone=True,
            base_path="/some/path",
        )
        assert config.math_backend == MathBackend.MATHML
        assert config.standalone is True
        assert config.base_path == "/some/path"


class TestEnvironments:
    """Tests for academic environments."""

    def test_theorem(self):
        """Test theorem environment."""
        html = mda.render("::: theorem\nStatement.\n:::")
        assert "theorem" in html.lower()

    def test_proof(self):
        """Test proof environment."""
        html = mda.render("::: proof\nThe proof.\n:::")
        assert "proof" in html.lower()

    def test_definition(self):
        """Test definition environment."""
        html = mda.render("::: definition\nA definition.\n:::")
        assert "definition" in html.lower()


class TestCrossReferences:
    """Tests for cross-references."""

    def test_section_reference(self):
        """Test section cross-reference."""
        html = mda.render("# Intro {#sec:intro}\n\nSee @sec:intro.")
        assert "sec-intro" in html  # HTML id
        assert "href" in html  # Link

    def test_labeled_heading(self):
        """Test heading with label."""
        html = mda.render("## Methods {#sec:methods}")
        assert 'id="sec-methods"' in html


class TestFootnotes:
    """Tests for footnotes."""

    def test_inline_footnote(self):
        """Test inline footnote."""
        html = mda.render("Text^[A footnote].")
        assert "footnote" in html.lower()


class TestTableOfContents:
    """Tests for table of contents."""

    def test_toc(self):
        """Test table of contents generation."""
        html = mda.render("[[toc]]\n\n# One\n\n## Two")
        assert "toc" in html.lower()


class TestVersion:
    """Tests for version info."""

    def test_version_exists(self):
        """Test that version string exists."""
        assert mda.__version__
        assert isinstance(mda.__version__, str)


class TestPdf:
    """Tests for PDF generation."""

    def test_has_pdf_support(self):
        """Test that PDF support check works."""
        result = mda.has_pdf_support()
        assert isinstance(result, bool)

    def test_render_pdf_basic(self):
        """Test basic PDF generation."""
        if not mda.has_pdf_support():
            pytest.skip("PDF support not available")
        
        pdf_bytes = mda.render_pdf("# Hello\n\nWorld")
        assert isinstance(pdf_bytes, bytes)
        assert len(pdf_bytes) > 0
        assert pdf_bytes[:4] == b"%PDF"  # Valid PDF header

    def test_render_pdf_with_math(self):
        """Test PDF generation with math."""
        if not mda.has_pdf_support():
            pytest.skip("PDF support not available")
        
        pdf_bytes = mda.render_pdf("# Test\n\n$E=mc^2$")
        assert pdf_bytes[:4] == b"%PDF"

    def test_render_pdf_config_options(self):
        """Test PDF configuration options."""
        if not mda.has_pdf_support():
            pytest.skip("PDF support not available")
        
        pdf_bytes = mda.render_pdf(
            "# Test",
            paper_size=mda.PaperSize.A4,
            font_size=12,
            title="Test Document",
        )
        assert pdf_bytes[:4] == b"%PDF"

    def test_render_pdf_to_file(self, tmp_path):
        """Test PDF file writing."""
        if not mda.has_pdf_support():
            pytest.skip("PDF support not available")
        
        output_file = tmp_path / "test.pdf"
        mda.render_pdf_to_file("# Hello", output_file)
        
        assert output_file.exists()
        assert output_file.read_bytes()[:4] == b"%PDF"

    def test_paper_sizes(self):
        """Test paper size enum values."""
        assert mda.PaperSize.LETTER == 0
        assert mda.PaperSize.A4 == 1

    def test_pdf_config_dataclass(self):
        """Test PdfConfig dataclass."""
        config = mda.PdfConfig()
        assert config.paper_size == mda.PaperSize.LETTER
        assert config.font_size == 11
        assert config.title_page is False
        assert config.page_numbers is True
