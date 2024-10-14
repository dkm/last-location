use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

struct TestData<'a> {
    db_file_name: &'a str,
}

impl<'a> Drop for TestData<'a> {
    fn drop(&mut self) {
        fs::remove_file(self.db_file_name).unwrap();
    }
}

impl<'a> TestData<'a> {
    fn new(db_file_name: &'a str) -> Self {
        if Path::new(db_file_name).try_exists().unwrap() {
            fs::remove_file(&db_file_name).unwrap();
        }
        TestData { db_file_name }
    }
}

#[test]
fn sanity_check() {
    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn init_db() {
    let test_db = TestData::new("init_db.sqlite");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();
}

#[test]
fn list_logs() {
    let test_db = TestData::new("list_logs.sqlite");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-logs");
    cmd.assert()
        .stdout(
            predicates::str::is_match("id: 1, priv_token: [a-zA-Z0-9]*, unique_url: .*").unwrap(),
        )
        .success();
}

#[test]
fn add_to_logs() {
    let test_db = TestData::new("add_to_logs.sqlite");
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/add_to_logs.json");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"));
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:1727038358, srv ts:1727038358, lat:45.3059779, lon:5.8875309, alt:500, speed:10, dir:120, acc:33, prov:gps, bat:99 \)
- \(dev ts:1727038236, srv ts:1727038236, lat:45.3075006, lon:5.8873286, alt:500, speed:10, dir:120, acc:33, prov:gps, bat:99 \)").unwrap()
        )
        .success();
}

#[test]
fn add_to_logs_secure() {
    let test_db = TestData::new("add_to_logs_secure.sqlite");
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/add_to_logs_secure.json");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"))
        .arg("--secure");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
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
}

#[test]
fn expire_clear_1() {
    let test_db = TestData::new("expire_clear_1.sqlite");
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/expire_clear_1.json");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"));
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("expire-logs")
        .arg("--max-count")
        .arg("10");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(10)
        );

    // Expire with max-count = 5, with 10 locations => check only 5 remaining
    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("expire-logs")
        .arg("--max-count")
        .arg("5");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(5)
        );

    // Expire with max-count = 5 and max-lifetime = 60, with 5 locations => check only 2
    // server timestamps in expire.json are modified for easier calculation :)
    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.env("LASTLOC_MOCK_SERVER_TIME", "150")
        .arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("expire-logs")
        .arg("--max-count")
        .arg("5")
        .arg("--max-lifetime")
        .arg("60");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(2)
        );
}

#[test]
fn expire_clear_2() {
    let test_db = TestData::new("expire_clear_2.sqlite");
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests/expire_clear_2.json");

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db").arg(test_db.db_file_name).arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("create-log");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("add-to-log")
        .arg("--data")
        .arg(d.to_str().expect("error getting resources"));
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("expire-logs")
        .arg("--max-count")
        .arg("2");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("2");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(2)
        );

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(test_db.db_file_name)
        .arg("list-locations")
        .arg("--log-id")
        .arg("1");
    cmd.assert()
        .stdout(
            predicates::str::is_match(r"- \(dev ts:[0-9]*, srv ts:[0-9]*, lat:[0-9]*\.[0-9]*, lon:[0-9]*\.[0-9]*, alt:[0-9]*, speed:[0-9]*, dir:[0-9]*, acc:[0-9]*, prov:gps, bat:[0-9]* \)").unwrap().count(2)
        );
}
