use mnemosyne::config::Config;
use mnemosyne::context::detect::{ProjectDetector, Signal};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_rust_project() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n[dependencies]\ntokio = \"1\"\n",
    )
    .unwrap();
    fs::create_dir(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::Language(lang) if lang == "rust")));
}

#[test]
fn test_detect_python_project() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
    fs::write(tmp.path().join("main.py"), "print('hello')").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::Language(lang) if lang == "python")));
}

#[test]
fn test_detect_project_name_from_git() {
    let tmp = TempDir::new().unwrap();
    let git_dir = tmp.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(
        git_dir.join("config"),
        "[remote \"origin\"]\n\turl = git@github.com:user/my-project.git\n",
    )
    .unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    assert!(signals.iter().any(|s| matches!(s, Signal::ProjectName(name) if name == "my-project")));
}

#[test]
fn test_detect_multiple_languages() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    fs::write(tmp.path().join("Package.swift"), "// swift-tools-version:5.5\n").unwrap();

    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let signals = detector.detect(tmp.path()).unwrap();

    let languages: Vec<&str> = signals
        .iter()
        .filter_map(|s| match s {
            Signal::Language(l) => Some(l.as_str()),
            _ => None,
        })
        .collect();
    assert!(languages.contains(&"rust"));
    assert!(languages.contains(&"swift"));
}
