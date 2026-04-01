use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Install the Claude Code adapter plugin.
pub fn run_install_claude_code(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("Failed to create {}", target.display()))?;

    copy_dir_recursive(source, target)?;

    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source).context("Failed to read source directory")? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);

        if source_path.is_dir() {
            fs::create_dir_all(&target_path)?;
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}
