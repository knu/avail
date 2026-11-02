#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn mise_aliases_have_explicit_evidence_in_json_and_text() {
    let directory = tempfile::tempdir().unwrap();
    let mise = directory.path().join("mise");
    fs::write(&mise, "#!/bin/sh\nprintf '%s\\n' '[{\"short\":\"demo\",\"backends\":[\"cargo:demo\"],\"aliases\":[\"dm\"]}]'\n").unwrap();
    fs::set_permissions(&mise, fs::Permissions::from_mode(0o755)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_avail"))
        .args(["-p", "mise", "--json", "dm"])
        .env("PATH", directory.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let result = &report["results"][0];
    assert_eq!(result["status"], "taken");
    assert_eq!(result["matched_kinds"], serde_json::json!(["tool-alias"]));
    assert_eq!(result["matches"][0]["name"], "dm");
    assert_eq!(result["matches"][0]["kind"], "tool-alias");
    assert_eq!(result["matches"][0]["exact"], true);

    let output = Command::new(env!("CARGO_BIN_EXE_avail"))
        .args(["-p", "mise", "--limit", "0", "dm"])
        .env("PATH", directory.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("taken (tool-alias)"));
    assert_eq!(text.lines().count(), 1);
}
