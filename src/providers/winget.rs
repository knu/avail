use anyhow::{Result, bail};

use crate::model::{Match, MatchKind};
use crate::process;

use super::SearchOutcome;

const NO_APPLICATIONS_FOUND: u32 = 0x8A150014;

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    search_with(name, limit, |field| {
        let output = process::run(
            "winget",
            &[
                "search",
                field,
                name,
                "--exact",
                "--source",
                "winget",
                "--count",
                "1",
                "--disable-interactivity",
            ],
            20,
        )?;
        has_matches(output.status.code(), &output.stdout)
    })
}

fn search_with(
    name: &str,
    limit: usize,
    mut lookup: impl FnMut(&str) -> Result<bool>,
) -> Result<SearchOutcome> {
    let mut matches = Vec::new();
    for (field, kind) in [
        ("--id", MatchKind::Package),
        ("--name", MatchKind::Package),
        ("--moniker", MatchKind::ToolAlias),
        ("--command", MatchKind::Command),
    ] {
        if lookup(field)? {
            matches.push(Match {
                kind,
                exact: true,
                name: name.to_string(),
                detail: Some(format!("WinGet {} match", field.trim_start_matches('-'))),
                url: None,
            });
        }
    }
    Ok(SearchOutcome::from_matches(matches, limit))
}

fn has_matches(code: Option<i32>, stdout: &[u8]) -> Result<bool> {
    match code.map(|code| code as u32) {
        Some(0) => Ok(true),
        Some(NO_APPLICATIONS_FOUND) => Ok(false),
        _ => bail!(
            "winget search failed: {}",
            String::from_utf8_lossy(stdout).trim()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_matches_are_distinct_from_package_names() {
        let result = search_with("demo", 0, |field| Ok(field == "--command")).unwrap();
        assert_eq!(result.matched_kinds, [MatchKind::Command]);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn failures_are_not_absent_names() {
        assert!(!has_matches(Some(NO_APPLICATIONS_FOUND as i32), b"").unwrap());
        assert!(has_matches(Some(1), b"source unavailable").is_err());
        assert!(has_matches(None, b"").is_err());
    }
}
