use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let commit_id = Command::new("git")
        .args(&["rev-parse", "--short=7", "HEAD"])
        .output()
        .expect("Executing git rev-parse failed.")
        .stdout;

    let out_dir = PathBuf::from(env::var_os("OUT_DIR")
        .expect("Env variable OUT_DIR not set."));
    File::create(out_dir.join("commit-id"))
        .expect("Could not open 'commit-id'.")
        .write_all(&commit_id)
        .expect("Could not write to 'commit-id'.");
}
