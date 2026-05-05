use anyhow::Result;

use crate::http::{http_get_text, url_component};
use crate::model::{Match, Status};

use super::{SearchOutcome, html_decode, html_to_text, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let body = http_get_text(&format!(
        "https://packages.debian.org/search?keywords={}&searchon=names&suite=all&section=all",
        url_component(name)
    ))?
    .unwrap_or_default();
    let packages = parse_debian_package_names(&body);
    let contents = search_contents(name)?;

    let status = packages
        .iter()
        .find(|package| names_equal(package, name))
        .map(|package| Status::Taken(package.clone()))
        .or_else(|| {
            contents
                .iter()
                .find(|entry| entry.is_command_match(name))
                .and_then(|entry| entry.packages.first())
                .map(|package| Status::Taken(format!("{name} is provided by {package}")))
        })
        .unwrap_or(Status::Available);
    let mut matches: Vec<Match> = contents
        .iter()
        .filter(|entry| entry.is_command_match(name))
        .flat_map(|entry| {
            entry.packages.iter().map(|package| Match {
                url: Some(format!("https://packages.debian.org/{}", package)),
                detail: Some(entry.path.clone()),
                name: package.clone(),
            })
        })
        .take(limit)
        .collect();
    for package in packages {
        if matches.len() >= limit {
            break;
        }
        if matches.iter().any(|item| item.name == package) {
            continue;
        }
        matches.push(Match {
            url: Some(format!("https://packages.debian.org/{}", package)),
            detail: None,
            name: package,
        });
    }

    Ok(SearchOutcome { status, matches })
}

fn search_contents(name: &str) -> Result<Vec<DebianContentEntry>> {
    let body = http_get_text(&format!(
        "https://packages.debian.org/search?arch=any&keywords={}&mode=exactfilename&searchon=contents&suite=stable",
        url_component(name)
    ))?
    .unwrap_or_default();

    Ok(parse_debian_content_entries(&body))
}

fn parse_debian_package_names(body: &str) -> Vec<String> {
    html_to_text(body)
        .lines()
        .filter_map(parse_debian_package_name_line)
        .collect()
}

fn parse_debian_package_name_line(line: &str) -> Option<String> {
    let line = line.trim().trim_start_matches('#').trim();
    let name = line.strip_prefix("Package ")?.trim();
    is_debian_package_name(name).then(|| name.to_string())
}

fn is_debian_package_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '+' | '-' | '.')
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DebianContentEntry {
    path: String,
    packages: Vec<String>,
}

impl DebianContentEntry {
    fn is_command_match(&self, query: &str) -> bool {
        let Some((dir, name)) = self.path.rsplit_once('/') else {
            return false;
        };

        name == query && matches!(dir, "/bin" | "/sbin" | "/usr/bin" | "/usr/sbin")
    }
}

fn parse_debian_content_entries(body: &str) -> Vec<DebianContentEntry> {
    let mut entries = Vec::new();
    let mut rest = body;

    while let Some(row_start) = rest.find("<tr>") {
        rest = &rest[row_start + "<tr>".len()..];
        let Some(row_end) = rest.find("</tr>") else {
            break;
        };
        let row = &rest[..row_end];
        rest = &rest[row_end + "</tr>".len()..];

        let Some(path) = parse_debian_content_path(row) else {
            continue;
        };
        let packages = parse_debian_content_packages(row);
        if !packages.is_empty() {
            entries.push(DebianContentEntry { path, packages });
        }
    }

    entries
}

fn parse_debian_content_path(row: &str) -> Option<String> {
    let file_class = row.find("class=\"file\"")?;
    let file_cell = &row[file_class..];
    let cell_start = file_cell.find('>')? + 1;
    let file_cell = &file_cell[cell_start..];
    let cell_end = file_cell.find("</td>")?;
    let path = html_to_text(&file_cell[..cell_end])
        .split_whitespace()
        .collect::<String>();
    (!path.is_empty()).then_some(path)
}

fn parse_debian_content_packages(row: &str) -> Vec<String> {
    const PACKAGE_LINK_PREFIX: &str = "<a href=\"/";
    let mut packages = Vec::new();
    let mut rest = row;

    while let Some(link_start) = rest.find(PACKAGE_LINK_PREFIX) {
        rest = &rest[link_start + PACKAGE_LINK_PREFIX.len()..];
        let Some(href_end) = rest.find('"') else {
            break;
        };
        let href = html_decode(&rest[..href_end]);
        rest = &rest[href_end..];

        let Some(package) = href.rsplit('/').next() else {
            continue;
        };
        if is_debian_package_name(package) && !packages.iter().any(|item| item == package) {
            packages.push(package.to_string());
        }
    }

    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_debian_package_names() {
        let body = "\
## Exact hits

### Package ttyp

  * sid (unstable): Terminal typing test

### Package ttyplot
";

        assert_eq!(parse_debian_package_names(body), ["ttyp", "ttyplot"]);
    }

    #[test]
    fn test_parse_debian_package_names_from_html() {
        let body = r#"
<h2>Exact hits</h2>
<h3>Package curl</h3>
<ul><li>trixie (web): command line tool for transferring data with URL syntax</li></ul>
<h2>Other hits</h2>
<h3>Package libcurl4</h3>
"#;

        assert_eq!(parse_debian_package_names(body), ["curl", "libcurl4"]);
    }

    #[test]
    fn test_parse_debian_content_entries() {
        let body = r#"
<tr>
    <td class="file">/usr/bin/<span class="keyword">ls</span></td>
    <td><a href="/trixie/coreutils">coreutils</a> [not mips64el]</td>
</tr>
<tr>
    <td class="file">/usr/sbin/<span class="keyword">service</span></td>
    <td><a href="/trixie/init-system-helpers">init-system-helpers</a></td>
</tr>
<tr>
    <td class="file">/usr/lib/plan9/bin/<span class="keyword">ls</span></td>
    <td><a href="/trixie/9base">9base</a></td>
</tr>
"#;

        assert_eq!(
            parse_debian_content_entries(body),
            [
                DebianContentEntry {
                    path: "/usr/bin/ls".to_string(),
                    packages: vec!["coreutils".to_string()],
                },
                DebianContentEntry {
                    path: "/usr/sbin/service".to_string(),
                    packages: vec!["init-system-helpers".to_string()],
                },
                DebianContentEntry {
                    path: "/usr/lib/plan9/bin/ls".to_string(),
                    packages: vec!["9base".to_string()],
                },
            ]
        );
        assert!(parse_debian_content_entries(body)[0].is_command_match("ls"));
        assert!(parse_debian_content_entries(body)[1].is_command_match("service"));
        assert!(!parse_debian_content_entries(body)[2].is_command_match("ls"));
    }
}
