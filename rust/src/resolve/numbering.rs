//! Automatic numbering for sections, environments, equations, etc.

use crate::ast::{Block, Document, EnvironmentKind};
use std::collections::HashMap;

/// Assign numbers to all numbered elements in the document.
/// Returns (section_numbers, env_numbers).
pub fn assign_numbers(document: &Document) -> (HashMap<String, String>, HashMap<String, u32>) {
    let mut section_numbers = HashMap::new();
    let mut env_numbers = HashMap::new();

    // Counters
    let mut section_counters = [0u32; 6]; // h1..h6
    let mut equation_counter = 0u32;
    let mut figure_counter = 0u32;
    let mut table_counter = 0u32;
    let mut theorem_counter = 0u32;
    let mut lemma_counter = 0u32;
    let mut definition_counter = 0u32;
    let mut example_counter = 0u32;
    let mut algorithm_counter = 0u32;

    for block in &document.blocks {
        assign_block_numbers(
            block,
            &mut section_counters,
            &mut section_numbers,
            &mut env_numbers,
            &mut equation_counter,
            &mut figure_counter,
            &mut table_counter,
            &mut theorem_counter,
            &mut lemma_counter,
            &mut definition_counter,
            &mut example_counter,
            &mut algorithm_counter,
        );
    }

    (section_numbers, env_numbers)
}

#[allow(clippy::too_many_arguments)]
fn assign_block_numbers(
    block: &Block,
    section_counters: &mut [u32; 6],
    section_numbers: &mut HashMap<String, String>,
    env_numbers: &mut HashMap<String, u32>,
    equation_counter: &mut u32,
    figure_counter: &mut u32,
    table_counter: &mut u32,
    theorem_counter: &mut u32,
    lemma_counter: &mut u32,
    definition_counter: &mut u32,
    example_counter: &mut u32,
    algorithm_counter: &mut u32,
) {
    match block {
        Block::Heading { level, label, .. } => {
            let idx = (*level as usize).saturating_sub(1).min(5);

            // Increment this level's counter
            section_counters[idx] += 1;

            // Reset lower level counters
            for i in (idx + 1)..6 {
                section_counters[i] = 0;
            }

            if let Some(lbl) = label {
                // Build section number string
                let number = build_section_number(section_counters, idx);
                section_numbers.insert(lbl.clone(), number);
            }
        }
        Block::DisplayMath { label, .. } => {
            *equation_counter += 1;
            if let Some(lbl) = label {
                env_numbers.insert(lbl.clone(), *equation_counter);
            }
        }
        Block::Environment { kind, label, content, .. } => {
            let counter = match kind {
                EnvironmentKind::Theorem | EnvironmentKind::Proposition | EnvironmentKind::Corollary => {
                    *theorem_counter += 1;
                    Some(*theorem_counter)
                }
                EnvironmentKind::Lemma => {
                    *lemma_counter += 1;
                    Some(*lemma_counter)
                }
                EnvironmentKind::Definition => {
                    *definition_counter += 1;
                    Some(*definition_counter)
                }
                EnvironmentKind::Example | EnvironmentKind::Remark => {
                    *example_counter += 1;
                    Some(*example_counter)
                }
                EnvironmentKind::Figure => {
                    *figure_counter += 1;
                    Some(*figure_counter)
                }
                EnvironmentKind::Table => {
                    *table_counter += 1;
                    Some(*table_counter)
                }
                EnvironmentKind::Algorithm => {
                    *algorithm_counter += 1;
                    Some(*algorithm_counter)
                }
                EnvironmentKind::Proof => None, // Proofs are not numbered
                EnvironmentKind::Custom(_) => None, // Custom environments not numbered by default
            };

            if let (Some(lbl), Some(num)) = (label, counter) {
                env_numbers.insert(lbl.clone(), num);
            }

            // Process nested blocks
            for inner in content {
                assign_block_numbers(
                    inner,
                    section_counters,
                    section_numbers,
                    env_numbers,
                    equation_counter,
                    figure_counter,
                    table_counter,
                    theorem_counter,
                    lemma_counter,
                    definition_counter,
                    example_counter,
                    algorithm_counter,
                );
            }
        }
        Block::Table { label, .. } => {
            *table_counter += 1;
            if let Some(lbl) = label {
                env_numbers.insert(lbl.clone(), *table_counter);
            }
        }
        Block::BlockQuote(blocks) => {
            for inner in blocks {
                assign_block_numbers(
                    inner,
                    section_counters,
                    section_numbers,
                    env_numbers,
                    equation_counter,
                    figure_counter,
                    table_counter,
                    theorem_counter,
                    lemma_counter,
                    definition_counter,
                    example_counter,
                    algorithm_counter,
                );
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for inner in &item.content {
                    assign_block_numbers(
                        inner,
                        section_counters,
                        section_numbers,
                        env_numbers,
                        equation_counter,
                        figure_counter,
                        table_counter,
                        theorem_counter,
                        lemma_counter,
                        definition_counter,
                        example_counter,
                        algorithm_counter,
                    );
                }
            }
        }
        _ => {}
    }
}

fn build_section_number(counters: &[u32; 6], max_level: usize) -> String {
    counters[..=max_level]
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_section_numbering() {
        let input = r#"
# First {#sec:first}

## Sub One {#sec:sub1}

## Sub Two {#sec:sub2}

# Second {#sec:second}
"#;

        let doc = parse(input).unwrap();
        let (section_numbers, _) = assign_numbers(&doc);

        assert_eq!(section_numbers.get("sec:first").map(String::as_str), Some("1"));
        assert_eq!(section_numbers.get("sec:sub1").map(String::as_str), Some("1.1"));
        assert_eq!(section_numbers.get("sec:sub2").map(String::as_str), Some("1.2"));
        assert_eq!(section_numbers.get("sec:second").map(String::as_str), Some("2"));
    }

    #[test]
    fn test_environment_numbering() {
        let input = r#"
::: theorem {#thm:one}
First theorem.
:::

::: theorem {#thm:two}
Second theorem.
:::

::: lemma {#lem:one}
A lemma.
:::
"#;

        let doc = parse(input).unwrap();
        let (_, env_numbers) = assign_numbers(&doc);

        assert_eq!(env_numbers.get("thm:one"), Some(&1));
        assert_eq!(env_numbers.get("thm:two"), Some(&2));
        assert_eq!(env_numbers.get("lem:one"), Some(&1));
    }
}
