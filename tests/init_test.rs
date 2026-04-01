use std::fs;
use tempfile::TempDir;
use mnemosyne::commands::init::run_init;

#[test]
fn test_init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    assert!(mnemosyne_dir.exists());
    assert!(mnemosyne_dir.join("knowledge").exists());
    assert!(mnemosyne_dir.join("knowledge/languages").exists());
    assert!(mnemosyne_dir.join("knowledge/domains").exists());
    assert!(mnemosyne_dir.join("knowledge/tools").exists());
    assert!(mnemosyne_dir.join("knowledge/techniques").exists());
    assert!(mnemosyne_dir.join("knowledge/projects").exists());
    assert!(mnemosyne_dir.join("archive").exists());
    assert!(mnemosyne_dir.join("cache").exists());
    assert!(mnemosyne_dir.join("config.yml").exists());
    assert!(mnemosyne_dir.join(".gitignore").exists());
}

#[test]
fn test_init_creates_valid_config() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    let config = mnemosyne::config::Config::load(&mnemosyne_dir).unwrap();
    assert!(config.language_profiles.contains_key("rust"));
}

#[test]
fn test_init_refuses_if_already_exists() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();
    let result = run_init(&mnemosyne_dir, None);

    assert!(result.is_err());
}

#[test]
fn test_init_creates_gitignore() {
    let tmp = TempDir::new().unwrap();
    let mnemosyne_dir = tmp.path().join(".mnemosyne");

    run_init(&mnemosyne_dir, None).unwrap();

    let gitignore = fs::read_to_string(mnemosyne_dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("cache/"));
}
