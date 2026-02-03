/**
 * markdown-academic
 *
 * Academic writing in Markdown with WebAssembly bindings.
 * Works in both Node.js and browser environments.
 *
 * @example
 * ```typescript
 * import { init, render, RenderOptions, MathBackend } from 'markdown-academic';
 *
 * // Initialize the WASM module (required once before use)
 * await init();
 *
 * // Render markdown to HTML
 * const html = render('# Hello $E=mc^2$');
 *
 * // With options
 * const options = new RenderOptions();
 * options.setStandalone(true);
 * options.setMathBackend('katex');
 * const fullHtml = render(source, options);
 * ```
 *
 * @packageDocumentation
 */

// Import from generated WASM package
import wasmInit, {
  RenderOptions as WasmRenderOptions,
  renderMarkdown as wasmRenderMarkdown,
  parseDocument as wasmParseDocument,
  parseToJson as wasmParseToJson,
  validateDocument as wasmValidateDocument,
  getVersion as wasmGetVersion,
  hasFeature as wasmHasFeature,
  type InitInput,
} from '../dist/pkg/markdown_academic.js';

// ============================================================================
// Types
// ============================================================================

/**
 * Math rendering backend options.
 */
export enum MathBackend {
  /** KaTeX - Fast client-side rendering (default) */
  KaTeX = 'katex',
  /** MathJax - Full LaTeX compatibility */
  MathJax = 'mathjax',
  /** MathML - Native browser rendering, no JS required */
  MathML = 'mathml',
}

/**
 * Options for rendering markdown to HTML.
 */
export interface RenderConfig {
  /** Math rendering backend. @default MathBackend.KaTeX */
  mathBackend?: MathBackend | 'katex' | 'mathjax' | 'mathml';
  /** Generate a complete HTML document. @default false */
  standalone?: boolean;
  /** Document title (for standalone mode). */
  title?: string;
  /** Custom CSS to include. */
  customCss?: string;
  /** Include table of contents. @default true */
  includeToc?: boolean;
  /** CSS class prefix. @default 'mda' */
  classPrefix?: string;
  /** Enable strict mode (throw on unresolved refs). @default false */
  strictMode?: boolean;
}

/**
 * Document metadata from front matter.
 */
export interface DocumentMetadata {
  title?: string;
  subtitle?: string;
  authors: string[];
  date?: string;
  keywords: string[];
  institution?: string;
  macros: string[];
  bibliographyPath?: string;
}

/**
 * Information about a block element.
 */
export interface BlockInfo {
  type: string;
  label?: string;
  level?: number;
  contentPreview?: string;
}

/**
 * Information about a label.
 */
export interface LabelInfo {
  label: string;
  type: string;
}

/**
 * Document statistics.
 */
export interface DocumentStats {
  blockCount: number;
  headingCount: number;
  equationCount: number;
  citationCount: number;
  figureCount: number;
  tableCount: number;
  footnoteCount: number;
  wordCount: number;
}

/**
 * Parsed document information.
 */
export interface ParsedDocument {
  metadata: DocumentMetadata;
  blocks: BlockInfo[];
  labels: LabelInfo[];
  statistics: DocumentStats;
}

/**
 * Validation result.
 */
export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Supported features.
 */
export type Feature =
  | 'math'
  | 'citations'
  | 'crossref'
  | 'environments'
  | 'footnotes'
  | 'toc'
  | 'mathml';

// ============================================================================
// State
// ============================================================================

let initialized = false;
let initPromise: Promise<void> | null = null;

// ============================================================================
// Exported Classes
// ============================================================================

/**
 * Configuration options for rendering.
 * 
 * @example
 * ```typescript
 * const options = new RenderOptions();
 * options.setStandalone(true);
 * options.setMathBackend('katex');
 * const html = render(source, options);
 * ```
 */
export class RenderOptions {
  /** @internal */
  _inner: WasmRenderOptions;

  constructor() {
    if (!initialized) {
      throw new Error('WASM module not initialized. Call init() first.');
    }
    this._inner = new WasmRenderOptions();
  }

