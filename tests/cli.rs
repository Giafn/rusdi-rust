use std::path::PathBuf;

use rudis_lite::cli::{parse_args, Command};

#[test]
fn parses_init_with_target() {
    assert_eq!(
        parse_args(["init", "/tmp/project"]).unwrap(),
        Command::Init {
            target: PathBuf::from("/tmp/project")
        }
    );
}

#[test]
fn parses_update_dry_run() {
    assert_eq!(
        parse_args(["update", "--dry-run"]).unwrap(),
        Command::Update {
            target: std::env::current_dir().unwrap(),
            dry_run: true
        }
    );
}

#[test]
fn parses_list_and_version() {
    assert_eq!(parse_args(["list"]).unwrap(), Command::List);
    assert_eq!(parse_args(["version"]).unwrap(), Command::Version);
}

#[test]
fn rejects_unknown_and_extra_arguments() {
    assert!(parse_args(["wat"]).is_err());
    assert!(parse_args(["list", "extra"]).is_err());
    assert!(parse_args(["init", "--here", "/tmp/project"]).is_err());
}
