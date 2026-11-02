use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = http_get_json::<NpmPackage>(&format!(
        "https://registry.npmjs.org/{}",
        url_component(name)
    ))?;
    let search = http_get_json::<NpmSearchResponse>(&format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size={}",
        url_component(name),
        limit.clamp(1, 250)
    ))?
    .map(|response| response.objects)
    .unwrap_or_default();

    let mut matches: Vec<_> = exact
        .into_iter()
        .map(|package| Match {
            kind: MatchKind::Package,
            exact: names_equal(&package.name, name),
            url: Some(format!(
                "https://www.npmjs.com/package/{}",
                url_component(&package.name)
            )),
            name: package.name,
            detail: package.description,
        })
        .collect();
    for item in search.into_iter().map(|object| Match {
        kind: MatchKind::Package,
        exact: names_equal(&object.package.name, name),
        url: object.package.links.and_then(|links| links.npm),
        detail: object.package.description,
        name: object.package.name,
    }) {
        if !matches.iter().any(|existing| existing.name == item.name) {
            matches.push(item);
        }
    }
    Ok(SearchOutcome::from_matches(matches, limit))
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
