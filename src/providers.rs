use anyhow::Result;

use crate::model::{Match, Provider, ProviderResult, Status};

mod cargo;
mod debian;
mod freebsd;
mod gem;
mod github;
mod homebrew;
mod npm;
mod pypi;

const PROVIDER_BACKENDS: &[ProviderBackend] = &[
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
    matches: Vec<Match>,
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
                matches: outcome.matches,
            },
            Err(err) => ProviderResult {
                provider: self.provider,
                status: Status::Unknown(err.to_string()),
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

fn exact_status(exact_name: Option<&str>, query: &str, detail: Option<String>) -> Status {
    match exact_name {
        Some(name) if names_equal(name, query) => Status::Taken(
            detail
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| name.to_string()),
        ),
        _ => Status::Available,
    }
}

fn names_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
