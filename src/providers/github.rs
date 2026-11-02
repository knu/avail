use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::ErrorKind;
use std::process::Command;

use crate::model::{Match, MatchKind, Status};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    const GITHUB_EXACT_MATCH_SCAN: usize = 100;

    let query = format!("{name} in:name");
    let scan_limit = limit.clamp(GITHUB_EXACT_MATCH_SCAN, 1000);
    let fetch_limit = scan_limit.to_string();
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

    let repos: Vec<GithubRepo> = serde_json::from_slice(&output.stdout)?;
    let complete = repos.len() < scan_limit;
    let matches = repos
        .into_iter()
        .map(|repo| Match {
            kind: MatchKind::Repository,
            exact: repo
                .full_name
                .rsplit('/')
                .next()
                .is_some_and(|repo| names_equal(repo, name)),
            name: repo.full_name,
            detail: repo.description,
            url: Some(repo.url),
        })
        .collect::<Vec<_>>();

    let mut outcome = SearchOutcome::from_matches(matches, limit);
    if !complete && matches!(outcome.status, Status::Available) {
        outcome.status = Status::Unknown(
            "GitHub search reached its result limit without an exact match".to_string(),
        );
    }
    Ok(outcome)
}

#[derive(Debug, Deserialize)]
struct GithubRepo {
    #[serde(rename = "fullName")]
    full_name: String,
    description: Option<String>,
    url: String,
}
