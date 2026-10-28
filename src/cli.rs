use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    List,
    Init { target: PathBuf },
    Update { target: PathBuf, dry_run: bool },
    Version,
}

pub fn default_target() -> io::Result<PathBuf> {
    std::env::current_dir()
}

pub fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let command = args
        .first()
        .and_then(|value| value.to_str())
        .ok_or_else(usage)?;
    match command {
        "list" if args.len() == 1 => Ok(Command::List),
        "version" if args.len() == 1 => Ok(Command::Version),
        "init" => parse_init(&args[1..]),
        "update" => parse_update(&args[1..]),
        _ => Err(usage()),
    }
}

fn parse_init(args: &[OsString]) -> Result<Command, String> {
    let mut target = None;
    for arg in args {
        if arg == "--here" {
            if target.is_some() {
                return Err(usage());
            }
            target = Some(default_target().map_err(|e| e.to_string())?);
        } else if target.is_none() {
            target = Some(PathBuf::from(arg));
        } else {
            return Err(usage());
        }
    }
    Ok(Command::Init {
        target: target.unwrap_or(default_target().map_err(|e| e.to_string())?),
    })
}

fn parse_update(args: &[OsString]) -> Result<Command, String> {
    let mut target = None;
    let mut dry_run = false;
    for arg in args {
        if arg == "--dry-run" {
            if dry_run {
                return Err(usage());
            }
            dry_run = true;
        } else if arg == "--here" {
            if target.is_some() {
                return Err(usage());
            }
            target = Some(default_target().map_err(|e| e.to_string())?);
        } else if target.is_none() {
            target = Some(PathBuf::from(arg));
        } else {
            return Err(usage());
        }
    }
    Ok(Command::Update {
        target: target.unwrap_or(default_target().map_err(|e| e.to_string())?),
        dry_run,
    })
}

fn usage() -> String {
    "usage: rudis-lite <list|init|update|version> [--here|--dry-run] [PROJECT_DIR]".into()
}
