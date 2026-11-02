use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<GemPackage>(&format!(
        "https://rubygems.org/api/v1/gems/{}.json",
        url_component(name)
    ))?;
    let search = http_get_json::<Vec<GemPackage>>(&format!(
        "https://rubygems.org/api/v1/search.json?query={}",
        url_component(name)
    ))?
    .unwrap_or_default();

    let mut packages = Vec::new();
    packages.extend(exact);
    for gem in search {
        if !packages.iter().any(|existing| existing.name == gem.name) {
            packages.push(gem);
        }
    }
    Ok(SearchOutcome::from_matches(
        packages
            .into_iter()
            .map(|gem| Match {
                kind: MatchKind::Package,
                exact: names_equal(&gem.name, name),
                url: gem.project_uri,
                detail: gem.info,
                name: gem.name,
            })
            .collect(),
        limit,
    ))
}

#[derive(Debug, Deserialize)]
struct GemPackage {
    name: String,
    info: Option<String>,
    project_uri: Option<String>,
}
