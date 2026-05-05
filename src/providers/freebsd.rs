use anyhow::Result;

use crate::http::{http_get_text, url_component};
use crate::model::{Match, Status};

use super::{SearchOutcome, html_decode, html_to_text, names_equal};

// These names are aliases in FreeBSD's man.cgi configuration:
// https://cgit.freebsd.org/doc/tree/website/content/en/cgi/man.cgi
const FREEBSD_BASE_MANPATH: &str = "freebsd-current";
const FREEBSD_PORTS_MANPATH: &str = "freebsd-ports";
const FREEBSD_BASE_SECTIONS: &[&str] = &["1", "8", "2", "3", "4", "5", "6", "7", "9"];

pub(super) fn search_base(name: &str, limit: usize) -> Result<SearchOutcome> {
    let mut pages = Vec::new();
    for section in FREEBSD_BASE_SECTIONS {
        if let Some(page) = fetch_freebsd_base_man_page(name, section)? {
            pages.push(page);
        }
    }

    let body = http_get_text(&format!(
        "https://man.freebsd.org/cgi/man.cgi?query={}&apropos=1&sektion=0&manpath={}&format=html",
        url_component(name),
        FREEBSD_BASE_MANPATH
    ))?
    .unwrap_or_default();
    for page in parse_freebsd_man_pages(&body)
        .into_iter()
        .filter(|page| freebsd_man_page_name_matches_query(&page.name, name))
    {
        if !pages
            .iter()
            .any(|existing| existing.name == page.name && existing.section == page.section)
        {
            pages.push(page);
        }
    }

    let status = pages
        .iter()
        .find(|page| freebsd_man_page_has_name(&page.name, name))
        .map(|page| Status::Taken(page.name.clone()))
        .unwrap_or(Status::Available);
    let matches = pages
        .into_iter()
        .take(limit)
        .map(|page| Match {
            url: Some(format!(
                "https://man.freebsd.org/cgi/man.cgi?query={}&sektion={}&manpath={}",
                url_component(page.primary_name()),
                url_component(&page.section),
                FREEBSD_BASE_MANPATH
            )),
            detail: Some(page.description),
            name: page.name,
        })
        .collect();

    Ok(SearchOutcome { status, matches })
}

fn fetch_freebsd_base_man_page(name: &str, section: &str) -> Result<Option<FreebsdManPage>> {
    let body = http_get_text(&format!(
        "https://man.freebsd.org/cgi/man.cgi?query={}&sektion={}&manpath={}&format=ascii",
        url_component(name),
        url_component(section),
        FREEBSD_BASE_MANPATH
    ))?
    .unwrap_or_default();

    Ok(parse_freebsd_exact_man_page(&body, name, section))
}

