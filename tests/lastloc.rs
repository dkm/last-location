use assert_cmd::prelude::*;
use std::process::Command;
use std::fs;
use std::path::Path;

#[test]
fn sanity_check() {
    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn init_db() {
    let db_url = "test_init_db.sqlite";
    if Path::new(&db_url).try_exists().unwrap(){
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("init");
    cmd.assert().success();
}

#[test]
fn list_users() {
    let db_url = "list_users.sqlite";
    if Path::new(&db_url).try_exists().unwrap(){
        fs::remove_file(&db_url).unwrap();
    }

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("init");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("create-user")
        .arg("--name")
        .arg("boby");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lastloc").unwrap();
    cmd.arg("--sqlite-db")
        .arg(db_url)
        .arg("list-users");
    cmd.assert()
        .stdout("id: 1, name: boby, priv_token: None, unique_url: None\n")
       .success();

}
