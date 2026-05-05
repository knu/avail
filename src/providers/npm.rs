use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::Match;

use super::{SearchOutcome, exact_status};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<NpmPackage>(&format!(
        "https://registry.npmjs.org/{}",
        url_component(name)
    ))?;
    let search = http_get_json::<NpmSearchResponse>(&format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size={}",
        url_component(name),
        limit
    ))?
    .map(|response| response.objects)
    .unwrap_or_default();

    Ok(SearchOutcome {
        status: exact_status(
            exact.as_ref().map(|package| package.name.as_str()),
            name,
            exact
                .as_ref()
                .and_then(|package| package.description.clone()),
        ),
        matches: search
            .into_iter()
            .map(|object| Match {
                url: object.package.links.and_then(|links| links.npm),
                detail: object.package.description,
                name: object.package.name,
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize)]
struct NpmPackage {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NpmSearchResponse {
    objects: Vec<NpmSearchObject>,
}

#[derive(Debug, Deserialize)]
struct NpmSearchObject {
    package: NpmSearchPackage,
}

#[derive(Debug, Deserialize)]
struct NpmSearchPackage {
    name: String,
    description: Option<String>,
    links: Option<NpmLinks>,
}

#[derive(Debug, Deserialize)]
struct NpmLinks {
    npm: Option<String>,
}