  /** Set the math rendering backend: 'katex', 'mathjax', or 'mathml'. */
  setMathBackend(backend: MathBackend | 'katex' | 'mathjax' | 'mathml'): this {
    this._inner.setMathBackend(backend);
    return this;
  }

  /** Get the current math backend. */
  getMathBackend(): string {
    return this._inner.getMathBackend();
  }

  /** Set whether to generate a complete HTML document. */
  setStandalone(standalone: boolean): this {
    this._inner.setStandalone(standalone);
    return this;
  }

  /** Get standalone setting. */
  getStandalone(): boolean {
    return this._inner.getStandalone();
  }

  /** Set the document title (for standalone mode). */
  setTitle(title: string): this {
    this._inner.setTitle(title);
    return this;
  }

  /** Get the document title. */
  getTitle(): string | undefined {
    return this._inner.getTitle();
  }

  /** Set custom CSS to include. */
  setCustomCss(css: string): this {
    this._inner.setCustomCss(css);
    return this;
  }

  /** Set whether to include table of contents. */
  setIncludeToc(include: boolean): this {
    this._inner.setIncludeToc(include);
    return this;
  }

  /** Set the CSS class prefix. */
  setClassPrefix(prefix: string): this {
    this._inner.setClassPrefix(prefix);
    return this;
  }

  /** Enable or disable strict mode. */
  setStrictMode(strict: boolean): this {
    this._inner.setStrictMode(strict);
    return this;
  }

  /** Free the underlying WASM memory. */
  free(): void {
    this._inner.free();
  }
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Initialize the WASM module.
 *
 * Must be called before using any other functions.
 * Safe to call multiple times (subsequent calls are no-ops).
 *
 * @example
 * ```typescript
 * await init();
 * const html = render('# Hello');
 * ```
 *
 * @param wasmPath - Optional custom path/URL to the WASM file
 */
export async function init(wasmPath?: InitInput): Promise<void> {
  if (initialized) return;

  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      // In Node.js, we need to read the WASM file from disk
      if (typeof globalThis.fetch === 'undefined' || typeof window === 'undefined') {
        // Node.js environment - read file from disk
        const fs = await import('fs');
        const path = await import('path');
        const url = await import('url');
        
        let wasmBuffer: BufferSource;
        
        if (wasmPath instanceof Uint8Array || wasmPath instanceof ArrayBuffer) {
          wasmBuffer = wasmPath;
        } else {
        // Resolve path to the WASM file
        const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
        // When running from source: src/ -> ../dist/pkg/
        // When running from bundle: dist/node/ -> ../pkg/
        // Try both locations
        let wasmFile = wasmPath?.toString();
        if (!wasmFile) {
          const bundlePath = path.resolve(__dirname, '../pkg/markdown_academic_bg.wasm');
          const sourcePath = path.resolve(__dirname, '../dist/pkg/markdown_academic_bg.wasm');
          wasmFile = fs.existsSync(bundlePath) ? bundlePath : sourcePath;
        }
          wasmBuffer = fs.readFileSync(wasmFile);
        }
        
        await wasmInit(wasmBuffer);
      } else {
        // Browser environment - use fetch
        await wasmInit(wasmPath);
      }
      initialized = true;
    } catch (error) {
      initPromise = null;
      throw new Error(`Failed to initialize WASM module: ${error}`);
    }
  })();

  return initPromise;
}

/**
 * Check if the WASM module is initialized.
 */
export function isInitialized(): boolean {
  return initialized;
}

/**
 * Render markdown-academic source to HTML.
 *
 * @example
 * ```typescript
 * // Simple usage
 * const html = render('# Hello $E=mc^2$');
 *
 * // With config object
 * const html = render(source, { standalone: true, mathBackend: 'katex' });
 *
 * // With RenderOptions instance
 * const options = new RenderOptions();
 * options.setStandalone(true);
 * const html = render(source, options);
 * ```
 */
