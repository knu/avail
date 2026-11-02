use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::SearchOutcome;

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<PypiPackage>(&format!(
        "https://pypi.org/pypi/{}/json",
        url_component(name)
    ))?;

    Ok(SearchOutcome::from_matches(
        exact
            .map(|package| {
                vec![Match {
                    kind: MatchKind::Package,
                    exact: pypi_names_equal(&package.info.name, name),
                    url: package.info.project_url,
                    detail: package.info.summary,
                    name: package.info.name,
                }]
            })
            .unwrap_or_default(),
        limit,
    ))
}

#[derive(Debug, Deserialize)]
struct PypiPackage {
    info: PypiInfo,
}

#[derive(Debug, Deserialize)]
struct PypiInfo {
    name: String,
    summary: Option<String>,
    #[serde(rename = "project_url")]
    project_url: Option<String>,
}

fn pypi_names_equal(left: &str, right: &str) -> bool {
    normalize_pypi_name(left) == normalize_pypi_name(right)
}

fn normalize_pypi_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut last_was_separator = false;

    for ch in name.chars() {
        if matches!(ch, '-' | '_' | '.') {
            if !last_was_separator {
                normalized.push('-');
                last_was_separator = true;
            }
        } else {
            normalized.extend(ch.to_lowercase());
            last_was_separator = false;
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pypi_names_equal_normalizes_separators() {
        assert!(pypi_names_equal("friendly.bard", "friendly-bard"));
        assert!(pypi_names_equal("friendly__bard", "Friendly-Bard"));
    }
}
