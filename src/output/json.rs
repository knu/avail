use anyhow::Result;

use crate::model::Report;

pub struct JsonOutput;

impl JsonOutput {
    pub fn finish(&self, report: &Report) -> Result<()> {
        println!("{}", serde_json::to_string_pretty(report)?);
        Ok(())
    }
}
