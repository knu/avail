use anyhow::{Context, Result, ensure};
use scraper::{Html, Selector};
use url::Url;

use crate::http::{http_get_text, url_component};
use crate::model::{Match, MatchKind};

use super::{SearchOutcome, names_equal};

const BASE: &str = "https://pkgs.alpinelinux.org";

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    ensure!(
        !name.is_empty()
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '+')),
        "unsupported Alpine package or command name"
    );
    let query = url_component(name);
    let mut matches = fetch_pages(
        &format!("/packages?name={query}&branch=edge&arch=x86_64"),
        name,
        false,
    )?;
    matches.extend(fetch_pages(
        &format!("/contents?file={query}&branch=edge&arch=x86_64"),
        name,
        true,
    )?);
    if limit > 0 {
        let body = http_get_text(&format!(
            "{BASE}/packages?name=*{query}*&branch=edge&arch=x86_64"
        ))?
        .context("Alpine package search not found")?;
        for item in parse_page(&body, name, false)?.0 {
            if !matches
                .iter()
                .any(|existing| existing.kind == item.kind && existing.url == item.url)
            {
                matches.push(item);
            }
        }
    }
    Ok(SearchOutcome::from_matches(matches, limit))
}

fn fetch_pages(path: &str, query: &str, contents: bool) -> Result<Vec<Match>> {
    let mut matches = Vec::new();
    for page in 1..=10 {
        let body = http_get_text(&format!("{BASE}{path}&page={page}"))?
            .context("Alpine search not found")?;
        let (items, last_page) = parse_page(&body, query, contents)?;
        matches.extend(items);
        if page >= last_page {
            return Ok(matches);
        }
    }
    anyhow::bail!("Alpine search exceeded 10 pages");
}

fn parse_page(body: &str, query: &str, contents: bool) -> Result<(Vec<Match>, usize)> {
    let document = Html::parse_document(body);
    let table = Selector::parse("table tbody").unwrap();
    ensure!(
        document.select(&table).next().is_some(),
        "unrecognized Alpine search response"
    );
    let row = Selector::parse("table tbody tr").unwrap();
    let cell = Selector::parse("td").unwrap();
    let link = Selector::parse("a[href^='/package/']").unwrap();
    let mut matches = Vec::new();
    for row in document.select(&row) {
        let Some(package) = row.select(&link).next() else {
            continue;
        };
        let name = package.text().collect::<String>().trim().to_string();
        let href = package
            .value()
            .attr("href")
            .context("missing Alpine package URL")?;
        if contents {
            let path = row
                .select(&cell)
                .next()
                .context("missing Alpine file path")?
                .text()
                .collect::<String>();
            let path = path.trim();
            let Some((directory, command)) = path.rsplit_once('/') else {
                continue;
            };
            if !matches!(directory, "/bin" | "/sbin" | "/usr/bin" | "/usr/sbin") || command != query
            {
                continue;
            }
            matches.push(Match {
                kind: MatchKind::Command,
                exact: true,
                name,
                detail: Some(path.to_string()),
                url: Some(format!("{BASE}{href}")),
            });
        } else {
            matches.push(Match {
                kind: MatchKind::Package,
                exact: names_equal(&name, query),
                name,
                detail: package.value().attr("aria-label").map(str::to_string),
                url: Some(format!("{BASE}{href}")),
            });
        }
    }
    let pagination = Selector::parse("#pagination a[href]").unwrap();
    let base = Url::parse(BASE)?;
    let mut last_page = 1;
    for link in document.select(&pagination) {
        let url = base.join(link.value().attr("href").unwrap())?;
        for (_, value) in url.query_pairs().filter(|(key, _)| key == "page") {
            last_page = last_page.max(value.parse::<usize>()?);
        }
    }
    Ok((matches, last_page))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    #[test]
    fn commands_exclude_private_binary_directories() {
        let body = r#"<table><tbody>
        <tr><td>/bin/demo</td><td><a href="/package/edge/main/x86_64/tools">tools</a></td></tr>
        <tr><td>/usr/lib/private/bin/demo</td><td><a href="/package/edge/main/x86_64/private">private</a></td></tr>
        </tbody></table>"#;
        let (matches, _) = parse_page(body, "demo", true).unwrap();
        let result = SearchOutcome::from_matches(matches, 0);
        assert!(matches!(result.status, Status::Taken(_)));
        assert_eq!(result.matched_kinds, [MatchKind::Command]);
    }

    #[test]
    fn html_errors_are_not_availability() {
        assert!(parse_page("<html>Service unavailable</html>", "demo", false).is_err());
    }
}
