use anyhow::{Result, ensure};
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::http::url_component;
use crate::model::{Match, MatchKind};
use crate::process;

use super::{SearchOutcome, names_equal};

pub(super) fn search(name: &str, limit: usize) -> Result<SearchOutcome> {
    let pattern = regex_syntax::escape(name);
    let output = process::run("nix", &["search", "--json", "nixpkgs", "--", &pattern], 30)?;
    ensure!(
        output.status.success(),
        "nix search failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let packages: BTreeMap<String, Package> = serde_json::from_slice(&output.stdout)?;
    Ok(packages_result(packages, name, limit))
}

fn packages_result(
    packages: BTreeMap<String, Package>,
    query: &str,
    limit: usize,
) -> SearchOutcome {
    SearchOutcome::from_matches(
        packages
            .into_iter()
            .map(|(attribute, package)| {
                let short_attribute = attribute.splitn(3, '.').nth(2).unwrap_or(&attribute);
                Match {
                    kind: MatchKind::Package,
                    exact: names_equal(&package.pname, query)
                        || names_equal(short_attribute, query),
                    url: Some(format!(
                        "https://search.nixos.org/packages?channel=unstable&show={}",
                        url_component(short_attribute)
                    )),
                    name: attribute,
                    detail: package.description,
                }
            })
            .collect(),
        limit,
    )
}

#[derive(Deserialize)]
struct Package {
    pname: String,
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_attributes_are_not_global_names() {
        let packages = serde_json::from_str(r#"{"legacyPackages.x86_64-linux.python3Packages.demo":{"pname":"python3-demo","description":"Example"}}"#).unwrap();
        let result = packages_result(packages, "demo", 5);
        assert!(!result.matches[0].exact);
    }
}
