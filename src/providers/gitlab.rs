use anyhow::Result;
use serde::Deserialize;

use crate::http::{http_get_json, url_component};
use crate::model::{Match, MatchKind, Status};

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    search_with(name, limit, |page| {
        http_get_json::<Vec<GitlabProject>>(&format!(
            "https://gitlab.com/api/v4/projects?search={}&simple=true&per_page=100&page={page}",
            url_component(name)
        ))?
        .ok_or_else(|| anyhow::anyhow!("GitLab project search not found"))
    })
}

fn search_with(
    name: &str,
    limit: usize,
    mut fetch: impl FnMut(usize) -> Result<Vec<GitlabProject>>,
) -> Result<SearchOutcome> {
    let mut matches = Vec::new();
    // Bound requests while keeping an incomplete search from claiming availability.
    for page in 1..=10 {
        let projects = fetch(page)?;
        let complete = projects.len() < 100;
        matches.extend(projects.into_iter().map(|project| project.into_match(name)));
        if complete || (matches.iter().any(|item| item.exact) && matches.len() >= limit) {
            return Ok(SearchOutcome::from_matches(matches, limit));
        }
    }
    let mut outcome = SearchOutcome::from_matches(matches, limit);
    if matches!(outcome.status, Status::Available) {
        outcome.status = Status::Unknown("GitLab search exceeded 1000 projects".to_string());
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(path: &str) -> GitlabProject {
        GitlabProject {
            path: path.to_string(),
            path_with_namespace: format!("example/{path}"),
            description: None,
            web_url: format!("https://gitlab.com/example/{path}"),
        }
    }

    #[test]
    fn scans_past_fuzzy_display_limit_for_exact_names() {
        let result = search_with("demo", 1, |page| {
            Ok(if page == 1 {
                (0..100)
                    .map(|index| project(&format!("demo-{index}")))
                    .collect()
            } else {
                vec![project("demo")]
            })
        })
        .unwrap();
        assert!(matches!(result.status, Status::Taken(_)));
        assert_eq!(result.matches[0].name, "example/demo");
        assert_eq!(result.matched_kinds, [MatchKind::Repository]);
    }

    #[test]
    fn incomplete_search_is_unknown_even_with_no_displayed_matches() {
        let result = search_with("demo", 0, |_| {
            Ok((0..100)
                .map(|index| project(&format!("demo-{index}")))
                .collect())
        })
        .unwrap();
        assert!(matches!(result.status, Status::Unknown(_)));
        assert!(result.matched_kinds.is_empty());
    }
}

#[derive(Deserialize)]
struct GitlabProject {
    path: String,
    path_with_namespace: String,
    description: Option<String>,
    web_url: String,
}

impl GitlabProject {
    fn into_match(self, query: &str) -> Match {
        Match {
            kind: MatchKind::Repository,
            exact: names_equal(&self.path, query),
            name: self.path_with_namespace,
            detail: self.description,
            url: Some(self.web_url),
        }
    }
}
