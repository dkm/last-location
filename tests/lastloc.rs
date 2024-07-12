use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use last_position::models::NewInfo;

#[test]
fn sanity_check() {
    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn init_db() {
    let db_url = "test_init_db.sqlite";
    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("init");
    cmd.assert().success();
    fs::remove_file(&db_url).unwrap();
}

#[test]
fn list_logs() {
    let db_url = "list_logs.sqlite";
    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("list-logs");
    cmd.assert()
        .stdout(
            predicates::str::is_match("id: 1, priv_token: [a-zA-Z0-9]*, unique_url: .*").unwrap(),
        )
        .success();
    fs::remove_file(&db_url).unwrap();
}

#[test]
fn add_to_logs() {
    let db_url = "add_to_logs.sqlite";
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/add_to_logs.json");

    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"));
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:1727038358, srv ts:1727038358, lat:45.3059779, lon:5.8875309, alt:500, speed:10, dir:120, acc:33, prov:gps, bat:99 \)
- \(dev ts:1727038236, srv ts:1727038236, lat:45.3075006, lon:5.8873286, alt:500, speed:10, dir:120, acc:33, prov:gps, bat:99 \)").unwrap()
        )
        .success();

    fs::remove_file(&db_url).unwrap();
}

#[test]
fn add_to_logs_secure() {
    let db_url = "add_to_logs_secure.sqlite";
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/add_to_logs_secure.json");

    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"))
        .arg("--secure");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1")
        .arg("--secure");
    cmd.assert()
        .stdout(
            predicates::str::is_match(
                r"- \[crypt\] \(srv ts:1727038236, data: 0xf96d3e7cae33ada2195d.*\)",
            )
            .unwrap(),
        )
        .success();

    fs::remove_file(&db_url).unwrap();
}

#[test]
fn expire() {
    let db_url = "expire.sqlite";
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/expire.json");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(db_url).arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"));
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("expire-logs")
        .arg("--max-count")
        .arg("10");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(10)
        );
    fs::remove_file(&db_url).unwrap();
}
