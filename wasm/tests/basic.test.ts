/**
 * Basic tests for markdown-academic
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { init, render, parseDocument, validate, getVersion, hasFeature, RenderOptions, MathBackend, createOptions } from '../src/index';

describe('markdown-academic', () => {
  beforeAll(async () => {
    await init();
  });

  describe('init()', () => {
    it('should initialize without error', async () => {
      // Already initialized in beforeAll, but should be safe to call again
      await init();
    });
  });

  describe('getVersion()', () => {
    it('should return a version string', () => {
      const version = getVersion();
      expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });
  });

  describe('hasFeature()', () => {
    it('should report math support', () => {
      expect(hasFeature('math')).toBe(true);
    });

    it('should report citations support', () => {
      expect(hasFeature('citations')).toBe(true);
    });

    it('should report crossref support', () => {
      expect(hasFeature('crossref')).toBe(true);
    });
  });

  describe('render()', () => {
    it('should render simple markdown', () => {
      const html = render('# Hello World');
      expect(html).toContain('Hello World');
      expect(html).toContain('<h1');
    });

    it('should render inline math', () => {
      const html = render('The equation $E=mc^2$ is famous.');
      expect(html).toContain('E=mc^2');
    });

    it('should render display math', () => {
      const html = render('$$\\int_0^\\infty e^{-x} dx = 1$$');
      expect(html).toContain('int');
    });

    it('should handle cross-references', () => {
      const source = `
# Introduction {#sec:intro}

See @sec:intro for details.
`;
      const html = render(source);
      // Labels with colons are converted to hyphens in HTML ids
      expect(html).toContain('sec-intro');
      expect(html).toContain('Section 1');
    });

    it('should render environments', () => {
      const source = `
::: theorem {#thm:main}
Every natural number is interesting.
:::
`;
      const html = render(source);
      expect(html).toContain('theorem');
    });

    it('should accept options object', () => {
      const html = render('# Test', { standalone: true });
      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('<html');
    });

    it('should accept RenderOptions instance', () => {
      const options = new RenderOptions();
      options.setStandalone(true);
      options.setTitle('My Document');
      const html = render('# Test', options);
      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('My Document');
      // Note: WASM takes ownership of options, don't call free() after render
    });
  });

  describe('parseDocument()', () => {
    it('should parse a simple document', () => {
      const doc = parseDocument('# Hello World\n\nSome text.');
      expect(doc.statistics.headingCount).toBe(1);
      expect(doc.statistics.blockCount).toBeGreaterThan(0);
    });

    it('should extract metadata from front matter', () => {
      const source = `+++
title = "Test Document"
authors = ["John Doe"]
+++

# Introduction
`;
      const doc = parseDocument(source);
      expect(doc.metadata.title).toBe('Test Document');
      expect(doc.metadata.authors).toContain('John Doe');
    });

    it('should count equations', () => {
      const source = `
The equation $E=mc^2$ is inline.

$$
\\frac{d}{dx} f(x) = f'(x)
$$
`;
      const doc = parseDocument(source);
      expect(doc.statistics.equationCount).toBeGreaterThanOrEqual(1);
    });

    it('should collect labels', () => {
      const source = `
# Section One {#sec:one}

## Subsection {#sec:sub}

::: theorem {#thm:main}
A theorem.
:::
`;
      const doc = parseDocument(source);
      expect(doc.labels.length).toBeGreaterThanOrEqual(2);
    });
  });

  describe('validate()', () => {
    it('should validate a correct document', () => {
      const result = validate('# Hello World');
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('RenderOptions', () => {
    it('should chain setters', () => {
      const options = new RenderOptions();
      options
        .setMathBackend(MathBackend.KaTeX)
        .setStandalone(true)
        .setTitle('Test')
        .setClassPrefix('test');
      
      expect(options.getMathBackend()).toBe('katex');
      expect(options.getStandalone()).toBe(true);
      expect(options.getTitle()).toBe('Test');
      
      options.free();
    });
  });

  describe('createOptions()', () => {
    it('should create options from config', () => {
      const options = createOptions({
        standalone: true,
        mathBackend: MathBackend.MathJax,
        title: 'Created',
      });
      
      expect(options.getStandalone()).toBe(true);
      expect(options.getMathBackend()).toBe('mathjax');
      expect(options.getTitle()).toBe('Created');
      
      options.free();
    });
  });
});
