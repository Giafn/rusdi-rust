use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::templates;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub unchanged: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum Error {
    InvalidTarget(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(path) => {
                write!(f, "target is not an existing directory: {}", path.display())
            }
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for Error {}

pub fn init(target: &Path) -> Result<Report, Error> {
    install(target, false, true)
}
pub fn update(target: &Path, dry_run: bool) -> Result<Report, Error> {
    install(target, dry_run, false)
}
pub fn is_owned(content: &str) -> bool {
    let Some(rest) = content.strip_prefix("---\n") else {
        return content.starts_with(templates::OWNERSHIP_MARKER);
    };
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    rest[end + "\n---".len()..]
        .trim_start_matches('\n')
        .starts_with(templates::OWNERSHIP_MARKER)
}

fn install(target: &Path, dry_run: bool, create_missing: bool) -> Result<Report, Error> {
    if !target.is_dir() {
        return Err(Error::InvalidTarget(target.to_path_buf()));
    }
    let command_dir = target.join(".claude/commands");
    let mut report = Report::default();
    for template in templates::all() {
        let path = command_dir.join(format!("rudis.{}.md", template.name));
        if path.exists() {
            let existing = fs::read_to_string(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            if !is_owned(&existing) {
                report.skipped.push(path);
                continue;
            }
            if existing == templates::managed_content(template) {
                report.unchanged.push(path);
                continue;
            }
        } else if !create_missing {
            continue;
        }
        if dry_run {
            report.written.push(path);
            continue;
        }
        fs::create_dir_all(&command_dir).map_err(|source| Error::Io {
            path: command_dir.clone(),
            source,
        })?;
        write_atomic(&path, templates::managed_content(template))?;
        report.written.push(path);
    }
    Ok(report)
}

fn write_atomic(path: &Path, content: String) -> Result<(), Error> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        suffix
    ));
    fs::write(&temporary, content).map_err(|source| Error::Io {
        path: temporary.clone(),
        source,
    })?;
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(Error::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}
