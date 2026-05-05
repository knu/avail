use anyhow::Result;

use crate::model::{ProviderResult, Report, Status};

use super::print_matches;

pub struct TextOutput;

impl TextOutput {
    pub fn finish(&self, report: &Report) -> Result<()> {
        print_result_groups(&report.results);
        Ok(())
    }
}

pub(super) fn print_result_groups(results: &[ProviderResult]) {
    for result in results {
        print_provider_status(result);
        print_matches(&result.matches);
    }
}

fn print_provider_status(result: &ProviderResult) {
    match &result.status {
        Status::Available => println!("{:<14} available", result.provider.name()),
        Status::Taken(detail) => println!("{:<14} taken - {}", result.provider.name(), detail),
        Status::Unknown(error) => println!("{:<14} unknown - {}", result.provider.name(), error),
    }
}
