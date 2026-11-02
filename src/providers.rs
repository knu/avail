use anyhow::Result;

use crate::model::{Match, MatchKind, Provider, ProviderResult, Status};

mod alpine;
mod arch;
mod aur;
mod cargo;
mod debian;
mod freebsd;
mod gem;
mod github;
mod gitlab;
mod homebrew;
mod mise;
mod nixpkgs;
mod npm;
mod nuget;
mod pypi;
mod winget;

const PROVIDER_BACKENDS: &[ProviderBackend] = &[
    ProviderBackend {
        provider: Provider::Arch,
        search: arch::search,
    },
    ProviderBackend {
        provider: Provider::Aur,
        search: aur::search,
    },
    ProviderBackend {
        provider: Provider::Alpine,
        search: alpine::search,
    },
    ProviderBackend {
        provider: Provider::Nuget,
        search: nuget::search,
    },
    ProviderBackend {
        provider: Provider::Nixpkgs,
        search: nixpkgs::search,
    },
    ProviderBackend {
        provider: Provider::Gitlab,
        search: gitlab::search,
    },
    ProviderBackend {
        provider: Provider::Winget,
        search: winget::search,
    },
    ProviderBackend {
        provider: Provider::Mise,
        search: mise::search,
    },
    ProviderBackend {
        provider: Provider::Cargo,
        search: cargo::search,
    },
    ProviderBackend {
        provider: Provider::Npm,
        search: npm::search,
    },
    ProviderBackend {
        provider: Provider::Pypi,
        search: pypi::search,
    },
    ProviderBackend {
        provider: Provider::Gem,
        search: gem::search,
    },
    ProviderBackend {
        provider: Provider::Debian,
        search: debian::search,
    },
    ProviderBackend {
        provider: Provider::FreebsdBase,
        search: freebsd::search_base,
    },
    ProviderBackend {
        provider: Provider::FreebsdPorts,
        search: freebsd::search_ports,
    },
    ProviderBackend {
        provider: Provider::Homebrew,
        search: homebrew::search,
    },
    ProviderBackend {
        provider: Provider::Github,
        search: github::search,
    },
];

#[derive(Debug)]
struct SearchOutcome {
    status: Status,
    matched_kinds: Vec<MatchKind>,
    matches: Vec<Match>,
}

impl SearchOutcome {
    fn from_matches(mut matches: Vec<Match>, limit: usize) -> Self {
        let mut matched_kinds = Vec::new();
        for item in matches.iter().filter(|item| item.exact) {
            if !matched_kinds.contains(&item.kind) {
                matched_kinds.push(item.kind);
            }
        }
        matches.sort_by_key(|item| !item.exact);
        let status = matches
            .iter()
            .find(|item| item.exact)
            .map(|item| Status::Taken(item.name.clone()))
            .unwrap_or(Status::Available);
        matches.truncate(limit);
        Self {
            status,
            matched_kinds,
            matches,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ProviderBackend {
    provider: Provider,
    search: fn(&str, usize) -> Result<SearchOutcome>,
}

impl ProviderBackend {
    fn run(self, name: &str, limit: usize) -> ProviderResult {
        match (self.search)(name, limit) {
            Ok(outcome) => ProviderResult {
                provider: self.provider,
                status: outcome.status,
                matched_kinds: outcome.matched_kinds,
                matches: outcome.matches,
            },
            Err(err) => ProviderResult {
                provider: self.provider,
                status: Status::Unknown(format!("{err:#}")),
                matched_kinds: Vec::new(),
                matches: Vec::new(),
            },
        }
    }
}

pub fn search_provider(provider: Provider, name: &str, limit: usize) -> ProviderResult {
    provider_backend(provider).run(name, limit)
}

fn provider_backend(provider: Provider) -> ProviderBackend {
    PROVIDER_BACKENDS
        .iter()
        .copied()
        .find(|backend| backend.provider == provider)
        .expect("all Provider variants have backends")
}

fn html_to_text(body: &str) -> String {
    let mut text = String::with_capacity(body.len());
    let mut in_tag = false;

    for ch in body.chars() {
        match ch {
            '<' => {
                in_tag = true;
                text.push('\n');
            }
            '>' => {
                in_tag = false;
                text.push('\n');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    html_decode(&text)
}

fn html_decode(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

fn names_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: MatchKind, exact: bool, name: &str) -> Match {
        Match {
            kind,
            exact,
            name: name.to_string(),
            detail: None,
            url: None,
        }
    }

    #[test]
    fn exact_evidence_survives_display_limits() {
        for limit in [0, 1] {
            let result = SearchOutcome::from_matches(
                vec![
                    item(MatchKind::Package, false, "demo-extra"),
                    item(MatchKind::Command, true, "tools"),
                    item(MatchKind::Package, true, "demo"),
                    item(MatchKind::Command, true, "other-tools"),
                ],
                limit,
            );
            assert!(matches!(result.status, Status::Taken(_)));
            assert_eq!(
                result.matched_kinds,
                [MatchKind::Command, MatchKind::Package]
            );
            assert_eq!(result.matches.len(), limit);
            assert!(result.matches.iter().all(|item| item.exact));
        }
    }

    #[test]
    fn fuzzy_results_do_not_claim_a_name() {
        let result = SearchOutcome::from_matches(
            vec![item(MatchKind::Repository, false, "example/demo-extra")],
            5,
        );
        assert!(matches!(result.status, Status::Available));
        assert!(result.matched_kinds.is_empty());
        assert_eq!(result.matches.len(), 1);
    }
}
