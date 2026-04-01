use mnemosyne::config::Config;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_config() {
    let yaml = r#"
language_profiles:
  rust:
    markers: ["Cargo.toml"]
    extensions: [".rs"]
    dependency_file: "Cargo.toml"
    dependency_parser: cargo
  python:
    markers: ["pyproject.toml", "setup.py"]
    extensions: [".py"]
context_mappings:
  cargo_dependencies:
    tokio: [async, tokio, concurrency]
    sqlx: [database, sql]
"#;

    let config = Config::parse(yaml).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
    assert_eq!(config.language_profiles["rust"].markers, vec!["Cargo.toml"]);
    assert_eq!(config.language_profiles["rust"].extensions, vec![".rs"]);
    assert_eq!(
        config.language_profiles["rust"].dependency_parser.as_deref(),
        Some("cargo")
    );
    assert!(config.context_mappings.contains_key("cargo_dependencies"));
}

#[test]
fn test_load_config_from_file() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.yml");
    fs::write(
        &config_path,
        "language_profiles:\n  rust:\n    markers: [\"Cargo.toml\"]\n    extensions: [\".rs\"]\n",
    )
    .unwrap();

    let config = Config::load(tmp.path()).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_load_config_returns_defaults_when_missing() {
    let tmp = TempDir::new().unwrap();
    let config = Config::load(tmp.path()).unwrap();
    // Should have default language profiles
    assert!(!config.language_profiles.is_empty());
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_default_config_has_all_expected_languages() {
    let config = Config::default();
    let expected = [
        "rust", "python", "haskell", "ocaml", "prolog", "mercury",
        "scheme", "racket", "common-lisp", "smalltalk", "idris", "swift",
    ];
    for lang in expected {
        assert!(
            config.language_profiles.contains_key(lang),
            "Missing language profile: {}",
            lang
        );
    }
}
