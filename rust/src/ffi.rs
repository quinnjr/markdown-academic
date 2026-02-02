//! C FFI layer for cross-language interoperability.

use crate::error::Error;
use crate::parser::parse;
use crate::render::{render_html, HtmlConfig, MathBackend};
use crate::resolve::{resolve, ResolveConfig};
use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ptr;

/// Opaque handle to a parsed document.
pub struct MdAcademicDocument {
    inner: crate::ast::ResolvedDocument,
}

/// Configuration for rendering.
#[repr(C)]
pub struct MdAcademicConfig {
    /// Math backend: 0 = KaTeX, 1 = MathJax, 2 = MathML
    pub math_backend: c_int,
    /// Whether to generate standalone HTML (with DOCTYPE, head, etc.)
    pub standalone: c_int,
    /// Base path for resolving relative paths (null for current directory)
    pub base_path: *const c_char,
}

impl Default for MdAcademicConfig {
    fn default() -> Self {
        Self {
            math_backend: 0,
            standalone: 0,
            base_path: ptr::null(),
        }
    }
}

/// Result type for FFI operations.
#[repr(C)]
pub struct MdAcademicResult {
    /// Pointer to result string (caller must free with mdacademic_free_string)
    pub data: *mut c_char,
    /// Error message if data is null (caller must free with mdacademic_free_string)
    pub error: *mut c_char,
}

impl MdAcademicResult {
    fn ok(data: String) -> Self {
        let c_string = CString::new(data).unwrap_or_else(|_| CString::new("").unwrap());
        Self {
            data: c_string.into_raw(),
            error: ptr::null_mut(),
        }
    }

    fn err(error: String) -> Self {
        let c_string = CString::new(error).unwrap_or_else(|_| CString::new("Unknown error").unwrap());
        Self {
            data: ptr::null_mut(),
            error: c_string.into_raw(),
        }
    }
}

/// Parse a Markdown document and return a handle.
///
/// # Safety
///
/// - `input` must be a valid null-terminated UTF-8 string.
/// - The returned document must be freed with `mdacademic_free_document`.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_parse(input: *const c_char) -> *mut MdAcademicDocument {
    if input.is_null() {
        return ptr::null_mut();
    }

    let input = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let doc = match parse(input) {
        Ok(d) => d,
        Err(_) => return ptr::null_mut(),
    };

    let config = ResolveConfig::default();
    let resolved = match resolve(doc, &config) {
        Ok(r) => r,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(MdAcademicDocument { inner: resolved }))
}

/// Parse a Markdown document with configuration.
///
/// # Safety
///
/// - `input` must be a valid null-terminated UTF-8 string.
/// - `config` must be a valid pointer to MdAcademicConfig.
/// - The returned document must be freed with `mdacademic_free_document`.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_parse_with_config(
    input: *const c_char,
    config: *const MdAcademicConfig,
) -> *mut MdAcademicDocument {
    if input.is_null() {
        return ptr::null_mut();
    }

    let input = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let doc = match parse(input) {
        Ok(d) => d,
        Err(_) => return ptr::null_mut(),
    };

    let resolve_config = if config.is_null() {
        ResolveConfig::default()
    } else {
        let cfg = &*config;
        ResolveConfig {
            base_path: if cfg.base_path.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.base_path)
                    .to_str()
                    .ok()
                    .map(String::from)
            },
            ..Default::default()
        }
    };

    let resolved = match resolve(doc, &resolve_config) {
        Ok(r) => r,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(MdAcademicDocument { inner: resolved }))
}

/// Render a document to HTML.
///
/// # Safety
///
/// - `doc` must be a valid pointer from `mdacademic_parse`.
/// - The returned string must be freed with `mdacademic_free_string`.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_render_html(
    doc: *const MdAcademicDocument,
    config: *const MdAcademicConfig,
) -> MdAcademicResult {
    if doc.is_null() {
        return MdAcademicResult::err("Null document pointer".to_string());
    }

    let doc = &(*doc).inner;

    let html_config = if config.is_null() {
        HtmlConfig::default()
    } else {
        let cfg = &*config;
        HtmlConfig {
            math_backend: match cfg.math_backend {
                1 => MathBackend::MathJax,
                2 => MathBackend::MathML,
                _ => MathBackend::KaTeX,
            },
            standalone: cfg.standalone != 0,
            ..Default::default()
        }
    };

    match render_html(doc, &html_config) {
        Ok(html) => MdAcademicResult::ok(html),
        Err(e) => MdAcademicResult::err(e.to_string()),
    }
}

