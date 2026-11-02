use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<CrateResponse>(&format!(
        "https://crates.io/api/v1/crates/{}",
        url_component(name)
    ))?
    .map(|response| response.krate);

    let search = http_get_json::<CratesSearchResponse>(&format!(
        "https://crates.io/api/v1/crates?q={}&per_page={}",
        url_component(name),
        limit.clamp(1, 100)
    ))?
    .map(|response| response.crates)
    .unwrap_or_default();

    let mut packages = Vec::new();
    packages.extend(exact);
    for package in search {
        if !packages.iter().any(|existing| existing.id == package.id) {
            packages.push(package);
        }
    }
    Ok(SearchOutcome::from_matches(
        packages
            .into_iter()
            .map(|krate| Match {
                kind: MatchKind::Package,
                exact: names_equal(&krate.id, name),
                url: Some(format!("https://crates.io/crates/{}", krate.id)),
                detail: krate.description,
                name: krate.id,
            })
            .collect(),
        limit,
    ))
}

#[derive(Debug, Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    krate: CrateSummary,
}

#[derive(Debug, Deserialize)]
struct CratesSearchResponse {
    crates: Vec<CrateSummary>,
}

#[derive(Debug, Deserialize)]
struct CrateSummary {
    id: String,
    description: Option<String>,
}
