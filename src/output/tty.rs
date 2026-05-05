use anyhow::Result;
use std::io::Write;

use crate::model::{Match, Provider, ProviderResult, Report, Status};

pub struct TtyOutput;

const PROVIDER_WIDTH: usize = 14;
const STATUS_WIDTH: usize = 12;

impl TtyOutput {
    pub fn start(&self, providers: &[Provider]) -> Result<()> {
        let colors = Colors;
        for provider in providers {
            println!(
                "{} {}",
                pad_provider(provider.name()),
                colors.blue(&pad_status("searching"))
            );
        }
        std::io::stdout().flush()?;
        Ok(())
    }

    pub fn provider_result(
        &self,
        total: usize,
        index: usize,
        result: &ProviderResult,
    ) -> Result<()> {
        let lines_up = total - index;
        print!("\x1b[{lines_up}A\x1b[2K\r");
        print_provider_status(result);
        if lines_up > 1 {
            print!("\x1b[{}B", lines_up - 1);
        }
        std::io::stdout().flush()?;
        Ok(())
    }

    pub fn finish(&self, report: &Report) -> Result<()> {
        println!();
        print!("\x1b[{}A", report.results.len() + 1);
        print!("\x1b[J");
        print_result_groups(&report.results, &report.name);
        std::io::stdout().flush()?;
        Ok(())
    }
}

fn print_result_groups(results: &[ProviderResult], query: &str) {
    for result in results {
        print_provider_status(result);
        print_matches(result, query);
    }
}

fn print_provider_status(result: &ProviderResult) {
    let colors = Colors;
    match &result.status {
        Status::Available => println!(
            "{} {}",
            colors.cyan(&pad_provider(result.provider.name())),
            colors.green(&pad_status("available"))
        ),
        Status::Taken(_) => println!(
            "{} {}",
            colors.cyan(&pad_provider(result.provider.name())),
            colors.red(&pad_status("taken"))
        ),
        Status::Unknown(error) => {
            println!(
                "{} {} {}",
                colors.cyan(&pad_provider(result.provider.name())),
                colors.yellow(&pad_status("unknown")),
                colors.dim(error)
            )
        }
    }
}

fn print_matches(result: &ProviderResult, query: &str) {
    let colors = Colors;
    let highlight = MatchHighlighter::new(result, query);

    for item in &result.matches {
        let highlighted = highlight.matches(item);
        let name = highlighted_text(colors, &item.name, highlighted);
        let detail = item
            .detail
            .as_deref()
            .map(super::first_line)
            .filter(|text| !text.is_empty());
        let prefix = colors.dim("  -");
        match (&detail, &item.url) {
            (Some(detail), Some(url)) => {
                println!(
                    "{prefix} {}{} {} {}{}{}",
                    name,
                    colors.dim(":"),
                    colors.dim(detail),
                    colors.dim("("),
                    highlighted_text(colors, url, highlighted),
                    colors.dim(")")
                );
            }
            (Some(detail), None) => {
                println!(
                    "{prefix} {}{} {}",
                    name,
                    colors.dim(":"),
                    colors.dim(detail)
                );
            }
            (None, Some(url)) => {
                println!(
                    "{prefix} {} {}{}{}",
                    name,
                    colors.dim("("),
                    highlighted_text(colors, url, highlighted),
                    colors.dim(")")
                );
            }
            (None, None) => println!("{prefix} {}", name),
        }
    }
}

fn highlighted_text<'a>(colors: Colors, text: &'a str, highlighted: bool) -> ColoredText<'a> {
    let text = colors.bold(text);
    if highlighted { text.underlined() } else { text }
}

#[derive(Debug, Clone, Copy)]
enum MatchHighlighter<'a> {
    None,
    Exact { query: &'a str },
    Detail { detail: &'a str },
    First { first: &'a Match },
}

impl<'a> MatchHighlighter<'a> {
    fn new(result: &'a ProviderResult, query: &'a str) -> Self {
        let Status::Taken(detail) = &result.status else {
            return Self::None;
        };
        if result
            .matches
            .iter()
            .any(|item| name_matches_query(&item.name, query))
        {
            return Self::Exact { query };
        }
        if result
            .matches
            .iter()
            .any(|item| status_detail_matches(item, detail))
        {
            return Self::Detail { detail };
        }
        result
            .matches
            .first()
            .map_or(Self::None, |first| Self::First { first })
    }

    fn matches(self, item: &Match) -> bool {
        match self {
            Self::None => false,
            Self::Exact { query } => name_matches_query(&item.name, query),
            Self::Detail { detail } => status_detail_matches(item, detail),
            Self::First { first } => std::ptr::eq(first, item),
        }
    }
}

fn name_matches_query(name: &str, query: &str) -> bool {
    name.eq_ignore_ascii_case(query)
        || name
            .rsplit_once('/')
            .is_some_and(|(_, suffix)| suffix.eq_ignore_ascii_case(query))
        || name
            .split(',')
            .map(str::trim)
            .any(|part| part.eq_ignore_ascii_case(query))
}

fn status_detail_matches(item: &Match, detail: &str) -> bool {
    item.name == detail
        || item.name.contains(&format!("({detail})"))
        || item.url.as_deref() == Some(detail)
        || detail
            .strip_prefix("is provided by ")
            .is_some_and(|package| package == item.name)
        || detail
            .rsplit_once(" is provided by ")
            .is_some_and(|(_, package)| package == item.name)
}

fn pad_provider(text: &str) -> String {
    format!("{text:<PROVIDER_WIDTH$}")
}

fn pad_status(text: &str) -> String {
    format!("{text:<STATUS_WIDTH$}")
}

#[derive(Debug, Clone, Copy)]
struct Colors;

impl Colors {
    fn green<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("32", text)
    }

    fn red<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("31", text)
    }

    fn yellow<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("33", text)
    }

    fn blue<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("34", text)
    }

    fn cyan<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("96", text)
    }

    fn dim<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("2", text)
    }

    fn bold<'a>(self, text: &'a str) -> ColoredText<'a> {
        self.paint("1", text)
    }

    fn paint<'a>(self, code: &'static str, text: &'a str) -> ColoredText<'a> {
        ColoredText { code, text }
    }
}

#[derive(Debug, Clone, Copy)]
struct ColoredText<'a> {
    code: &'static str,
    text: &'a str,
}

impl<'a> ColoredText<'a> {
    fn underlined(self) -> Self {
        Self {
            code: match self.code {
                "1" => "1;4",
                "2" => "2;4",
                "31" => "31;4",
                "32" => "32;4",
                "33" => "33;4",
                "34" => "34;4",
                "96" => "96;4",
                _ => self.code,
            },
            text: self.text,
        }
    }
}

impl std::fmt::Display for ColoredText<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "\x1b[{}m{}\x1b[0m", self.code, self.text)
    }
}
