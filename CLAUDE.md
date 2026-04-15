# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

markdown-academic is a multi-language project with a Rust core library and bindings for Python (FFI) and JavaScript (WebAssembly). All language targets share the same Rust implementation.

## Build Commands

### Rust (primary library)
```bash
cd rust
cargo build                        # debug build
cargo build --release              # release build
cargo build --features wasm        # build with WASM support
cargo build --all-features         # build all optional features
```

### WASM / JavaScript package
```bash
# Step 1: compile Rust to WASM (outputs to wasm/dist/pkg/)
cd rust && wasm-pack build --target web --out-dir ../wasm/dist/pkg --features wasm

# Step 2: bundle TypeScript wrappers
cd wasm && node scripts/bundle.cjs

# Combined (runs both steps)
cd wasm && pnpm build
```

### Python bindings
```bash
cd rust && cargo build --release   # must build Rust library first
cd python && pip install -e .
```

## Test Commands

### Rust
```bash
cd rust
cargo test                         # default features
cargo test --all-features          # all feature-gated code
cargo test test_name               # single test by name
cargo test -- --nocapture          # show println output
```

### WASM / JavaScript
```bash
cd wasm && pnpm test               # vitest runner (needs dist/pkg/ built first)
cd wasm && pnpm test:watch         # watch mode
```

### Python
```bash
cd python && pytest tests/ -v
cd python && pytest tests/test_core.py::TestClassName::test_method -v  # single test
```

## Lint / Format

```bash
cd rust && cargo fmt               # format Rust code
cd rust && cargo fmt --check       # check formatting (CI mode)
cd rust && cargo clippy --all-features -- -D warnings   # lint (warnings are errors)
cd python && ruff check .          # Python linting
cd python && black .               # Python formatting
```

## Architecture

The rendering pipeline is a three-stage transformation:

```
source (.mda) → parse() → Document → resolve() → ResolvedDocument → render_html() → HTML
```

1. **`parser/`** — tokenises (lexer.rs) then builds the AST. `parse_blocks` handles block-level structure; `parse_inlines` handles inline spans within blocks. TOML front matter (`+++...+++`) is extracted before block parsing.

2. **`resolve/`** — walks the `Document` AST and resolves:
   - `numbering.rs` — assigns sequence numbers to headings, equations, theorems, figures
   - `references.rs` — resolves `@label` cross-references to their targets
   - `citations.rs` — looks up `[@key]` entries against the parsed bibliography
   - `macros.rs` — expands user-defined LaTeX macros from front matter

3. **`render/`** — converts the `ResolvedDocument` to output:
   - `html.rs` — main HTML renderer
   - `math/` — math backend dispatch (KaTeX placeholder tags, MathJax script tags, or MathML via `latex2mathml`)
   - `pdf.rs` — PDF output via `genpdf` (feature-gated with `pdf`)

### Key types (ast.rs)

- `Document` — raw parse result: `metadata: Metadata` + `blocks: Vec<Block>`
- `ResolvedDocument` — post-resolution result with all references linked and numbered
- `Block` / `Inline` — AST node enums

### Language bindings

- **`ffi.rs`** — C-compatible ABI for Python (uses `libc`, compiled as `cdylib`/`staticlib`). The Python `core.py` loads this via ctypes.
- **`wasm.rs`** — `wasm-bindgen` exports for JavaScript; only compiled when `target_arch = "wasm32"` and `--features wasm`.
- **`wasm/src/index.ts`** — TypeScript wrapper over the generated WASM package, providing a typed API for both Node.js and browser environments.

### Optional Cargo features

| Feature | Enables |
|---------|---------|
| `wasm` | wasm-bindgen exports in `wasm.rs` |
| `mathml` | MathML backend via `latex2mathml` |
| `pdf` | PDF rendering via `genpdf` in `render/pdf.rs` |
| `editor` | `mda-preview` GUI binary using `eframe`/`egui` |

## File Extension

Source documents use `.mda` (not `.md`). The whitepaper (`WHITEPAPER.mda`) is itself written in the format the library implements.

## Plans and Design Docs

Never commit plan files or design documents (`docs/plans/`) to git. Keep them local-only as working references.
