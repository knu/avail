use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, Status};

use super::SearchOutcome;

pub(super) fn search(name: &str, _limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<PypiPackage>(&format!(
        "https://pypi.org/pypi/{}/json",
        url_component(name)
    ))?;

    Ok(SearchOutcome {
        status: exact_status(
            exact.as_ref().map(|package| package.info.name.as_str()),
            name,
            exact
                .as_ref()
                .and_then(|package| package.info.summary.clone()),
        ),
        matches: exact
            .map(|package| {
                vec![Match {
                    url: package.info.project_url,
                    detail: package.info.summary,
                    name: package.info.name,
                }]
            })
            .unwrap_or_default(),
    })
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

fn exact_status(exact_name: Option<&str>, query: &str, detail: Option<String>) -> Status {
    match exact_name {
        Some(name) if pypi_names_equal(name, query) => Status::Taken(
            detail
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| name.to_string()),
        ),
        _ => Status::Available,
    }
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
