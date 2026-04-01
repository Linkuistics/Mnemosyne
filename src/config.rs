use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProfile {
    #[serde(default)]
    pub markers: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub dependency_file: Option<String>,
    #[serde(default)]
    pub dependency_parser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub language_profiles: HashMap<String, LanguageProfile>,
    #[serde(default)]
    pub context_mappings: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Config {
    pub fn parse(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse config YAML")
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let config_path = dir.join("config.yml");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            Self::parse(&content)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let config_path = dir.join("config.yml");
        let yaml = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(&config_path, yaml)
            .with_context(|| format!("Failed to write {}", config_path.display()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language_profiles: default_language_profiles(),
            context_mappings: default_context_mappings(),
        }
    }
}

fn default_language_profiles() -> HashMap<String, LanguageProfile> {
    let mut profiles = HashMap::new();

    profiles.insert(
        "rust".into(),
        LanguageProfile {
            markers: vec!["Cargo.toml".into()],
            extensions: vec![".rs".into()],
            dependency_file: Some("Cargo.toml".into()),
            dependency_parser: Some("cargo".into()),
        },
    );
    profiles.insert(
        "python".into(),
        LanguageProfile {
            markers: vec![
                "pyproject.toml".into(),
                "setup.py".into(),
                "requirements.txt".into(),
            ],
            extensions: vec![".py".into()],
            dependency_file: Some("pyproject.toml".into()),
            dependency_parser: Some("pyproject".into()),
        },
    );
    profiles.insert(
        "haskell".into(),
        LanguageProfile {
            markers: vec![
                "*.cabal".into(),
                "stack.yaml".into(),
                "cabal.project".into(),
            ],
            extensions: vec![".hs".into()],
            dependency_file: None,
            dependency_parser: Some("cabal".into()),
        },
    );
    profiles.insert(
        "ocaml".into(),
        LanguageProfile {
            markers: vec!["dune-project".into(), "*.opam".into()],
            extensions: vec![".ml".into(), ".mli".into()],
            dependency_file: None,
            dependency_parser: Some("opam".into()),
        },
    );
    profiles.insert(
        "prolog".into(),
        LanguageProfile {
            markers: vec!["pack.pl".into()],
            extensions: vec![".pl".into(), ".pro".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "mercury".into(),
        LanguageProfile {
            markers: vec!["Mercury.options".into()],
            extensions: vec![".m".into(), ".mh".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "scheme".into(),
        LanguageProfile {
            markers: vec![],
            extensions: vec![".scm".into(), ".ss".into(), ".sld".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "racket".into(),
        LanguageProfile {
            markers: vec!["info.rkt".into()],
            extensions: vec![".rkt".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "common-lisp".into(),
        LanguageProfile {
            markers: vec!["*.asd".into()],
            extensions: vec![".lisp".into(), ".cl".into(), ".lsp".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "smalltalk".into(),
        LanguageProfile {
            markers: vec![".smalltalk.ston".into()],
            extensions: vec![".st".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "idris".into(),
        LanguageProfile {
            markers: vec!["*.ipkg".into()],
            extensions: vec![".idr".into()],
            dependency_file: None,
            dependency_parser: None,
        },
    );
    profiles.insert(
        "swift".into(),
        LanguageProfile {
            markers: vec!["Package.swift".into()],
            extensions: vec![".swift".into()],
            dependency_file: Some("Package.swift".into()),
            dependency_parser: None,
        },
    );

    profiles
}

fn default_context_mappings() -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut mappings = HashMap::new();

    let mut cargo = HashMap::new();
    cargo.insert(
        "tokio".into(),
        vec!["async".into(), "tokio".into(), "concurrency".into()],
    );
    cargo.insert(
        "sqlx".into(),
        vec!["database".into(), "sql".into(), "async".into()],
    );
    cargo.insert(
        "axum".into(),
        vec!["web".into(), "http".into(), "api".into()],
    );
    cargo.insert(
        "serde".into(),
        vec!["serialization".into(), "serde".into()],
    );
    mappings.insert("cargo_dependencies".into(), cargo);

    mappings
}
