use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::ErrorKind;
use std::process::Command;

use crate::model::{Match, Status};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    const GITHUB_EXACT_MATCH_SCAN: usize = 100;

    let query = format!("{name} in:name");
    let fetch_limit = limit.max(GITHUB_EXACT_MATCH_SCAN).to_string();
    let output = Command::new("gh")
        .args([
            "search",
            "repos",
            &query,
            "--limit",
            &fetch_limit,
            "--json",
            "fullName,description,url",
        ])
        .output()
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                anyhow::bail!("gh command not found")
            } else {
                Err(err).context("failed to run gh search repos")
            }
        })?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let mut repos: Vec<GithubRepo> = serde_json::from_slice(&output.stdout)?;
    let exact_index = repos.iter().position(|repo| {
        repo.full_name
            .rsplit('/')
            .next()
            .is_some_and(|repo| names_equal(repo, name))
    });
    let status = exact_index
        .and_then(|index| repos.get(index))
        .map(|repo| Status::Taken(repo.full_name.clone()))
        .unwrap_or(Status::Available);
    if let Some(index) = exact_index {
        repos.swap(0, index);
    }
    let matches = repos
        .into_iter()
        .take(limit)
        .map(|repo| Match {
            name: repo.full_name,
            detail: repo.description,
            url: Some(repo.url),
        })
        .collect::<Vec<_>>();

    Ok(SearchOutcome { status, matches })
}

#[derive(Debug, Deserialize)]
struct GithubRepo {
    #[serde(rename = "fullName")]
    full_name: String,
    description: Option<String>,
    url: String,
}
