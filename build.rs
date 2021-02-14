use std::process::Command;

// https://stackoverflow.com/a/44407625/8843830
fn main() {
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}