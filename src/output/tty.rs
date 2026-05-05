use anyhow::Result;
use std::io::Write;

use crate::model::{Provider, ProviderResult, Report, Status};

use super::print_matches;

pub struct TtyOutput;

impl TtyOutput {
    pub fn start(&self, providers: &[Provider]) -> Result<()> {
        let colors = Colors;
        for provider in providers {
            println!("{:<14} {}", provider.name(), colors.blue("searching..."));
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
        print_result_groups(&report.results);
        std::io::stdout().flush()?;
        Ok(())
    }
}

fn print_result_groups(results: &[ProviderResult]) {
    for result in results {
        print_provider_status(result);
        print_matches(&result.matches);
    }
}

fn print_provider_status(result: &ProviderResult) {
    let colors = Colors;
    match &result.status {
        Status::Available => println!(
            "{:<14} {}",
            result.provider.name(),
            colors.green("available")
        ),
        Status::Taken(detail) => println!(
            "{:<14} {} - {}",
            result.provider.name(),
            colors.red("taken"),
            detail
        ),
        Status::Unknown(error) => {
            println!(
                "{:<14} {} - {}",
                result.provider.name(),
                colors.yellow("unknown"),
                error
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Colors;

impl Colors {
    fn green(self, text: &'static str) -> ColoredText {
        self.paint("32", text)
    }

    fn red(self, text: &'static str) -> ColoredText {
        self.paint("31", text)
    }

    fn yellow(self, text: &'static str) -> ColoredText {
        self.paint("33", text)
    }

    fn blue(self, text: &'static str) -> ColoredText {
        self.paint("34", text)
    }

    fn paint(self, code: &'static str, text: &'static str) -> ColoredText {
        ColoredText { code, text }
    }
}

#[derive(Debug, Clone, Copy)]
struct ColoredText {
    code: &'static str,
    text: &'static str,
}

impl std::fmt::Display for ColoredText {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "\x1b[{}m{}\x1b[0m", self.code, self.text)
    }
}