pub(super) fn search_ports(name: &str, limit: usize) -> Result<SearchOutcome> {
    let body = http_get_text(&format!(
        "https://ports.freebsd.org/cgi/ports.cgi?query={}&stype=name&sektion=all&manpath={}",
        url_component(name),
        FREEBSD_PORTS_MANPATH
    ))?
    .unwrap_or_default();
    let ports = parse_freebsd_ports(&body);

    let status = ports
        .iter()
        .find(|port| names_equal(&port.package, name) || names_equal(port.port_name(), name))
        .map(|port| Status::Taken(port.origin.clone()))
        .unwrap_or(Status::Available);
    let matches = ports
        .into_iter()
        .take(limit)
        .map(|port| {
            let name = port.display_name();
            Match {
                url: Some(format!("https://www.freshports.org/{}/", port.origin)),
                detail: port.description,
                name,
            }
        })
        .collect();

    Ok(SearchOutcome { status, matches })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FreebsdManPage {
    name: String,
    section: String,
    description: String,
}

impl FreebsdManPage {
    fn primary_name(&self) -> &str {
        self.name
            .split(',')
            .next()
            .map(str::trim)
            .unwrap_or(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FreebsdPort {
    origin: String,
    package: String,
    description: Option<String>,
}

impl FreebsdPort {
    fn port_name(&self) -> &str {
        self.origin
            .rsplit('/')
            .next()
            .unwrap_or(self.origin.as_str())
    }

    fn display_name(&self) -> String {
        if self.package == self.port_name() {
            self.origin.clone()
        } else {
            format!("{} ({})", self.package, self.origin)
        }
    }
}

fn parse_freebsd_man_pages(body: &str) -> Vec<FreebsdManPage> {
    html_to_text(body)
        .lines()
        .filter_map(parse_freebsd_man_line)
        .collect()
}

fn parse_freebsd_exact_man_page(body: &str, query: &str, section: &str) -> Option<FreebsdManPage> {
    let text = html_to_text(body);
    let heading = format!("{}({})", query, section).to_lowercase();
    if !text.to_lowercase().contains(&heading) {
        return None;
    }

    Some(FreebsdManPage {
        name: query.to_string(),
        section: section.to_string(),
        description: parse_freebsd_exact_man_page_description(&text, query)
            .unwrap_or_else(|| format!("FreeBSD base section {section} manual page")),
    })
}

fn parse_freebsd_exact_man_page_description(text: &str, query: &str) -> Option<String> {
    let query = query.to_lowercase();
    text.lines().find_map(|line| {
        let line = line.trim();
        let (name, description) = line.split_once(" - ")?;
        names_equal(name.trim(), &query).then(|| description.trim().to_string())
    })
}

fn parse_freebsd_man_line(line: &str) -> Option<FreebsdManPage> {
    let (name, rest) = line.split_once('(')?;
    let (section, rest) = rest.split_once(')')?;
    let description = rest.strip_prefix(" - ")?;
    let name = name.trim();
    let section = section.trim();
    let description = description.trim();
    if name.is_empty() || section.is_empty() || description.is_empty() {
        return None;
    }

    Some(FreebsdManPage {
        name: name.to_string(),
        section: section.to_string(),
        description: description.to_string(),
    })
}

fn freebsd_man_page_has_name(names: &str, query: &str) -> bool {
    names.split(',').any(|name| names_equal(name.trim(), query))
}

fn freebsd_man_page_name_matches_query(names: &str, query: &str) -> bool {
    let query = query.to_lowercase();
    names
        .split(',')
        .any(|name| name.trim().to_lowercase().contains(&query))
}

fn parse_freebsd_ports(body: &str) -> Vec<FreebsdPort> {
    const CGIT_PREFIX: &str = "https://cgit.FreeBSD.org/ports/tree/";
    let mut ports = Vec::new();
    let mut rest = body;

    while let Some(index) = rest.find(CGIT_PREFIX) {
        rest = &rest[index + CGIT_PREFIX.len()..];
        let Some(origin_end) = rest.find('"') else {
            break;
        };
        let origin = html_decode(&rest[..origin_end]);
        if !origin.contains('/') {
            continue;
        }

        let port_body = rest.get(origin_end..).unwrap_or_default();
        let package = parse_freebsd_port_package(port_body)
            .unwrap_or_else(|| origin.rsplit('/').next().unwrap_or(&origin).to_string());
        let description = parse_freebsd_port_description(port_body);
        if !ports.iter().any(|port: &FreebsdPort| port.origin == origin) {
            ports.push(FreebsdPort {
                origin,
                package,
                description,
            });
        }
    }

    ports
}

fn parse_freebsd_port_package(body: &str) -> Option<String> {
    let body = body.strip_prefix("\">")?;
    let end = body.find("</a>")?;
    let package_with_version = html_decode(&body[..end]);
    package_name_without_version(&package_with_version)
}

fn parse_freebsd_port_description(body: &str) -> Option<String> {
    let dd_start = body.find("<dd>")? + "<dd>".len();
    let dd_body = body.get(dd_start..)?;
    let br_end = dd_body.find("<br")?;
    let description = html_to_text(&dd_body[..br_end]).trim().to_string();
    (!description.is_empty()).then_some(description)
}

fn package_name_without_version(package: &str) -> Option<String> {
    let (name, version) = package.rsplit_once('-')?;
    version
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
        .then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_freebsd_man_line() {
        assert_eq!(
            parse_freebsd_man_line("ls(1) - list directory contents"),
            Some(FreebsdManPage {
                name: "ls".to_string(),
                section: "1".to_string(),
                description: "list directory contents".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_freebsd_exact_man_page() {
        let body = "\
LS(1)                   FreeBSD General Commands Manual

NAME
     ls - list directory contents
";

        assert_eq!(
            parse_freebsd_exact_man_page(body, "ls", "1"),
            Some(FreebsdManPage {
                name: "ls".to_string(),
                section: "1".to_string(),
                description: "list directory contents".to_string(),
            })
        );
        assert_eq!(parse_freebsd_exact_man_page(body, "ifconfig", "8"), None);
    }

    #[test]
    fn test_freebsd_man_page_has_name() {
        assert!(freebsd_man_page_has_name("apropos, whatis", "whatis"));
        assert!(!freebsd_man_page_has_name("apropos, whatis", "man"));
    }

    #[test]
    fn test_freebsd_man_page_name_matches_query() {
        assert!(freebsd_man_page_name_matches_query("ls, dir", "ls"));
        assert!(!freebsd_man_page_name_matches_query(
            "barman-cloud-backup-list",
            "ls"
        ));
    }

    #[test]
    fn test_parse_freebsd_ports() {
        let body = r#"
<dt><b><a name="ttyplot-1.7.0"></a><a href="https://cgit.FreeBSD.org/ports/tree/graphics/ttyplot">ttyplot-1.7.0</a></b></dt>
<dd>Realtime plotting utility for TTY with data input from stdin<br />
<a href="?stype=pkg&amp;query=graphics/ttyplot">Packages</a>
</dd>
<dt><b><a name="uw-ttyp0-2.1"></a><a href="https://cgit.FreeBSD.org/ports/tree/x11-fonts/uw-ttyp0">uw-ttyp0-2.1</a></b></dt>
<dd>Monospaced bitmap fonts for X11<br />
<a href="?stype=pkg&amp;query=x11-fonts/uw-ttyp0">Packages</a>
</dd>
"#;

        assert_eq!(
            parse_freebsd_ports(body),
            [
                FreebsdPort {
                    origin: "graphics/ttyplot".to_string(),
                    package: "ttyplot".to_string(),
                    description: Some(
                        "Realtime plotting utility for TTY with data input from stdin".to_string()
                    ),
                },
                FreebsdPort {
                    origin: "x11-fonts/uw-ttyp0".to_string(),
                    package: "uw-ttyp0".to_string(),
                    description: Some("Monospaced bitmap fonts for X11".to_string()),
                },
            ]
        );
    }

    #[test]
    fn test_package_name_without_version() {
        assert_eq!(
            package_name_without_version("py311-traittypes-0.2.1_1"),
            Some("py311-traittypes".to_string())
        );
        assert_eq!(
            package_name_without_version("gettext-runtime-1.0"),
            Some("gettext-runtime".to_string())
        );
    }
}
