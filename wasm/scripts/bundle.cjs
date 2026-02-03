#!/usr/bin/env node
/**
 * Build script for bundling the TypeScript source into various module formats.
 */

const esbuild = require('esbuild');
const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, '..', 'src');
const distDir = path.join(__dirname, '..', 'dist');

// Plugin to rewrite the WASM import paths
const wasmImportPlugin = {
  name: 'wasm-import-rewriter',
  setup(build) {
    // Resolve imports of the WASM pkg
    build.onResolve({ filter: /\.\.\/dist\/pkg\/markdown_academic\.js$/ }, (args) => {
      return {
        path: '../pkg/markdown_academic.js',
        external: true,
      };
    });
  },
};

async function build() {
  console.log('Building bundles...\n');

  // Ensure output directories exist
  fs.mkdirSync(path.join(distDir, 'browser'), { recursive: true });
  fs.mkdirSync(path.join(distDir, 'esm'), { recursive: true });
  fs.mkdirSync(path.join(distDir, 'node'), { recursive: true });

  const commonOptions = {
    entryPoints: [path.join(srcDir, 'index.ts')],
    bundle: true,
    sourcemap: true,
    plugins: [wasmImportPlugin],
  };

  // ESM bundle for browsers
  await esbuild.build({
    ...commonOptions,
    outfile: path.join(distDir, 'browser', 'index.js'),
    format: 'esm',
    platform: 'browser',
    target: 'es2020',
  });
  console.log('✓ Browser ESM bundle (dist/browser/index.js)');

  // ESM bundle (generic)
  await esbuild.build({
    ...commonOptions,
    outfile: path.join(distDir, 'esm', 'index.js'),
    format: 'esm',
    platform: 'neutral',
    target: 'es2020',
  });
  console.log('✓ ESM bundle (dist/esm/index.js)');

  // CommonJS bundle for Node.js
  await esbuild.build({
    ...commonOptions,
    outfile: path.join(distDir, 'node', 'index.js'),
    format: 'cjs',
    platform: 'node',
    target: 'node18',
  });
  console.log('✓ Node.js CJS bundle (dist/node/index.js)');

  // ESM bundle for Node.js
  await esbuild.build({
    ...commonOptions,
    outfile: path.join(distDir, 'node', 'index.mjs'),
    format: 'esm',
    platform: 'node',
    target: 'node18',
  });
  console.log('✓ Node.js ESM bundle (dist/node/index.mjs)');

  // Generate type declarations
  const typesDir = path.join(distDir, 'types');
  fs.mkdirSync(typesDir, { recursive: true });

  const typeContent = `/**
 * markdown-academic - Type Definitions
 */

// Re-export WASM types
export * from '../pkg/markdown_academic.d.ts';

// Enums
export declare enum MathBackend {
  KaTeX = 'katex',
  MathJax = 'mathjax',
  MathML = 'mathml'
}

// Configuration interfaces
export interface RenderConfig {
  mathBackend?: MathBackend | 'katex' | 'mathjax' | 'mathml';
  standalone?: boolean;
  title?: string;
  customCss?: string;
  includeToc?: boolean;
  classPrefix?: string;
  strictMode?: boolean;
}

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

export interface BlockInfo {
  type: string;
  label?: string;
  level?: number;
  contentPreview?: string;
}

export interface LabelInfo {
  label: string;
  type: string;
}

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

export interface ParsedDocument {
  metadata: DocumentMetadata;
  blocks: BlockInfo[];
  labels: LabelInfo[];
  statistics: DocumentStats;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export type Feature = 'math' | 'citations' | 'crossref' | 'environments' | 'footnotes' | 'toc' | 'mathml';

// API functions
export declare function init(wasmPath?: any): Promise<void>;
export declare function isInitialized(): boolean;
export declare function render(input: string, options?: RenderConfig | RenderOptions): string;
export declare const renderMarkdown: typeof render;
export declare function parseDocument(input: string): ParsedDocument;
export declare function parseToJson(input: string): string;
export declare function validate(input: string): ValidationResult;
export declare function getVersion(): string;
export declare function hasFeature(feature: Feature): boolean;
export declare function createOptions(config: RenderConfig): RenderOptions;

// RenderOptions class
export declare class RenderOptions {
  constructor();
  setMathBackend(backend: MathBackend | string): this;
  getMathBackend(): string;
  setStandalone(standalone: boolean): this;
  getStandalone(): boolean;
  setTitle(title: string): this;
  getTitle(): string | undefined;
  setCustomCss(css: string): this;
  setIncludeToc(include: boolean): this;
  setClassPrefix(prefix: string): this;
  setStrictMode(strict: boolean): this;
  free(): void;
}
`;

  fs.writeFileSync(path.join(typesDir, 'index.d.ts'), typeContent);
  console.log('✓ Type declarations (dist/types/index.d.ts)');

  // Clean up wasm-pack's .gitignore that prevents npm from including the pkg files
  const pkgGitignore = path.join(distDir, 'pkg', '.gitignore');
  if (fs.existsSync(pkgGitignore)) {
    fs.unlinkSync(pkgGitignore);
    console.log('✓ Cleaned up dist/pkg/.gitignore for npm publishing');
  }

  console.log('\n✨ Build complete!');
}

build().catch((err) => {
  console.error('\n❌ Build failed:', err);
  process.exit(1);
});
