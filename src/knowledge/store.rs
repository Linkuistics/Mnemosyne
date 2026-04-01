use crate::knowledge::entry::Entry;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct KnowledgeStore {
    root: PathBuf,
    archive_root: PathBuf,
}

impl KnowledgeStore {
    pub fn new(root: PathBuf, archive_root: PathBuf) -> Self {
        Self { root, archive_root }
    }

    /// Load all knowledge entries from the knowledge directory tree.
    pub fn load_all(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        self.walk_dir(&self.root, &mut entries)?;
        Ok(entries)
    }

    fn walk_dir(&self, dir: &Path, entries: &mut Vec<Entry>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for dir_entry in fs::read_dir(dir).context("Failed to read knowledge directory")? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();

            if path.is_dir() {
                self.walk_dir(&path, entries)?;
            } else if path.extension().is_some_and(|ext| ext == "md") {
                match self.load_entry(&path) {
                    Ok(entry) => entries.push(entry),
                    Err(_) => {
                        // Skip files that don't have valid frontmatter
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single entry from a file path.
    pub fn load_entry(&self, path: &Path) -> Result<Entry> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let mut entry = Entry::parse(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        entry.file_path = Some(path.to_path_buf());
        Ok(entry)
    }

    /// Save an entry back to its file. The entry must have a file_path set.
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let path = entry
            .file_path
            .as_ref()
            .context("Entry has no file_path — cannot save")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = entry.serialize();
        fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Archive an entry: move it from knowledge/ to archive/ with a reason note.
    pub fn archive_entry(&self, entry: &Entry, reason: &str) -> Result<()> {
        let source_path = entry
            .file_path
            .as_ref()
            .context("Entry has no file_path — cannot archive")?;

        fs::create_dir_all(&self.archive_root)?;

        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let filename = source_path
            .file_name()
            .context("Invalid file path")?
            .to_string_lossy();
        let archive_name = format!("{}-{}", timestamp, filename);
        let archive_path = self.archive_root.join(&archive_name);

        let mut archived = entry.clone();
        archived.body = format!(
            "{}\n\n## Archived\n\n**{}:** {}\n",
            entry.body.trim(),
            chrono::Local::now().format("%Y-%m-%d"),
            reason
        );
        archived.file_path = Some(archive_path);
        self.save_entry(&archived)?;

        fs::remove_file(source_path)
            .with_context(|| format!("Failed to remove {}", source_path.display()))?;

        Ok(())
    }

    /// Get the root path of the knowledge store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create a new entry file at the appropriate axis path.
    pub fn create_entry(&self, axis: &str, filename: &str, entry: &mut Entry) -> Result<()> {
        let dir = self.root.join(axis);
        fs::create_dir_all(&dir)?;
        let path = dir.join(filename);
        entry.file_path = Some(path);
        self.save_entry(entry)
    }
}
