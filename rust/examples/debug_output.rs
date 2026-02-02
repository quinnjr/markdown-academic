//! Debug HTML output

use markdown_academic::{parse, render_html, resolve, HtmlConfig, ResolveConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"H~2~O and x^2^

---pagebreak---
"#;

    let doc = parse(input)?;
    let resolved = resolve(doc, &ResolveConfig::default())?;
    let html = render_html(&resolved, &HtmlConfig::default())?;

    println!("HTML Output:");
    println!("{}", html);
    Ok(())
}
