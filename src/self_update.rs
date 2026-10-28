use crate::{download, github, platform, release::Release};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum UpdateError {
    UnsupportedTarget(String),
    InvalidRelease(String),
    MissingAsset(String),
    InvalidChecksum(String),
    ChecksumMismatch { expected: String, actual: String },
    Network(String),
    Io { path: PathBuf, source: io::Error },
}
impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTarget(v) => write!(f, "unsupported platform: {v}"),
            Self::InvalidRelease(v) => write!(f, "invalid GitHub release: {v}"),
            Self::MissingAsset(v) => write!(f, "release asset not found: {v}"),
            Self::InvalidChecksum(v) => write!(f, "invalid checksum line: {v}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "checksum mismatch (expected {expected}, got {actual})")
            }
            Self::Network(v) => write!(f, "network error: {v}"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}
impl std::error::Error for UpdateError {}

#[derive(Debug, PartialEq, Eq)]
pub struct UpdateResult {
    pub old_version: String,
    pub new_version: String,
    pub changed: bool,
}

pub fn run() -> Result<UpdateResult, UpdateError> {
    let current = platform::current()?;
    let release = github::latest_release()?;
    let new_version = release.tag_name.trim_start_matches('v').to_string();
    let old_version = env!("CARGO_PKG_VERSION").to_string();
    if !is_newer(&old_version, &new_version) {
        return Ok(UpdateResult {
            old_version,
            new_version,
            changed: false,
        });
    }
    let (binary, sums) = github::release_assets(&release, current)?;
    let binary_bytes = get_bytes(&binary.browser_download_url)?;
    let checksums = download::parse_checksums(&get_text(&sums.browser_download_url)?)?;
    download::verify_sha256(
        &binary_bytes,
        checksums
            .get(&binary.name)
            .ok_or_else(|| UpdateError::MissingAsset(binary.name.clone()))?,
    )?;
    replace_current(
        &std::env::current_exe().map_err(|source| UpdateError::Io {
            path: PathBuf::from("current executable"),
            source,
        })?,
        &binary_bytes,
    )?;
    Ok(UpdateResult {
        old_version,
        new_version,
        changed: true,
    })
}

pub fn is_newer(old: &str, new: &str) -> bool {
    new.split('.')
        .map(|v| v.parse::<u64>().unwrap_or(0))
        .collect::<Vec<_>>()
        > old
            .split('.')
            .map(|v| v.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
}

fn get_bytes(url: &str) -> Result<Vec<u8>, UpdateError> {
    let response = ureq::get(url)
        .header("User-Agent", "rudis-lite")
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    response
        .into_body()
        .read_to_vec()
        .map_err(|e| UpdateError::Network(e.to_string()))
}
fn get_text(url: &str) -> Result<String, UpdateError> {
    String::from_utf8(get_bytes(url)?).map_err(|e| UpdateError::Network(e.to_string()))
}
pub fn replace_current(executable: &Path, replacement: &[u8]) -> Result<(), UpdateError> {
    let temporary = executable.with_extension("rudis-lite-update.tmp");
    fs::write(&temporary, replacement).map_err(|source| UpdateError::Io {
        path: temporary.clone(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(executable)
            .map_err(|source| UpdateError::Io {
                path: executable.into(),
                source,
            })?
            .permissions();
        p.set_mode(p.mode() | 0o111);
        fs::set_permissions(&temporary, p).map_err(|source| UpdateError::Io {
            path: temporary.clone(),
            source,
        })?;
    }
    if let Err(source) = fs::rename(&temporary, executable) {
        let _ = fs::remove_file(&temporary);
        return Err(UpdateError::Io {
            path: executable.into(),
            source,
        });
    }
    Ok(())
}

pub fn parse_release(json: &str) -> Result<Release, UpdateError> {
    Release::parse(json)
}
