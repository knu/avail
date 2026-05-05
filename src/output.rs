use anyhow::Result;
use std::io::IsTerminal;

use crate::model::{Match, Provider, ProviderResult, Report};

mod json;
mod text;
mod tty;

pub enum Output {
    Json(json::JsonOutput),
    Text(text::TextOutput),
    Tty(tty::TtyOutput),
}

impl Output {
    pub fn stdout(json: bool) -> Self {
        if json {
            Self::Json(json::JsonOutput)
        } else if std::io::stdout().is_terminal() {
            Self::Tty(tty::TtyOutput)
        } else {
            Self::Text(text::TextOutput)
        }
    }

    pub fn shows_progress(&self) -> bool {
        matches!(self, Self::Tty(_))
    }

    pub fn start(&self, providers: &[Provider]) -> Result<()> {
        match self {
            Self::Tty(output) => output.start(providers),
            Self::Json(_) | Self::Text(_) => Ok(()),
        }
    }

    pub fn provider_result(
        &self,
        total: usize,
        index: usize,
        result: &ProviderResult,
    ) -> Result<()> {
        match self {
            Self::Tty(output) => output.provider_result(total, index, result),
            Self::Json(_) | Self::Text(_) => Ok(()),
        }
    }

    pub fn finish(&self, report: &Report) -> Result<()> {
        match self {
            Self::Json(output) => output.finish(report),
            Self::Text(output) => output.finish(report),
            Self::Tty(output) => output.finish(report),
        }
    }
}

fn print_matches(matches: &[Match]) {
    for item in matches {
        let detail = item
            .detail
            .as_deref()
            .map(first_line)
            .filter(|text| !text.is_empty());
        match (&detail, &item.url) {
            (Some(detail), Some(url)) => println!("  - {}: {} ({})", item.name, detail, url),
            (Some(detail), None) => println!("  - {}: {}", item.name, detail),
            (None, Some(url)) => println!("  - {} ({})", item.name, url),
            (None, None) => println!("  - {}", item.name),
        }
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("").trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_line_trims_and_discards_later_lines() {
        assert_eq!(first_line("  first  \nsecond"), "first");
    }
}
