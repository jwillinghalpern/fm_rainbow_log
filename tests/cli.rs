use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;
// // I'm planning use fs to read in input files for comparison
// use std::fs;

type TestResult = Result<(), Box<dyn Error>>;

const PRG: &str = "fmrl";

// --------------------------------------------------
// this test is not super useful because it's basically only testing the clap crate, which I imagine is already well tested
#[test]
fn usage() -> TestResult {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(PRG)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("Usage"));
    }
    Ok(())
}
