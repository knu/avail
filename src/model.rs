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
    pub matches: Vec<Match>,
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
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
