use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let exact = fetch(&format!("name={}", url_component(name)))?;
    let mut matches: Vec<_> = exact
        .results
        .into_iter()
        .map(|item| item.into_match(name))
        .collect();
    if limit > 0 {
        let fuzzy = fetch(&format!("q={}", url_component(name)))?;
        for item in fuzzy.results {
            let item = item.into_match(name);
            if !matches.iter().any(|existing| existing.url == item.url) {
                matches.push(item);
            }
        }
    }
    Ok(SearchOutcome::from_matches(matches, limit))
}

fn fetch(query: &str) -> Result<ArchResponse> {
    let response = http_get_json::<ArchResponse>(&format!(
        "https://archlinux.org/packages/search/json/?{query}"
    ))?
    .ok_or_else(|| anyhow::anyhow!("Arch package search not found"))?;
    ensure!(response.valid, "invalid Arch search query");
    Ok(response)
}

#[derive(Deserialize)]
struct ArchResponse {
    valid: bool,
    results: Vec<ArchPackage>,
}

#[derive(Deserialize)]
struct ArchPackage {
    pkgname: String,
    pkgdesc: Option<String>,
    repo: String,
    arch: String,
}

impl ArchPackage {
    fn into_match(self, query: &str) -> Match {
        Match {
            kind: MatchKind::Package,
            exact: names_equal(&self.pkgname, query),
            url: Some(format!(
                "https://archlinux.org/packages/{}/{}/{}/",
                url_component(&self.repo),
                url_component(&self.arch),
                url_component(&self.pkgname)
            )),
            name: self.pkgname,
            detail: self.pkgdesc,
        }
    }
}
