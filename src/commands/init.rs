use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Initialize a new Mnemosyne knowledge store.
pub fn run_init(mnemosyne_dir: &Path, from_repo: Option<&str>) -> Result<()> {
    if mnemosyne_dir.exists() {
        bail!(
            "Directory already exists: {}. Use a different path or remove it first.",
            mnemosyne_dir.display()
        );
    }

    if let Some(repo_url) = from_repo {
        clone_from_repo(mnemosyne_dir, repo_url)?;
    } else {
        create_fresh(mnemosyne_dir)?;
    }

    Ok(())
}

fn create_fresh(dir: &Path) -> Result<()> {
    let dirs = [
        "knowledge/languages",
        "knowledge/domains",
        "knowledge/tools",
        "knowledge/techniques",
        "knowledge/projects",
        "archive",
        "cache",
    ];

    for subdir in &dirs {
        fs::create_dir_all(dir.join(subdir))
            .with_context(|| format!("Failed to create {}", subdir))?;
    }

    // Write default config
    let config = Config::default();
    config.save(dir)?;

    // Write .gitignore
    let gitignore = "# Derived data — rebuilt automatically\ncache/\n";
    fs::write(dir.join(".gitignore"), gitignore)?;

    // Initialize git repo (best-effort; failure is non-fatal)
    let git_result = Command::new("git").args(["init"]).current_dir(dir).output();

    if let Ok(output) = git_result {
        if output.status.success() {
            let _ = Command::new("git")
                .args(["add", "."])
                .current_dir(dir)
                .output();
            let _ = Command::new("git")
                .args(["commit", "-m", "Initialize Mnemosyne knowledge store"])
                .current_dir(dir)
                .output();
        }
    }

    Ok(())
}

fn clone_from_repo(dir: &Path, repo_url: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", repo_url, &dir.to_string_lossy()])
        .output()
        .context("Failed to run git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed: {}", stderr);
    }

    Ok(())
}
