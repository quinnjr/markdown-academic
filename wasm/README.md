# markdown-academic

Academic writing in Markdown - Math, citations, cross-references, and more.

This package provides JavaScript/TypeScript bindings (via WebAssembly) that work in both **Node.js** and **browser** environments.

## Installation

### npm / pnpm / yarn

```bash
npm install markdown-academic
# or
pnpm add markdown-academic
# or
yarn add markdown-academic
```

### CDN (No Build Required)

Use directly in the browser via CDN - no bundler needed:

```html
<!-- Using esm.sh (recommended for ESM) -->
<script type="module">
import { init, render } from 'https://esm.sh/markdown-academic';

await init();
const html = render('# Hello $E=mc^2$');
</script>

<!-- Using jsDelivr -->
<script type="module">
import { init, render } from 'https://cdn.jsdelivr.net/npm/markdown-academic/+esm';

await init();
const html = render('# Hello $E=mc^2$');
</script>

<!-- Using unpkg -->
<script type="module">
import { init, render } from 'https://unpkg.com/markdown-academic?module';

await init();
const html = render('# Hello $E=mc^2$');
</script>
```

## Quick Start

```typescript
import { init, render, MathBackend } from 'markdown-academic';

// Initialize the WASM module (required once)
await init();

// Render markdown to HTML
const html = render('# Hello $E=mc^2$');

// With options
const fullHtml = render(source, {
  standalone: true,
  mathBackend: MathBackend.KaTeX,
  title: 'My Document'
});
```

## API Reference

### `init(wasmPath?: string): Promise<void>`

Initialize the WASM module. Must be called before using any other functions.

```typescript
await init();
// or with custom WASM path
await init('/path/to/markdown_academic_bg.wasm');
```

### `render(input: string, options?: RenderOptions): string`

Render markdown-academic source to HTML.

```typescript
// Simple usage
const html = render('# Hello');

// With options
const html = render(source, {
  standalone: true,      // Generate complete HTML document
  mathBackend: 'katex',  // 'katex', 'mathjax', or 'mathml'
  title: 'My Document',  // Document title
  customCss: 'body { }', // Custom CSS
  includeToc: true,      // Include table of contents
  classPrefix: 'mda',    // CSS class prefix
  strictMode: false      // Error on unresolved refs
});
```

### `parseDocument(input: string): ParsedDocument`

Parse a document and get structured information.

```typescript
const doc = parseDocument(source);

console.log(doc.metadata.title);
console.log(doc.metadata.authors);
console.log(doc.statistics.wordCount);
console.log(doc.labels); // All cross-reference labels
```

### `validate(input: string): ValidationResult`

Validate a document without rendering.

```typescript
const result = validate(source);

if (!result.valid) {
  console.error('Errors:', result.errors);
  console.warn('Warnings:', result.warnings);
}
```

### `parseToJson(input: string): string`

Get the full parsed document as JSON (useful for debugging).

```typescript
const json = parseToJson(source);
console.log(JSON.parse(json));
```

### `getVersion(): string`

Get the library version.

```typescript
console.log(getVersion()); // "0.1.0"
```

### `hasFeature(feature: Feature): boolean`

Check if a feature is supported.

```typescript
if (hasFeature('mathml')) {
  // MathML is available
}
```

## RenderOptions Class

For more control, use the `RenderOptions` class:

```typescript
import { RenderOptions, MathBackend } from 'markdown-academic';

const options = new RenderOptions();
options
  .setStandalone(true)
  .setMathBackend(MathBackend.KaTeX)
  .setTitle('My Document')
  .setCustomCss('body { max-width: 800px; }')
  .setIncludeToc(true);

const html = render(source, options);
```

## Types

### MathBackend

```typescript
enum MathBackend {
  KaTeX = 'katex',    // Fast, client-side (default)
  MathJax = 'mathjax', // Full LaTeX support
  MathML = 'mathml'    // Native browser rendering
}
```

### ParsedDocument

```typescript
interface ParsedDocument {
  metadata: {
    title?: string;
    subtitle?: string;
    authors: string[];
    date?: string;
    keywords: string[];
    institution?: string;
    macros: string[];
    bibliographyPath?: string;
  };
  blocks: BlockInfo[];
  labels: LabelInfo[];
  statistics: {
    blockCount: number;
    headingCount: number;
    equationCount: number;
    citationCount: number;
    figureCount: number;
    tableCount: number;
    footnoteCount: number;
    wordCount: number;
  };
}
```

## Browser Usage

### ES Modules

```html
<script type="module">
import { init, render } from 'markdown-academic';

await init();

const source = document.getElementById('editor').value;
const html = render(source, { standalone: false });
document.getElementById('preview').innerHTML = html;
</script>
```

### With a Bundler (Vite, Webpack, etc.)

```typescript
import { init, render } from 'markdown-academic';

async function setup() {
  await init();
  // Ready to use
}
```

## Node.js Usage

### ESM

```javascript
import { init, render } from 'markdown-academic';

await init();
const html = render('# Hello');
console.log(html);
```

### CommonJS

```javascript
const { init, render } = require('markdown-academic');

async function main() {
  await init();
  const html = render('# Hello');
  console.log(html);
}

main();
```

## Building from Source

Prerequisites:
- Rust toolchain
- wasm-pack (`cargo install wasm-pack`)
- Node.js 18+
- pnpm (recommended)

```bash
# Clone the repository
git clone https://github.com/quinnjr/markdown-academic.git
cd markdown-academic/wasm

# Install dependencies
pnpm install

# Build (compiles Rust to WASM and bundles TypeScript)
pnpm run build
```

## Example: Live Preview Editor (CDN)

This example works without any build step - just save as an HTML file and open in a browser:

```html
<!DOCTYPE html>
<html>
<head>
  <title>MDA Editor</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
  <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>
</head>
<body>
  <div style="display: flex; gap: 20px;">
    <textarea id="editor" style="width: 50%; height: 400px;">
# Introduction {#sec:intro}

The equation $E = mc^2$ demonstrates mass-energy equivalence.

See @sec:intro for more.
    </textarea>
    <div id="preview" style="width: 50%; height: 400px; overflow: auto;"></div>
  </div>

  <script type="module">
    // Load directly from CDN - no npm install needed!
    import { init, render } from 'https://esm.sh/markdown-academic';

    await init();

    const editor = document.getElementById('editor');
    const preview = document.getElementById('preview');

    function update() {
      try {
        preview.innerHTML = render(editor.value);
        renderMathInElement(preview);
      } catch (e) {
        preview.innerHTML = `<pre style="color: red;">${e.message}</pre>`;
      }
    }

    editor.addEventListener('input', update);
    update();
  </script>
</body>
</html>
```

## License

MIT License - see [LICENSE](../LICENSE)

## Author

Joseph R. Quinn <quinn.josephr@protonmail.com>
