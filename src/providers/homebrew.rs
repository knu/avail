use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

use crate::http::http_get_json;
use crate::model::{Match, Status};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    if let Ok(output) = Command::new("brew").args(["search", name]).output()
        && output.status.success()
    {
        return Ok(homebrew_result_from_names(
            parse_homebrew_search_output(&String::from_utf8_lossy(&output.stdout)),
            name,
            limit,
        ));
    }

    let formulae =
        http_get_json::<Vec<HomebrewPackage>>("https://formulae.brew.sh/api/formula.json")?
            .unwrap_or_default();
    let casks =
        http_get_json::<Vec<HomebrewCaskPackage>>("https://formulae.brew.sh/api/cask.json")?
            .unwrap_or_default();

    Ok(homebrew_result_from_names(
        formulae
            .into_iter()
            .map(|package| HomebrewEntry::formula(package.name))
            .chain(
                casks
                    .into_iter()
                    .map(|package| HomebrewEntry::cask(package.token)),
            )
            .filter(|package| package.matches_query(name)),
        name,
        limit,
    ))
}

fn homebrew_result_from_names<I>(names: I, query: &str, limit: usize) -> SearchOutcome
where
    I: IntoIterator<Item = HomebrewEntry>,
{
    let mut status = Status::Available;
    let mut matches = Vec::with_capacity(limit);
    for entry in names {
        if matches!(status, Status::Available) && names_equal(&entry.name, query) {
            status = Status::Taken(entry.name.clone());
        }
        if matches.len() < limit {
            matches.push(Match {
                url: Some(entry.url()),
                detail: None,
                name: entry.name,
            });
        }
    }

    SearchOutcome { status, matches }
}

fn parse_homebrew_search_output(output: &str) -> Vec<HomebrewEntry> {
    let mut kind = HomebrewKind::Formula;
    let mut entries = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(heading) = line.strip_prefix("==>") {
            let heading = heading.trim();
            if heading.eq_ignore_ascii_case("casks") {
                kind = HomebrewKind::Cask;
            } else if heading.eq_ignore_ascii_case("formulae")
                || heading.eq_ignore_ascii_case("formula")
            {
                kind = HomebrewKind::Formula;
            }
            continue;
        }

        entries.extend(line.split_whitespace().map(|name| HomebrewEntry {
            name: name.to_string(),
            kind,
        }));
    }

    entries
}

#[derive(Debug, Deserialize)]
struct HomebrewPackage {
    name: String,
}

#[derive(Debug, Deserialize)]
struct HomebrewCaskPackage {
    token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HomebrewKind {
    Formula,
    Cask,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HomebrewEntry {
    name: String,
    kind: HomebrewKind,
}

impl HomebrewEntry {
    fn formula(name: String) -> Self {
        Self {
            name,
            kind: HomebrewKind::Formula,
        }
    }

    fn cask(name: String) -> Self {
        Self {
            name,
            kind: HomebrewKind::Cask,
        }
    }

    fn matches_query(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase())
    }

    fn url(&self) -> String {
        let kind = match self.kind {
            HomebrewKind::Formula => "formula",
            HomebrewKind::Cask => "cask",
        };
        format!("https://formulae.brew.sh/{kind}/{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_result_from_names_marks_exact_match_taken() {
        let result = homebrew_result_from_names(
            [
                HomebrewEntry::formula("goimports".to_string()),
                HomebrewEntry::formula("go".to_string()),
            ],
            "go",
            5,
        );

        assert!(matches!(result.status, Status::Taken(ref name) if name == "go"));
        assert_eq!(result.matches.len(), 2);
    }

    #[test]
    fn test_homebrew_result_from_names_finds_exact_match_past_limit() {
        let names = vec![
            HomebrewEntry::formula("goimports".to_string()),
            HomebrewEntry::formula("golangci-lint".to_string()),
            HomebrewEntry::formula("gopls".to_string()),
            HomebrewEntry::formula("go".to_string()),
        ];
        let result = homebrew_result_from_names(names, "go", 2);

        assert!(matches!(result.status, Status::Taken(ref name) if name == "go"));
        assert_eq!(result.matches.len(), 2);
    }

    #[test]
    fn test_parse_homebrew_search_output_tracks_casks() {
        let entries = parse_homebrew_search_output(
            "\
==> Formulae
go  goimports

==> Casks
go-server
",
        );

        assert_eq!(
            entries,
            [
                HomebrewEntry::formula("go".to_string()),
                HomebrewEntry::formula("goimports".to_string()),
                HomebrewEntry::cask("go-server".to_string()),
            ]
        );
        assert_eq!(entries[2].url(), "https://formulae.brew.sh/cask/go-server");
    }
}
