use std::process::Command;
use std::str;

fn main() {
    let revision = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("failed to run git");

    if revision.status.success() {
        let hash = str::from_utf8(&revision.stdout).unwrap().trim();
        println!("cargo:rustc-env=GIT_HASH={}", hash);
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }
}
