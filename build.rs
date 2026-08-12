use std::env;
use std::path::Path;
use std::process::Command;

fn git_output(repo_dir: &Path, args: &[&str], allow_empty: bool) -> Option<String> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() && !allow_empty {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-env-changed=CI");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_owned());
    let repo_dir = Path::new(&manifest_dir);

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_owned());
    let commit = git_output(repo_dir, &["rev-parse", "--short=12", "HEAD"], false)
        .unwrap_or_else(|| "no-commit".to_owned());
    let exact_tag = git_output(
        repo_dir,
        &["describe", "--tags", "--exact-match", "HEAD"],
        false,
    );
    let clean = git_output(repo_dir, &["status", "--porcelain"], true)
        .map(|status| status.is_empty())
        .unwrap_or(false);

    let version = if let Some(tag) = exact_tag {
        if clean {
            tag
        } else {
            format!("{}+{}.dirty", pkg_version, commit)
        }
    } else {
        let state = if clean { "clean" } else { "dirty" };
        format!("{}+{}.{}", pkg_version, commit, state)
    };

    println!("cargo:rustc-env=PARQUET_MERGE_VERSION={version}");
}
