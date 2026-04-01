use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use mnemosyne::knowledge::entry::Entry;

#[derive(Debug, Deserialize)]
pub struct QuerySet {
    pub queries: Vec<QuerySpec>,
}

#[derive(Debug, Deserialize)]
pub struct QuerySpec {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub context: Option<String>,
    #[serde(default)]
    pub relevant: Vec<RelevanceJudgement>,
}

#[derive(Debug, Deserialize)]
pub struct RelevanceJudgement {
    pub entry: String,
    pub relevance: u8,
}

#[derive(Debug, Deserialize)]
pub struct ContradictionSet {
    pub pairs: Vec<ContradictionPair>,
}

#[derive(Debug, Deserialize)]
pub struct ContradictionPair {
    pub entry_a: String,
    pub entry_b: String,
    pub is_contradiction: bool,
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedContext {
    pub languages: Vec<String>,
    pub dependencies: Vec<String>,
    pub expected_tags: Vec<String>,
}

pub struct ProjectFixture {
    pub path: PathBuf,
    pub expected: ExpectedContext,
}

pub struct Corpus {
    pub entries: Vec<Entry>,
    pub entry_map: HashMap<String, usize>,
    pub queries: QuerySet,
    pub contradictions: ContradictionSet,
    pub projects: Vec<ProjectFixture>,
}

impl Corpus {
    pub fn load(corpus_dir: &Path) -> Result<Self> {
        let entries_dir = corpus_dir.join("entries");
        let entries = Self::load_entries(&entries_dir)?;
        let entry_map = entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                e.file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|name| (name.to_string_lossy().to_string(), i))
            })
            .collect();

        let queries_path = corpus_dir.join("queries.yaml");
        let queries: QuerySet = serde_yaml::from_str(
            &fs::read_to_string(&queries_path)
                .with_context(|| format!("reading {}", queries_path.display()))?,
        )
        .context("parsing queries.yaml")?;

        let contradictions_path = corpus_dir.join("contradictions.yaml");
        let contradictions: ContradictionSet = serde_yaml::from_str(
            &fs::read_to_string(&contradictions_path)
                .with_context(|| format!("reading {}", contradictions_path.display()))?,
        )
        .context("parsing contradictions.yaml")?;

        let projects = Self::load_projects(&corpus_dir.join("projects"))?;

        Ok(Corpus {
            entries,
            entry_map,
            queries,
            contradictions,
            projects,
        })
    }

    fn load_entries(entries_dir: &Path) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        let mut paths: Vec<_> = fs::read_dir(entries_dir)
            .with_context(|| format!("reading {}", entries_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();
        paths.sort();

        for path in paths {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let mut entry = Entry::parse(&content)
                .with_context(|| format!("parsing {}", path.display()))?;
            entry.file_path = Some(path);
            entries.push(entry);
        }
        Ok(entries)
    }

    fn load_projects(projects_dir: &Path) -> Result<Vec<ProjectFixture>> {
        let mut projects = Vec::new();
        if !projects_dir.exists() {
            return Ok(projects);
        }
        let mut dirs: Vec<_> = fs::read_dir(projects_dir)
            .with_context(|| format!("reading {}", projects_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        dirs.sort();

        for dir in dirs {
            let expected_path = dir.join("expected.yaml");
            if expected_path.exists() {
                let content = fs::read_to_string(&expected_path)
                    .with_context(|| format!("reading {}", expected_path.display()))?;
                let expected: ExpectedContext =
                    serde_yaml::from_str(&content).context("parsing expected.yaml")?;
                projects.push(ProjectFixture {
                    path: dir,
                    expected,
                });
            }
        }
        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_corpus_from_test_directory() {
        let corpus_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");
        if !corpus_dir.exists() {
            return;
        }
        let corpus = Corpus::load(&corpus_dir).expect("corpus should load");
        assert!(corpus.entries.len() >= 30, "expected at least 30 entries");
        assert!(
            corpus.queries.queries.len() >= 10,
            "expected at least 10 queries"
        );
        assert!(
            corpus.contradictions.pairs.len() >= 5,
            "expected at least 5 contradiction pairs"
        );
        assert!(
            corpus.projects.len() >= 3,
            "expected at least 3 mock projects"
        );
    }

    #[test]
    fn entry_map_indexes_by_filename() {
        let corpus_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");
        if !corpus_dir.exists() {
            return;
        }
        let corpus = Corpus::load(&corpus_dir).expect("corpus should load");
        assert!(
            corpus.entry_map.contains_key("languages-rust-lifetimes.md"),
            "entry_map should contain languages-rust-lifetimes.md"
        );
    }
}
