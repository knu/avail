use anyhow::{Context, Result};
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let index = http_get_json::<ServiceIndex>("https://api.nuget.org/v3/index.json")?
        .context("NuGet service index not found")?;
    let base = index.resource("PackageBaseAddress/3.0.0")?;
    let exact = http_get_json::<Versions>(&format!(
        "{}{}/index.json",
        base,
        url_component(&name.to_ascii_lowercase())
    ))?;
    let mut matches = Vec::new();
    if exact.is_some_and(|item| !item.versions.is_empty()) {
        matches.push(package_match(name.to_string(), None, name));
    }
    if limit > 0 {
        let search_url = index.resource("SearchQueryService")?;
        let response = http_get_json::<SearchResponse>(&format!(
            "{search_url}?q={}&take={}&prerelease=true&semVerLevel=2.0.0",
            url_component(name),
            limit.clamp(1, 1000)
        ))?
        .context("NuGet search service not found")?;
        for package in response.data {
            if let Some(existing) = matches
                .iter_mut()
                .find(|item| names_equal(&item.name, &package.id))
            {
                existing.detail = package.description;
            } else {
                matches.push(package_match(package.id, package.description, name));
            }
        }
    }
    Ok(SearchOutcome::from_matches(matches, limit))
}

fn package_match(id: String, description: Option<String>, query: &str) -> Match {
    Match {
        kind: MatchKind::Package,
        exact: names_equal(&id, query),
        url: Some(format!(
            "https://www.nuget.org/packages/{}",
            url_component(&id)
        )),
        name: id,
        detail: description,
    }
}

#[derive(Deserialize)]
struct ServiceIndex {
    resources: Vec<Resource>,
}

impl ServiceIndex {
    fn resource(&self, kind: &str) -> Result<&str> {
        self.resources
            .iter()
            .find(|item| item.kind == kind)
            .map(|item| item.id.as_str())
            .with_context(|| format!("NuGet resource {kind} not found"))
    }
}

#[derive(Deserialize)]
struct Resource {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    kind: String,
}

#[derive(Deserialize)]
struct Versions {
    versions: Vec<String>,
}
#[derive(Deserialize)]
struct SearchResponse {
    data: Vec<Package>,
}
#[derive(Deserialize)]
struct Package {
    id: String,
    description: Option<String>,
}
