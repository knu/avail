use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::model::{Match, MatchKind};
use crate::process;

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let output = process::run("mise", &["registry", "--json"], 10)?;
    ensure!(
        output.status.success(),
        "mise registry failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let tools: Vec<Tool> = serde_json::from_slice(&output.stdout)?;
    Ok(registry_result(tools, name, limit))
}

fn registry_result(tools: Vec<Tool>, query: &str, limit: usize) -> SearchOutcome {
    let mut matches = Vec::new();
    for tool in tools {
        // Older mise versions return aliases only in the canonical entry.
        for name in std::iter::once(&tool.short).chain(&tool.aliases) {
            if name.to_lowercase().contains(&query.to_lowercase())
                && !matches.iter().any(|item: &Match| item.name == *name)
            {
                matches.push(Match {
                    kind: MatchKind::ToolAlias,
                    exact: names_equal(name, query),
                    name: name.clone(),
                    detail: Some(tool.backends.join(", ")),
                    url: Some("https://mise.jdx.dev/registry.html".to_string()),
                });
            }
        }
    }
    SearchOutcome::from_matches(matches, limit)
}

#[derive(Deserialize)]
struct Tool {
    short: String,
    backends: Vec<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    #[test]
    fn aliases_are_taken_even_without_a_separate_row() {
        let tools = serde_json::from_str(
            r#"[{"short":"ripgrep","backends":["cargo:ripgrep"],"aliases":["rg"]}]"#,
        )
        .unwrap();
        let result = registry_result(tools, "rg", 0);
        assert!(matches!(result.status, Status::Taken(_)));
        assert_eq!(result.matched_kinds, [MatchKind::ToolAlias]);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn backend_names_do_not_claim_a_shorthand() {
        let tools =
            serde_json::from_str(r#"[{"short":"demo","backends":["github:example/other"]}]"#)
                .unwrap();
        assert!(matches!(
            registry_result(tools, "other", 5).status,
            Status::Available
        ));
    }
}