/// Parse and render in one step.
///
/// # Safety
///
/// - `input` must be a valid null-terminated UTF-8 string.
/// - The returned string must be freed with `mdacademic_free_string`.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_parse_and_render(
    input: *const c_char,
    config: *const MdAcademicConfig,
) -> MdAcademicResult {
    if input.is_null() {
        return MdAcademicResult::err("Null input pointer".to_string());
    }

    let input = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return MdAcademicResult::err("Invalid UTF-8 input".to_string()),
    };

    let doc = match parse(input) {
        Ok(d) => d,
        Err(e) => return MdAcademicResult::err(format!("Parse error: {}", e)),
    };

    let resolve_config = if config.is_null() {
        ResolveConfig::default()
    } else {
        let cfg = &*config;
        ResolveConfig {
            base_path: if cfg.base_path.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.base_path)
                    .to_str()
                    .ok()
                    .map(String::from)
            },
            ..Default::default()
        }
    };

    let resolved = match resolve(doc, &resolve_config) {
        Ok(r) => r,
        Err(e) => return MdAcademicResult::err(format!("Resolution error: {}", e)),
    };

    let html_config = if config.is_null() {
        HtmlConfig::default()
    } else {
        let cfg = &*config;
        HtmlConfig {
            math_backend: match cfg.math_backend {
                1 => MathBackend::MathJax,
                2 => MathBackend::MathML,
                _ => MathBackend::KaTeX,
            },
            standalone: cfg.standalone != 0,
            ..Default::default()
        }
    };

    match render_html(&resolved, &html_config) {
        Ok(html) => MdAcademicResult::ok(html),
        Err(e) => MdAcademicResult::err(format!("Render error: {}", e)),
    }
}

/// Free a string returned by mdacademic functions.
///
/// # Safety
///
/// - `s` must be a pointer returned by a mdacademic function, or null.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Free a document handle.
///
/// # Safety
///
/// - `doc` must be a pointer returned by `mdacademic_parse`, or null.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_free_document(doc: *mut MdAcademicDocument) {
    if !doc.is_null() {
        drop(Box::from_raw(doc));
    }
}

/// Free a result struct.
///
/// # Safety
///
/// - `result` must be a valid MdAcademicResult.
#[no_mangle]
pub unsafe extern "C" fn mdacademic_free_result(result: MdAcademicResult) {
    mdacademic_free_string(result.data);
    mdacademic_free_string(result.error);
}

/// Get the library version.
///
/// # Safety
///
/// The returned string is static and must not be freed.
#[no_mangle]
pub extern "C" fn mdacademic_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

// PDF FFI functions (feature-gated)

/// PDF configuration for FFI.
#[cfg(feature = "pdf")]
#[repr(C)]
pub struct MdAcademicPdfConfig {
    /// Paper size: 0 = Letter, 1 = A4
    pub paper_size: c_int,
    /// Font size in points
    pub font_size: c_int,
    /// Whether to include a title page
    pub title_page: c_int,
    /// Whether to include page numbers
    pub page_numbers: c_int,
    /// Document title (null for none)
    pub title: *const c_char,
    /// Base path for resolving relative paths
    pub base_path: *const c_char,
}

/// PDF result containing raw bytes.
#[cfg(feature = "pdf")]
#[repr(C)]
pub struct MdAcademicPdfResult {
    /// Pointer to PDF bytes (caller must free with mdacademic_free_pdf_data)
    pub data: *mut u8,
    /// Length of PDF data in bytes
    pub len: usize,
    /// Error message if data is null (caller must free with mdacademic_free_string)
    pub error: *mut c_char,
}

/// Parse and render to PDF in one step.
///
/// # Safety
///
/// - `input` must be a valid null-terminated UTF-8 string.
/// - The returned PDF data must be freed with `mdacademic_free_pdf_data`.
/// - The error string (if any) must be freed with `mdacademic_free_string`.
#[cfg(feature = "pdf")]
#[no_mangle]
pub unsafe extern "C" fn mdacademic_render_pdf(
    input: *const c_char,
    config: *const MdAcademicPdfConfig,
) -> MdAcademicPdfResult {
    use crate::render::{render_pdf, PaperSize, PdfConfig};

    if input.is_null() {
        return MdAcademicPdfResult {
            data: ptr::null_mut(),
            len: 0,
            error: CString::new("Null input pointer")
                .unwrap()
                .into_raw(),
        };
    }

    let input = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            return MdAcademicPdfResult {
                data: ptr::null_mut(),
                len: 0,
                error: CString::new("Invalid UTF-8 input")
                    .unwrap()
                    .into_raw(),
            }
        }
    };

    let doc = match parse(input) {
        Ok(d) => d,
        Err(e) => {
            return MdAcademicPdfResult {
                data: ptr::null_mut(),
                len: 0,
                error: CString::new(format!("Parse error: {}", e))
                    .unwrap()
                    .into_raw(),
            }
        }
    };

    let resolve_config = if config.is_null() {
        ResolveConfig::default()
    } else {
        let cfg = &*config;
        ResolveConfig {
            base_path: if cfg.base_path.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.base_path)
                    .to_str()
                    .ok()
                    .map(String::from)
            },
            ..Default::default()
        }
    };

    let resolved = match resolve(doc, &resolve_config) {
        Ok(r) => r,
        Err(e) => {
            return MdAcademicPdfResult {
                data: ptr::null_mut(),
                len: 0,
                error: CString::new(format!("Resolution error: {}", e))
                    .unwrap()
                    .into_raw(),
            }
        }
    };

    let pdf_config = if config.is_null() {
        PdfConfig::default()
    } else {
        let cfg = &*config;
        PdfConfig {
            paper_size: if cfg.paper_size == 1 {
                PaperSize::A4
            } else {
                PaperSize::Letter
            },
            font_size: if cfg.font_size > 0 {
                cfg.font_size as u8
            } else {
                11
            },
            title_page: cfg.title_page != 0,
            page_numbers: cfg.page_numbers != 0,
            title: if cfg.title.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.title).to_str().ok().map(String::from)
            },
            ..Default::default()
        }
    };

    match render_pdf(&resolved, &pdf_config) {
        Ok(mut bytes) => {
            let data = bytes.as_mut_ptr();
            let len = bytes.len();
            std::mem::forget(bytes); // Prevent deallocation
            MdAcademicPdfResult {
                data,
                len,
                error: ptr::null_mut(),
            }
        }
        Err(e) => MdAcademicPdfResult {
            data: ptr::null_mut(),
            len: 0,
            error: CString::new(format!("Render error: {}", e))
                .unwrap()
                .into_raw(),
        },
    }
}

