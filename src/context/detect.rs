use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Language(String),
    Dependency { ecosystem: String, name: String },
    ProjectName(String),
}

pub struct ProjectDetector<'a> {
    config: &'a Config,
}

impl<'a> ProjectDetector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn detect(&self, project_root: &Path) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();
        self.detect_languages(project_root, &mut signals)?;
        self.detect_project_name(project_root, &mut signals)?;
        Ok(signals)
    }

    fn detect_languages(&self, root: &Path, signals: &mut Vec<Signal>) -> Result<()> {
        for (lang_name, profile) in &self.config.language_profiles {
            let detected = profile.markers.iter().any(|marker| {
                if marker.contains('*') {
                    self.glob_matches(root, marker)
                } else {
                    root.join(marker).exists()
                }
            });

            if !detected {
                let has_extension = profile
                    .extensions
                    .iter()
                    .any(|ext| self.has_files_with_extension(root, ext));
                if !has_extension {
                    continue;
                }
            }

            signals.push(Signal::Language(lang_name.clone()));

            if let Some(ref dep_file) = profile.dependency_file {
                if let Some(ref parser) = profile.dependency_parser {
                    let dep_path = root.join(dep_file);
                    if dep_path.exists() {
                        self.extract_dependencies(&dep_path, parser, signals)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn glob_matches(&self, root: &Path, pattern: &str) -> bool {
        let Ok(entries) = fs::read_dir(root) else {
            return false;
        };
        let suffix = pattern.trim_start_matches('*');
        entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().ends_with(suffix))
    }

    fn has_files_with_extension(&self, root: &Path, ext: &str) -> bool {
        let Ok(entries) = fs::read_dir(root) else {
            return false;
        };
        entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().ends_with(ext))
    }

    fn extract_dependencies(
        &self,
        dep_path: &Path,
        parser: &str,
        signals: &mut Vec<Signal>,
    ) -> Result<()> {
        let content = fs::read_to_string(dep_path)?;
        let ecosystem = format!("{}_dependencies", parser);

        match parser {
            "cargo" => self.parse_cargo_deps(&content, &ecosystem, signals),
            "pyproject" => self.parse_pyproject_deps(&content, &ecosystem, signals),
            _ => {}
        }
        Ok(())
    }

    fn parse_cargo_deps(&self, content: &str, ecosystem: &str, signals: &mut Vec<Signal>) {
        let mut in_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[dependencies]" || trimmed == "[dev-dependencies]" {
                in_deps = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_deps = false;
                continue;
            }
            if in_deps {
                if let Some(name) = trimmed.split('=').next() {
                    let name = name.trim();
                    if !name.is_empty() && !name.starts_with('#') {
                        signals.push(Signal::Dependency {
                            ecosystem: ecosystem.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    fn parse_pyproject_deps(&self, content: &str, ecosystem: &str, signals: &mut Vec<Signal>) {
        let mut in_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "dependencies = [" || trimmed == "[project.dependencies]" {
                in_deps = true;
                continue;
            }
            if in_deps && trimmed == "]" {
                in_deps = false;
                continue;
            }
            if in_deps {
                let clean = trimmed.trim_matches(|c: char| c == '"' || c == '\'' || c == ',');
                if let Some(name) = clean
                    .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .next()
                {
                    let name = name.trim();
                    if !name.is_empty() {
                        signals.push(Signal::Dependency {
                            ecosystem: ecosystem.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    fn detect_project_name(&self, root: &Path, signals: &mut Vec<Signal>) -> Result<()> {
        let git_config = root.join(".git/config");
        if git_config.exists() {
            let content = fs::read_to_string(&git_config)?;
            if let Some(name) = self.extract_project_name_from_git(&content) {
                signals.push(Signal::ProjectName(name));
                return Ok(());
            }
        }
        if let Some(dir_name) = root.file_name() {
            signals.push(Signal::ProjectName(dir_name.to_string_lossy().to_string()));
        }
        Ok(())
    }

    fn extract_project_name_from_git(&self, git_config: &str) -> Option<String> {
        for line in git_config.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("url = ") {
                let url = trimmed.strip_prefix("url = ")?;
                let name = url
                    .rsplit('/')
                    .next()?
                    .strip_suffix(".git")
                    .unwrap_or(url.rsplit('/').next()?);
                return Some(name.to_string());
            }
        }
        None
    }
}
