use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::Match;

use super::{SearchOutcome, exact_status};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<CrateResponse>(&format!(
        "https://crates.io/api/v1/crates/{}",
        url_component(name)
    ))?
    .map(|response| response.krate);

    let search = http_get_json::<CratesSearchResponse>(&format!(
        "https://crates.io/api/v1/crates?q={}&per_page={}",
        url_component(name),
        limit
    ))?
    .map(|response| response.crates)
    .unwrap_or_default();

    Ok(SearchOutcome {
        status: exact_status(
            exact.as_ref().map(|krate| krate.id.as_str()),
            name,
            exact.as_ref().and_then(|krate| krate.description.clone()),
        ),
        matches: search
            .into_iter()
            .map(|krate| Match {
                url: Some(format!("https://crates.io/crates/{}", krate.id)),
                detail: krate.description,
                name: krate.id,
            })
            .collect(),
    })
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
