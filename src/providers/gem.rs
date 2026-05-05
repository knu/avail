use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::Match;

use super::{SearchOutcome, exact_status};

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

    Ok(SearchOutcome {
        status: exact_status(
            exact.as_ref().map(|gem| gem.name.as_str()),
            name,
            exact.as_ref().and_then(|gem| gem.info.clone()),
        ),
        matches: search
            .into_iter()
            .take(limit)
            .map(|gem| Match {
                url: gem.project_uri,
                detail: gem.info,
                name: gem.name,
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize)]
struct GemPackage {
    name: String,
    info: Option<String>,
    project_uri: Option<String>,
}
