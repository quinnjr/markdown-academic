#!/usr/bin/env node
/**
 * Node.js example for @markdown-academic/wasm
 *
 * Run with: node examples/node-example.mjs
 */

import { init, render, parseDocument, validate, getVersion, hasFeature, MathBackend } from '../dist/node/index.mjs';

async function main() {
  console.log('Initializing markdown-academic WASM...');
  await init();
  console.log(`Version: ${getVersion()}\n`);

  // Check features
  console.log('Available features:');
  const features = ['math', 'citations', 'crossref', 'environments', 'footnotes', 'toc', 'mathml'];
  for (const feature of features) {
    console.log(`  ${feature}: ${hasFeature(feature) ? '✓' : '✗'}`);
  }
  console.log();

  // Example document
  const source = `+++
title = "Example Document"
authors = ["Alice", "Bob"]
date = "2026-02-03"
+++

# Introduction {#sec:intro}

This is an example of **markdown-academic**.

## Math Support

Inline math: $E = mc^2$

Display math:

$$
\\int_0^\\infty e^{-x} dx = 1
$$ {#eq:integral}

See Equation @eq:integral.

::: theorem {#thm:main}
Every positive integer is interesting.
:::

See @thm:main for proof by strong induction.
`;

  // Validate
  console.log('Validating document...');
  const validation = validate(source);
  console.log(`  Valid: ${validation.valid}`);
  if (validation.errors.length > 0) {
    console.log('  Errors:', validation.errors);
  }
  console.log();

  // Parse and analyze
  console.log('Parsing document...');
  const doc = parseDocument(source);
  console.log(`  Title: ${doc.metadata.title}`);
  console.log(`  Authors: ${doc.metadata.authors.join(', ')}`);
  console.log(`  Word count: ${doc.statistics.wordCount}`);
  console.log(`  Headings: ${doc.statistics.headingCount}`);
  console.log(`  Equations: ${doc.statistics.equationCount}`);
  console.log(`  Labels: ${doc.labels.map(l => l.label).join(', ')}`);
  console.log();

  // Render HTML fragment
  console.log('Rendering HTML fragment...');
  const fragment = render(source);
  console.log(`  Output length: ${fragment.length} characters`);
  console.log(`  Preview: ${fragment.substring(0, 100)}...`);
  console.log();

  // Render standalone HTML
  console.log('Rendering standalone HTML...');
  const standalone = render(source, {
    standalone: true,
    mathBackend: MathBackend.KaTeX,
    title: 'Generated Document',
  });
  console.log(`  Output length: ${standalone.length} characters`);
  console.log(`  Contains DOCTYPE: ${standalone.includes('<!DOCTYPE html>')}`);
  console.log();

  console.log('Done!');
}

main().catch(console.error);
