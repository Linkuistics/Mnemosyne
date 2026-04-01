use mnemosyne::commands::install::run_install_claude_code;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_install_claude_code_copies_plugin() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("adapters/claude-code");
    let target = tmp.path().join("plugins/observational-memory");

    fs::create_dir_all(source.join("skills")).unwrap();
    fs::create_dir_all(source.join("references")).unwrap();
    fs::write(
        source.join("plugin.json"),
        r#"{"name": "observational-memory"}"#,
    )
    .unwrap();
    fs::write(source.join("skills/begin-work.md"), "# begin-work skill").unwrap();
    fs::write(source.join("references/guide.md"), "# guide").unwrap();

    run_install_claude_code(&source, &target).unwrap();

    assert!(target.join("plugin.json").exists());
    assert!(target.join("skills/begin-work.md").exists());
    assert!(target.join("references/guide.md").exists());
}

#[test]
fn test_install_preserves_existing_project_skills() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("adapters/claude-code");
    let target = tmp.path().join("plugins/observational-memory");

    fs::create_dir_all(source.join("skills")).unwrap();
    fs::write(
        source.join("plugin.json"),
        r#"{"name": "observational-memory"}"#,
    )
    .unwrap();
    fs::write(source.join("skills/begin-work.md"), "# updated begin-work").unwrap();

    fs::create_dir_all(target.join("skills")).unwrap();
    fs::write(target.join("skills/my-custom.md"), "# my custom skill").unwrap();
    fs::write(target.join("skills/begin-work.md"), "# old begin-work").unwrap();

    run_install_claude_code(&source, &target).unwrap();

    let custom = fs::read_to_string(target.join("skills/my-custom.md")).unwrap();
    assert_eq!(custom, "# my custom skill");

    let updated = fs::read_to_string(target.join("skills/begin-work.md")).unwrap();
    assert_eq!(updated, "# updated begin-work");
}
