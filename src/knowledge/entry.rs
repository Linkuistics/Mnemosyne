use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Tag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
    Prospective,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Origin {
    pub project: String,
    pub date: NaiveDate,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    pub title: String,
    pub tags: Vec<Tag>,
    pub created: NaiveDate,
    pub last_validated: NaiveDate,
    pub confidence: Confidence,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub origins: Vec<Origin>,
    #[serde(default)]
    pub supersedes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub title: String,
    pub tags: Vec<Tag>,
    pub created: NaiveDate,
    pub last_validated: NaiveDate,
    pub confidence: Confidence,
    pub source: Option<String>,
    pub origins: Vec<Origin>,
    pub supersedes: Vec<String>,
    pub body: String,
    pub file_path: Option<PathBuf>,
}

impl Entry {
    /// Parse a knowledge entry from markdown with YAML frontmatter.
    /// Expects the format: ---\n<yaml>\n---\n<markdown body>
    pub fn parse(content: &str) -> Result<Self> {
        let content = content.trim_start();
        if !content.starts_with("---") {
            return Err(anyhow!("Missing YAML frontmatter delimiter"));
        }

        let after_first = &content[3..];
        let end = after_first
            .find("\n---")
            .ok_or_else(|| anyhow!("Missing closing frontmatter delimiter"))?;

        let yaml_str = &after_first[..end];
        let body_start = end + 4; // skip \n---
        let body = after_first[body_start..]
            .trim_start_matches('\n')
            .to_string();

        let fm: Frontmatter =
            serde_yaml::from_str(yaml_str).context("Failed to parse frontmatter YAML")?;

        Ok(Self {
            title: fm.title,
            tags: fm.tags,
            created: fm.created,
            last_validated: fm.last_validated,
            confidence: fm.confidence,
            source: fm.source,
            origins: fm.origins,
            supersedes: fm.supersedes,
            body,
            file_path: None,
        })
    }

    /// Serialize the entry back to markdown with YAML frontmatter.
    pub fn serialize(&self) -> String {
        let fm = Frontmatter {
            title: self.title.clone(),
            tags: self.tags.clone(),
            created: self.created,
            last_validated: self.last_validated,
            confidence: self.confidence.clone(),
            source: self.source.clone(),
            origins: self.origins.clone(),
            supersedes: self.supersedes.clone(),
        };

        let yaml = serde_yaml::to_string(&fm).expect("Failed to serialize frontmatter");
        format!("---\n{}---\n\n{}\n", yaml, self.body.trim())
    }
}
