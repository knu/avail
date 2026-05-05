use anyhow::Result;
use clap::Parser;
use std::sync::mpsc;
use std::thread;

mod http;
mod model;
mod output;
mod providers;

use model::{Provider, ProviderResult, Report};
use output::Output;
use providers::search_provider;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")");

#[derive(Parser, Debug)]
#[command(
    name = "avail",
    version = VERSION,
    about = "Check tool name availability and uniqueness"
)]
struct Cli {
    /// Tool name to check
    name: String,

    /// Providers to search
    #[arg(short, long = "provider", value_enum)]
    providers: Vec<Provider>,

    /// Maximum fuzzy search results per provider
    #[arg(short, long, default_value_t = 5)]
    limit: usize,

    /// Print machine-readable JSON
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    let providers = if cli.providers.is_empty() {
        Provider::ALL.to_vec()
    } else {
        std::mem::take(&mut cli.providers)
    };

    let output = Output::stdout(cli.json);
    let results = if output.shows_progress() {
        search_providers_with_progress(&providers, &cli.name, cli.limit, &output)?
    } else {
        search_providers(&providers, &cli.name, cli.limit)
    };

    let report = Report {
        name: cli.name,
        results,
    };
    output.finish(&report)
}

fn search_providers(providers: &[Provider], name: &str, limit: usize) -> Vec<ProviderResult> {
    let handles = providers
        .iter()
        .copied()
        .map(|provider| {
            let name = name.to_string();
            thread::spawn(move || search_provider(provider, &name, limit))
        })
        .collect::<Vec<_>>();

    handles
        .into_iter()
        .map(|handle| handle.join().expect("provider thread panicked"))
        .collect()
}

fn search_providers_with_progress(
    providers: &[Provider],
    name: &str,
    limit: usize,
    output: &Output,
) -> Result<Vec<ProviderResult>> {
    output.start(providers)?;

    let (sender, receiver) = mpsc::channel();
    let handles = providers
        .iter()
        .copied()
        .enumerate()
        .map(|(index, provider)| {
            let sender = sender.clone();
            let name = name.to_string();
            thread::spawn(move || {
                let result = search_provider(provider, &name, limit);
                let _ = sender.send((index, result));
            })
        })
        .collect::<Vec<_>>();
    drop(sender);

    let mut results = Vec::with_capacity(providers.len());
    results.resize_with(providers.len(), || None);
    for (index, result) in receiver {
        output.provider_result(providers.len(), index, &result)?;
        results[index] = Some(result);
    }
    for handle in handles {
        handle.join().expect("provider thread panicked");
    }

    let results = results
        .into_iter()
        .map(|result| result.expect("provider result missing"))
        .collect::<Vec<_>>();

    Ok(results)
}
