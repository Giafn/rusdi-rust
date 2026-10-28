use rudis_lite::cli::{parse_args, Command};
use rudis_lite::installer;
use rudis_lite::self_update;
use rudis_lite::templates;

fn main() {
    if let Err(error) = run() {
        eprintln!("rudis-lite: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(std::env::args_os().skip(1))? {
        Command::List => {
            for template in templates::all() {
                println!("/rudis.{}", template.name);
            }
        }
        Command::Version => println!(
            "rudis-lite {} (upstream {})",
            env!("CARGO_PKG_VERSION"),
            templates::UPSTREAM_REVISION
        ),
        Command::SelfUpdate => {
            let result = self_update::run().map_err(|e| e.to_string())?;
            if result.changed {
                println!(
                    "updated rudis-lite {} -> {}",
                    result.old_version, result.new_version
                );
            } else {
                println!("rudis-lite {} is already current", result.old_version);
            }
        }
        Command::Init { target } => {
            print_report(installer::init(&target).map_err(|e| e.to_string())?)
        }
        Command::Update { target, dry_run } => {
            print_report(installer::update(&target, dry_run).map_err(|e| e.to_string())?)
        }
    }
    Ok(())
}

fn print_report(report: installer::Report) {
    for path in report.written {
        println!("written {}", path.display());
    }
    for path in report.unchanged {
        println!("unchanged {}", path.display());
    }
    for path in report.skipped {
        println!("skipped {}", path.display());
    }
}
