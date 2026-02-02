//! Test new academic features

use markdown_academic::{parse, render_html, resolve, HtmlConfig, ResolveConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"+++
title = "Test Document"
subtitle = "A Test"
authors = ["John Doe", "Jane Smith"]
keywords = ["test", "markdown", "academic"]
institution = "University"
department = "Computer Science"
advisor = "Dr. Advisor"
+++

# Introduction {#sec:intro}

This is a test with H~2~O (subscript) and x^2^ (superscript).

[sc]Small Caps Text[/sc] is also supported.

## Description Lists

Term One
: This is the definition of term one.
: Additional paragraph for term one.

Term Two
: Definition of term two.

## Citation Styles

These would work with a bibliography file:

- Parenthetical citation style
- Textual citation style  
- Year only style
- Author only style

## Page Break

---pagebreak---

## Appendix

---appendix---

# Appendix A {#app:a}

This is the appendix.

::: abstract
This is the abstract of the document.
:::

::: note
This is a note.
:::

::: warning
This is a warning.
:::
"#;

    let doc = parse(input)?;
    let resolved = resolve(doc, &ResolveConfig::default())?;
    let html = render_html(&resolved, &HtmlConfig::default())?;

    println!("Parsing successful!");
    println!("Title: {:?}", resolved.document.metadata.title);
    println!("Subtitle: {:?}", resolved.document.metadata.subtitle);
    println!("Keywords: {:?}", resolved.document.metadata.keywords);
    println!("Institution: {:?}", resolved.document.metadata.institution);
    println!();

    // Check for specific HTML elements
    let checks = [
        ("<sub>", "subscript"),
        ("<sup>", "superscript"),
        ("small-caps", "small caps"),
        ("<dl>", "description list"),
        ("pagebreak", "page break"),
        ("appendix", "appendix marker"),
        ("abstract", "abstract environment"),
        ("note", "note environment"),
        ("warning", "warning environment"),
    ];

    for (pattern, name) in &checks {
        if html.contains(pattern) {
            println!("✓ Found {}", name);
        } else {
            println!("✗ Missing {} (looking for '{}')", name, pattern);
        }
    }

    println!("\nAll features tested!");
    Ok(())
}