export function render(input: string, options?: RenderConfig | RenderOptions): string {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }

  if (!options) {
    return wasmRenderMarkdown(input, null);
  }

  if (options instanceof RenderOptions) {
    return wasmRenderMarkdown(input, options._inner);
  }

  // Create options from config object
  const opts = new RenderOptions();
  if (options.mathBackend) opts.setMathBackend(options.mathBackend);
  if (options.standalone !== undefined) opts.setStandalone(options.standalone);
  if (options.title) opts.setTitle(options.title);
  if (options.customCss) opts.setCustomCss(options.customCss);
  if (options.includeToc !== undefined) opts.setIncludeToc(options.includeToc);
  if (options.classPrefix) opts.setClassPrefix(options.classPrefix);
  if (options.strictMode !== undefined) opts.setStrictMode(options.strictMode);

  // Note: WASM takes ownership of the options, don't call free() after
  return wasmRenderMarkdown(input, opts._inner);
}

/** Alias for render(). */
export const renderMarkdown = render;

/**
 * Parse a markdown document and return structured information.
 *
 * @example
 * ```typescript
 * const doc = parseDocument(source);
 * console.log(doc.metadata.title);
 * console.log(doc.statistics.wordCount);
 * ```
 */
export function parseDocument(input: string): ParsedDocument {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }

  const result = wasmParseDocument(input);

  // Convert snake_case to camelCase
  return {
    metadata: {
      title: result.metadata?.title,
      subtitle: result.metadata?.subtitle,
      authors: result.metadata?.authors || [],
      date: result.metadata?.date,
      keywords: result.metadata?.keywords || [],
      institution: result.metadata?.institution,
      macros: result.metadata?.macros || [],
      bibliographyPath: result.metadata?.bibliography_path,
    },
    blocks: (result.blocks || []).map((b: any) => ({
      type: b.type,
      label: b.label,
      level: b.level,
      contentPreview: b.content_preview,
    })),
    labels: (result.labels || []).map((l: any) => ({
      label: l.label,
      type: l.type,
    })),
    statistics: {
      blockCount: result.statistics?.block_count || 0,
      headingCount: result.statistics?.heading_count || 0,
      equationCount: result.statistics?.equation_count || 0,
      citationCount: result.statistics?.citation_count || 0,
      figureCount: result.statistics?.figure_count || 0,
      tableCount: result.statistics?.table_count || 0,
      footnoteCount: result.statistics?.footnote_count || 0,
      wordCount: result.statistics?.word_count || 0,
    },
  };
}

/**
 * Parse markdown and return the full structure as JSON.
 */
export function parseToJson(input: string): string {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }
  return wasmParseToJson(input);
}

/**
 * Validate a markdown document without rendering.
 *
 * @example
 * ```typescript
 * const result = validate(source);
 * if (!result.valid) {
 *   console.error('Errors:', result.errors);
 * }
 * ```
 */
export function validate(input: string): ValidationResult {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }
  return wasmValidateDocument(input) as ValidationResult;
}

/**
 * Get the library version.
 */
export function getVersion(): string {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }
  return wasmGetVersion();
}

/**
 * Check if a feature is supported.
 *
 * @example
 * ```typescript
 * if (hasFeature('mathml')) {
 *   options.setMathBackend('mathml');
 * }
 * ```
 */
export function hasFeature(feature: Feature): boolean {
  if (!initialized) {
    throw new Error('WASM module not initialized. Call init() first.');
  }
  return wasmHasFeature(feature);
}

/**
 * Create RenderOptions from a config object.
 *
 * @example
 * ```typescript
 * const options = createOptions({ standalone: true, mathBackend: 'katex' });
 * const html = render(source, options);
 * ```
 */
export function createOptions(config: RenderConfig): RenderOptions {
  const opts = new RenderOptions();
  if (config.mathBackend) opts.setMathBackend(config.mathBackend);
  if (config.standalone !== undefined) opts.setStandalone(config.standalone);
  if (config.title) opts.setTitle(config.title);
  if (config.customCss) opts.setCustomCss(config.customCss);
  if (config.includeToc !== undefined) opts.setIncludeToc(config.includeToc);
  if (config.classPrefix) opts.setClassPrefix(config.classPrefix);
  if (config.strictMode !== undefined) opts.setStrictMode(config.strictMode);
  return opts;
}
