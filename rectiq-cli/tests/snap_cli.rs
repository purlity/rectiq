use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn cli_sketch_snapshot() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("empty.json");
    file.write_str("").unwrap();

    let assert = Command::cargo_bin("rectiq")
        .unwrap()
        .env("RECTIQ_SILENT", "1")
        .arg("sketch")
        .arg(file.path())
        .assert()
        .success()
        .stderr("");

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "");
}
