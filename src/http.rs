use anyhow::{Context, Result};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use std::time::Duration;

const USER_AGENT: &str = concat!("avail/", env!("CARGO_PKG_VERSION"));
const URL_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

pub fn http_get_json<T>(url: &str) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    http_get(url)?
        .map(|mut response| {
            response
                .body_mut()
                .read_json()
                .with_context(|| format!("failed to parse JSON from {url}"))
        })
        .transpose()
}

pub fn http_get_text(url: &str) -> Result<Option<String>> {
    http_get(url)?
        .map(|mut response| {
            response
                .body_mut()
                .read_to_string()
                .with_context(|| format!("failed to read response body from {url}"))
        })
        .transpose()
}

pub fn url_component(input: &str) -> String {
    utf8_percent_encode(input, URL_COMPONENT_ENCODE_SET).to_string()
}

fn http_get(url: &str) -> Result<Option<ureq::http::Response<ureq::Body>>> {
    match ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .config()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .call()
    {
        Ok(response) => Ok(Some(response)),
        Err(ureq::Error::StatusCode(404)) => Ok(None),
        Err(err) => Err(err).with_context(|| format!("failed to GET {url}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_component_preserves_safe_bytes() {
        assert_eq!(url_component("ttyp-name_1.2~3"), "ttyp-name_1.2~3");
    }

    #[test]
    fn test_url_component_escapes_reserved_bytes() {
        assert_eq!(url_component("@scope/name"), "%40scope%2Fname");
        assert_eq!(url_component("a&b+c"), "a%26b%2Bc");
    }
}
