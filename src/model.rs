use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Cargo,
    Npm,
    Pypi,
    Gem,
    Debian,
    FreebsdBase,
    FreebsdPorts,
    Homebrew,
    Github,
    Arch,
    Aur,
    Alpine,
    Nuget,
    Nixpkgs,
    Gitlab,
    Winget,
    Mise,
}

impl Provider {
    pub const ALL: &'static [Self] = &[
        Self::Cargo,
        Self::Npm,
        Self::Pypi,
        Self::Gem,
        Self::Debian,
        Self::FreebsdBase,
        Self::FreebsdPorts,
        Self::Homebrew,
        Self::Github,
        Self::Arch,
        Self::Aur,
        Self::Alpine,
        Self::Nuget,
        Self::Nixpkgs,
        Self::Gitlab,
        Self::Winget,
        Self::Mise,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Npm => "npm",
            Self::Pypi => "pypi",
            Self::Gem => "gem",
            Self::Debian => "debian",
            Self::FreebsdBase => "freebsd-base",
            Self::FreebsdPorts => "freebsd-ports",
            Self::Homebrew => "homebrew",
            Self::Github => "github",
            Self::Arch => "arch",
            Self::Aur => "aur",
            Self::Alpine => "alpine",
            Self::Nuget => "nuget",
            Self::Nixpkgs => "nixpkgs",
            Self::Gitlab => "gitlab",
            Self::Winget => "winget",
            Self::Mise => "mise",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub name: String,
    pub results: Vec<ProviderResult>,
}

#[derive(Debug, Serialize)]
pub struct ProviderResult {
    pub provider: Provider,
    #[serde(flatten)]
    pub status: Status,
    pub matched_kinds: Vec<MatchKind>,
    pub matches: Vec<Match>,
}

impl ProviderResult {
    pub fn match_summary(&self) -> String {
        self.matched_kinds
            .iter()
            .map(|kind| kind.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "kebab-case")]
pub enum Status {
    Available,
    Taken(String),
    Unknown(String),
}

#[derive(Debug, Serialize)]
pub struct Match {
    pub kind: MatchKind,
    pub exact: bool,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Match {
    pub fn label(&self) -> String {
        format!(
            "[{}{}]",
            self.kind.name(),
            if self.exact { ":exact" } else { "" }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchKind {
    Package,
    Command,
    Repository,
    ManualPage,
    ToolAlias,
}

impl MatchKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Command => "command",
            Self::Repository => "repository",
            Self::ManualPage => "manual-page",
            Self::ToolAlias => "tool-alias",
        }
    }
}
