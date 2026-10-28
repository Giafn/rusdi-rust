use rudis_lite::{installer, templates};
use std::fs;

#[test]
fn init_installs_and_repeats_without_changing_owned_files() {
    let root = temp_dir("init");
    let first = installer::init(&root).unwrap();
    assert_eq!(first.written.len(), templates::all().len());
    let path = root.join(".claude/commands/rudis.specify.md");
    let original = fs::read_to_string(&path).unwrap();
    let second = installer::init(&root).unwrap();
    assert!(second.unchanged.contains(&path));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn init_preserves_unowned_file() {
    let root = temp_dir("unowned");
    let dir = root.join(".claude/commands");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rudis.specify.md");
    fs::write(&path, "user content\n").unwrap();
    let report = installer::init(&root).unwrap();
    assert!(report.skipped.contains(&path));
    assert_eq!(fs::read_to_string(path).unwrap(), "user content\n");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn dry_run_does_not_write() {
    let root = temp_dir("dry-run");
    let report = installer::update(&root, true).unwrap();
    assert!(report.written.is_empty());
    assert!(!root.join(".claude").exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_missing_target() {
    let root = std::env::temp_dir().join(format!("rudis-lite-missing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    assert!(installer::init(&root).is_err());
}

#[test]
fn rejects_file_target() {
    let root = temp_dir("file");
    let path = root.join("not-a-directory");
    fs::write(&path, "x").unwrap();
    assert!(installer::init(&path).is_err());
    let _ = fs::remove_dir_all(root);
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("rudis-lite-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