/// Free PDF data returned by mdacademic_render_pdf.
///
/// # Safety
///
/// - `data` and `len` must be from a MdAcademicPdfResult.
#[cfg(feature = "pdf")]
#[no_mangle]
pub unsafe extern "C" fn mdacademic_free_pdf_data(data: *mut u8, len: usize) {
    if !data.is_null() && len > 0 {
        // Reconstruct the Vec and drop it
        let _ = Vec::from_raw_parts(data, len, len);
    }
}

/// Write PDF directly to a file.
///
/// # Safety
///
/// - `input` must be a valid null-terminated UTF-8 string.
/// - `path` must be a valid null-terminated UTF-8 file path.
/// - Returns 0 on success, -1 on error.
#[cfg(feature = "pdf")]
#[no_mangle]
pub unsafe extern "C" fn mdacademic_render_pdf_to_file(
    input: *const c_char,
    config: *const MdAcademicPdfConfig,
    path: *const c_char,
) -> c_int {
    use crate::render::{render_pdf_to_file, PaperSize, PdfConfig};

    if input.is_null() || path.is_null() {
        return -1;
    }

    let input = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let doc = match parse(input) {
        Ok(d) => d,
        Err(_) => return -1,
    };

    let resolve_config = if config.is_null() {
        ResolveConfig::default()
    } else {
        let cfg = &*config;
        ResolveConfig {
            base_path: if cfg.base_path.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.base_path)
                    .to_str()
                    .ok()
                    .map(String::from)
            },
            ..Default::default()
        }
    };

    let resolved = match resolve(doc, &resolve_config) {
        Ok(r) => r,
        Err(_) => return -1,
    };

    let pdf_config = if config.is_null() {
        PdfConfig::default()
    } else {
        let cfg = &*config;
        PdfConfig {
            paper_size: if cfg.paper_size == 1 {
                PaperSize::A4
            } else {
                PaperSize::Letter
            },
            font_size: if cfg.font_size > 0 {
                cfg.font_size as u8
            } else {
                11
            },
            title_page: cfg.title_page != 0,
            page_numbers: cfg.page_numbers != 0,
            title: if cfg.title.is_null() {
                None
            } else {
                CStr::from_ptr(cfg.title).to_str().ok().map(String::from)
            },
            ..Default::default()
        }
    };

    match render_pdf_to_file(&resolved, &pdf_config, path_str) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// Generate C header content for documentation
/// ```c
/// // markdown_academic.h
/// #ifndef MARKDOWN_ACADEMIC_H
/// #define MARKDOWN_ACADEMIC_H
///
/// #include <stdint.h>
///
/// typedef struct MdAcademicDocument MdAcademicDocument;
///
/// typedef struct {
///     int math_backend;  // 0 = KaTeX, 1 = MathJax, 2 = MathML
///     int standalone;    // 0 = fragment, 1 = full HTML document
///     const char* base_path;
/// } MdAcademicConfig;
///
/// typedef struct {
///     char* data;
///     char* error;
/// } MdAcademicResult;
///
/// MdAcademicDocument* mdacademic_parse(const char* input);
/// MdAcademicDocument* mdacademic_parse_with_config(const char* input, const MdAcademicConfig* config);
/// MdAcademicResult mdacademic_render_html(const MdAcademicDocument* doc, const MdAcademicConfig* config);
/// MdAcademicResult mdacademic_parse_and_render(const char* input, const MdAcademicConfig* config);
/// void mdacademic_free_string(char* s);
/// void mdacademic_free_document(MdAcademicDocument* doc);
/// void mdacademic_free_result(MdAcademicResult result);
/// const char* mdacademic_version(void);
///
/// #endif
/// ```
const _: () = ();
