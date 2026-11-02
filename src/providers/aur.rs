use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = fetch(&format!("info?arg%5B%5D={}", url_component(name)))?;
    let mut packages = exact.results;
    // AUR substring searches require at least two characters.  Exact lookup does not.
    if limit > 0 && name.chars().count() >= 2 {
        for item in fetch(&format!("search/{}?by=name", url_component(name)))?.results {
            if !packages.iter().any(|existing| existing.name == item.name) {
                packages.push(item);
            }
        }
    }
    Ok(SearchOutcome::from_matches(
        packages
            .into_iter()
            .map(|item| Match {
                kind: MatchKind::Package,
                exact: names_equal(&item.name, name),
                url: Some(format!(
                    "https://aur.archlinux.org/packages/{}",
                    url_component(&item.name)
                )),
                name: item.name,
                detail: item.description,
            })
            .collect(),
        limit,
    ))
}

fn fetch(path: &str) -> Result<AurResponse> {
    let response =
        http_get_json::<AurResponse>(&format!("https://aur.archlinux.org/rpc/v5/{path}"))?
            .ok_or_else(|| anyhow::anyhow!("AUR RPC not found"))?;
    ensure!(
        response.kind != "error",
        "AUR: {}",
        response.error.as_deref().unwrap_or("search failed")
    );
    Ok(response)
}

#[derive(Deserialize)]
struct AurResponse {
    #[serde(rename = "type")]
    kind: String,
    error: Option<String>,
    #[serde(default)]
    results: Vec<AurPackage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AurPackage {
    name: String,
    description: Option<String>,
}
