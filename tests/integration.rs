use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::predicate;
use regex::Regex;
use std::{fs, io::Write, process::Command};
use tempfile::{NamedTempFile, TempDir};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn short_help() {
    init();

    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-h").arg("-dddd");
    cmd.assert().success().stdout(predicate::str::contains(
        "Find SEARCH_TERM in available nix packages and sort results by relevance",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("experimental"));
}

#[test]
fn long_help() {
    init();

    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("--help").arg("-dddd");
    cmd.assert().success().stdout(predicate::str::contains(
        "Use up to four times for increased verbosity",
    ));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Flip the order of matches?"));
}

#[test]
fn no_search_term() {
    init();

    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-dddd");
    cmd.assert().failure().stderr(predicate::str::contains(
        "error: the following required arguments were not provided:
  <SEARCH_TERM>",
    ));
}

#[test]
fn too_much_debug() {
    init();

    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-ddddd").arg("search_term").env_clear();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Max log level is 4"));
}

#[test]
fn experimental_output_case_sensitive() {
    init();

    let desired_output = "\
        MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
        MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        \n\
        MyTestPackageName3   1.2.1  More test package description\n\
        MyTestPackageName2   1.0.1\n\
        MyTestPackageName1   1.1.0  Another test package description\n\
        \n\
        MyTestPackageName    1.0.0  Test package description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output() {
    init();

    let desired_output = "\
        MatchMyDescription2  9.8.7  mytestpackageName appears in my description with different capitalization\n\
        MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
        MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        \n\
        mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        MyTestPackageName3   1.2.1  More test package description\n\
        MyTestPackageName2   1.0.1\n\
        MyTestPackageName1   1.1.0  Another test package description\n\
        \n\
        MyTestPackageName    1.0.0  Test package description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_command_line_no_equals() {
    init();

    let desired_output = "\
        MyTestPackageName    1.0.0  Test package description\n\
        \n\
        MyTestPackageName1   1.1.0  Another test package description\n\
        MyTestPackageName2   1.0.1\n\
        MyTestPackageName3   1.2.1  More test package description\n\
        \n\
        mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false")
        .arg("-f")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_command_line_equals() {
    init();

    let desired_output = "\
        MyTestPackageName    1.0.0  Test package description\n\
        \n\
        MyTestPackageName1   1.1.0  Another test package description\n\
        MyTestPackageName2   1.0.1\n\
        MyTestPackageName3   1.2.1  More test package description\n\
        \n\
        mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false")
        .arg("-f=true")
        .arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn experimental_output_flip_by_env_var() {
    init();

    let desired_output = "\
        MyTestPackageName    1.0.0  Test package description\n\
        \n\
        MyTestPackageName1   1.1.0  Another test package description\n\
        MyTestPackageName2   1.0.1\n\
        MyTestPackageName3   1.2.1  More test package description\n\
        \n\
        mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false");
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear()
        .env("NIX_PACKAGE_SEARCH_FLIP", "true");

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn output_case_sensitive() {
    init();

    // The cache mixes scenarios for nixos-the-OS and nix-the-package-manager. We test for both at
    // the same time.
    let desired_output = "\
        nixos.MatchMyDescription1    9.8.7  Also here MyTestPackageName appears in my description\n\
        nixos.MatchMyDescription     a.b.c  MyTestPackageName appears in my description\n\
        nixos.mytestpackageName3     3.2.1  More test package description, now with MyTestPackageName\n\
        nixpkgs.MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
        nixpkgs.MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        nixpkgs.mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        \n\
        nixos.MyTestPackageName3     1.2.1  More test package description\n\
        nixos.MyTestPackageName2     1.0.1\n\
        nixos.MyTestPackageName1     1.1.0  Another test package description\n\
        nixpkgs.MyTestPackageName3   1.2.1  More test package description\n\
        nixpkgs.MyTestPackageName2   1.0.1\n\
        nixpkgs.MyTestPackageName1   1.1.0  Another test package description\n\
        \n\
        nixos.MyTestPackageName      1.0.0  Test package description\n\
        nixpkgs.MyTestPackageName    1.0.0  Test package description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false")
        .arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

#[test]
fn output() {
    init();

    // The cache mixes scenarios for nixos-the-OS and nix-the-package-manager. We test for both at
    // the same time.
    let desired_output = "\
        nixos.MatchMyDescription2    9.8.7  mytestpackageName appears in my description with different capitalization\n\
        nixos.MatchMyDescription1    9.8.7  Also here MyTestPackageName appears in my description\n\
        nixos.MatchMyDescription     a.b.c  MyTestPackageName appears in my description\n\
        nixpkgs.MatchMyDescription2  9.8.7  mytestpackageName appears in my description with different capitalization\n\
        nixpkgs.MatchMyDescription1  9.8.7  Also here MyTestPackageName appears in my description\n\
        nixpkgs.MatchMyDescription   a.b.c  MyTestPackageName appears in my description\n\
        \n\
        nixos.mytestpackageName3     3.2.1  More test package description, now with MyTestPackageName\n\
        nixos.MyTestPackageName3     1.2.1  More test package description\n\
        nixos.MyTestPackageName2     1.0.1\n\
        nixos.MyTestPackageName1     1.1.0  Another test package description\n\
        nixpkgs.mytestpackageName3   3.2.1  More test package description, now with MyTestPackageName\n\
        nixpkgs.MyTestPackageName3   1.2.1  More test package description\n\
        nixpkgs.MyTestPackageName2   1.0.1\n\
        nixpkgs.MyTestPackageName1   1.1.0  Another test package description\n\
        \n\
        nixos.MyTestPackageName      1.0.0  Test package description\n\
        nixpkgs.MyTestPackageName    1.0.0  Test package description\n\
    ";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("--cache-folder=tests/")
        .arg("--experimental=false")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}

// The following tests are not run by default. Use
//
// cargo test -- --ignored
//
// to execute them. N.B.: By exception, we test for multiple things
// at the same time, since these tests are expensive to run.
#[test]
#[ignore]
/// Testing the creation of new caches. This requires internet connection, so
/// it is disabled by default. We also test for correct user messages.
fn cache_creation() {
    init();

    // Create a temporary directory for a cache
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_owned();

    // Create a temporary directory for a nix.conf file
    let tempdir = tempfile::TempDir::new().unwrap();
    let nix_conf_dir = &tempdir.path().join("nix");
    fs::create_dir_all(nix_conf_dir).unwrap();

    let tempfile = NamedTempFile::new_in(&tempdir).unwrap();
    // Enable experimental features: "nix-command" and "flakes"
    write!(&tempfile, "experimental-features = nix-command flakes").unwrap();
    tempfile.persist(nix_conf_dir.join("nix.conf")).unwrap();

    temp_env::with_var("XDG_CONFIG_HOME", Some(&tempdir.path()), || {
        let mut cmd = Command::new(cargo::cargo_bin!("nps"));
        cmd.arg(format!("--cache-folder={}", &temp_path.display()))
            .arg("--experimental=false")
            .arg("-dddd")
            .arg("-r");

        let output = cmd.assert().success();

        let cache_content = fs::read_to_string(temp_path.join("nps.cache")).unwrap();
        let re_vim = Regex::new("vim .*popular clone of the VI editor").unwrap();
        assert!(re_vim.is_match(&cache_content));

        let re_done = Regex::new("Done. Cached info of").unwrap();
        assert!(re_done.is_match(String::from_utf8_lossy(&output.get_output().stderr).as_ref()));
        assert!(re_done.is_match(String::from_utf8_lossy(&output.get_output().stdout).as_ref()));

        let re_flakes = Regex::new("Your system seems to be based on flakes").unwrap();
        assert!(re_flakes.is_match(String::from_utf8_lossy(&output.get_output().stderr).as_ref()));
        assert!(re_flakes.is_match(String::from_utf8_lossy(&output.get_output().stdout).as_ref()));
    });
}

#[test]
#[ignore]
/// Testing the creation of new caches. This requires internet connection, so
/// it is disabled by default. We also test for correct user messages and the
/// -q/--quiet flag.
fn experimental_cache_creation() {
    init();

    // Create a temporary directory for a nix.conf file
    let tempdir = tempfile::TempDir::new().unwrap();
    let nix_conf_dir = &tempdir.path().join("nix");
    fs::create_dir_all(nix_conf_dir).unwrap();

    let tempfile = NamedTempFile::new_in(&tempdir).unwrap();
    // Disable all experimental features
    write!(&tempfile, "experimental-features = ").unwrap();
    tempfile.persist(nix_conf_dir.join("nix.conf")).unwrap();

    temp_env::with_var("XDG_CONFIG_HOME", Some(&tempdir.path()), || {
        // Create a temporary directory for a cache
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_owned();

        let mut cmd = Command::new(cargo::cargo_bin!("nps"));
        cmd.arg(format!("--cache-folder={}", &temp_path.display()))
            .arg("--experimental=true")
            .arg("--quiet")
            .arg("-dddd")
            .arg("-r");

        let output = cmd.assert().success();

        let cache_content = fs::read_to_string(temp_path.join("nps.experimental.cache")).unwrap();
        let re = Regex::new("vim .*popular clone of the VI editor").unwrap();
        assert!(re.is_match(&cache_content));

        let re_done = Regex::new("Done. Cached info of").unwrap();
        assert!(re_done.is_match(String::from_utf8_lossy(&output.get_output().stderr).as_ref()));
        // Check that we suppressed messages (--quiet works)
        assert!(!re_done.is_match(String::from_utf8_lossy(&output.get_output().stdout).as_ref()));

        let re_channels = Regex::new("Your system seems to be based on channels").unwrap();
        assert!(re_channels.is_match(String::from_utf8_lossy(&output.get_output().stderr).as_ref()));
        // Check that we suppressed messages (--quiet works)
        assert!(
            !re_channels.is_match(String::from_utf8_lossy(&output.get_output().stdout).as_ref())
        );
    });
}

#[test]
fn multi_line() {
    init();

    let desired_output = "\
MatchMyDescription1  9.8.7
    Also here MyTestPackageName appears in my description

MatchMyDescription  a.b.c
    MyTestPackageName appears in my description

mytestpackageName3  3.2.1
    More test package description, now with MyTestPackageName

MyTestPackageName3  1.2.1
    More test package description

MyTestPackageName2  1.0.1

MyTestPackageName1  1.1.0
    Another test package description

MyTestPackageName  1.0.0
    Test package description
";
    let mut cmd = Command::new(cargo::cargo_bin!("nps"));
    cmd.arg("-i=false")
        .arg("--cache-folder=tests/")
        .arg("--multi-line=true")
        .arg("--experimental=true")
        .arg("MyTestPackageName")
        .arg("-dddd")
        .env_clear(); // remove env vars

    cmd.assert()
        .success()
        .stdout(predicate::str::diff(desired_output));
}
