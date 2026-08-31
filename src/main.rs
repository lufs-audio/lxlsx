//! lxlsx CLI.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use lxlsx::error::exit;
use lxlsx::{json, reader, registry, render, verify, Error, Snapshot};

#[derive(Parser)]
#[command(
    name = "lxlsx",
    version,
    about = "A contract-verified process registry for Excel workbooks"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render the review control panel (snapshot process).
    Snapshot {
        /// Path to finance.db
        #[arg(long, default_value = "data/finance.db")]
        db: PathBuf,
        /// Output .xlsx path
        #[arg(long, default_value = "data/exports/finances.xlsx")]
        out: PathBuf,
        /// Review anchor date (YYYY-MM-DD)
        #[arg(long)]
        last_met: Option<String>,
        /// Emit the JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Run a registered process.
    Run {
        slug: String,
        #[arg(long, default_value = "data/finance.db")]
        db: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// List registered processes.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Verify an emitted .xlsx against the layout contract.
    Verify {
        #[arg(long)]
        xlsx: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Regenerate (or check) the registry index.
    Index {
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = run(cli);
    match result {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(e) => {
            let code = e.exit_code();
            eprintln!("{e}");
            // For --json paths we'd emit the envelope; simplest correct behavior:
            // always emit the error envelope to stderr as JSON for agent parsability.
            let env = json::err(code, &e.to_string());
            eprintln!("{env}");
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
    }
}

#[allow(clippy::too_many_lines)]
fn run(cli: Cli) -> Result<i32, Error> {
    match cli.command {
        Command::Snapshot {
            db,
            out,
            last_met,
            json: as_json,
        } => {
            let last_met = last_met.unwrap_or_else(default_last_met);
            let snapshot = load_snapshot(&db, &last_met, &out)?;
            ensure_index()?;
            if as_json {
                let data = serde_json::json!({
                    "process": "snapshot",
                    "out": out.display().to_string(),
                    "last_met": snapshot.meta.last_met,
                    "totals": snapshot.totals,
                });
                println!("{}", json::ok(&data));
            } else {
                println!("Wrote {}", out.display());
            }
            Ok(exit::SUCCESS)
        }
        Command::Run {
            slug,
            db,
            out,
            json: as_json,
        } => {
            if slug != "snapshot" {
                return Err(Error::ProcessNotFound(slug));
            }
            let out = out.unwrap_or_else(|| PathBuf::from("data/exports/finances.xlsx"));
            let last_met = default_last_met();
            let snapshot = load_snapshot(&db, &last_met, &out)?;
            if as_json {
                println!(
                    "{}",
                    json::ok(
                        &serde_json::to_value(&snapshot.meta)
                            .map_err(|e| Error::Contract(e.to_string()))?
                    )
                );
            } else {
                println!("Wrote {}", out.display());
            }
            Ok(exit::SUCCESS)
        }
        Command::List { json: as_json } => {
            let manifest_dir = manifest_dir()?;
            let manifests = registry::discover(&manifest_dir)?;
            if as_json {
                println!(
                    "{}",
                    json::ok(
                        &serde_json::to_value(&manifests)
                            .map_err(|e| Error::Contract(e.to_string()))?
                    )
                );
            } else {
                for m in &manifests {
                    println!("{:<12} {}", m.slug, m.name);
                }
            }
            Ok(exit::SUCCESS)
        }
        Command::Verify {
            xlsx,
            json: as_json,
        } => {
            let report = verify::verify_xlsx(&xlsx)?;
            let ok = report.verdict == "verified";
            if as_json {
                println!(
                    "{}",
                    json::ok(
                        &serde_json::to_value(&report)
                            .map_err(|e| Error::Contract(e.to_string()))?
                    )
                );
            } else {
                for c in &report.checks {
                    println!(
                        "[{}] {} — {}",
                        if c.ok { "ok" } else { "X" },
                        c.name,
                        c.detail
                    );
                }
                println!("verdict: {}", report.verdict);
            }
            if ok {
                Ok(exit::SUCCESS)
            } else {
                Err(Error::Contract("verification failed".into()))
            }
        }
        Command::Index { check } => {
            let manifest_dir = manifest_dir()?;
            if check {
                if registry::index_is_current(&manifest_dir)? {
                    println!("index current");
                    Ok(exit::SUCCESS)
                } else {
                    Err(Error::Contract(
                        "registry index is stale; run `lxlsx index`".into(),
                    ))
                }
            } else {
                let p = registry::generate_index(&manifest_dir)?;
                println!("Wrote {}", p.display());
                Ok(exit::SUCCESS)
            }
        }
    }
}

fn load_snapshot(
    db: &std::path::Path,
    last_met: &str,
    out: &std::path::Path,
) -> Result<Snapshot, Error> {
    let conn = rusqlite::Connection::open(db)?;
    let snapshot = reader::build_snapshot(&conn, last_met, false)?;
    render::render_snapshot(&snapshot, out)?;
    let report = verify::verify_xlsx(out)?;
    if report.verdict != "verified" {
        return Err(Error::Contract(format!(
            "output failed verification: {}",
            report
                .checks
                .iter()
                .filter(|c| !c.ok)
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(snapshot)
}

fn default_last_met() -> String {
    // Without a config source, fall back to a fixed date (deterministic).
    "2026-08-15".to_string()
}

fn manifest_dir() -> Result<PathBuf, Error> {
    let dir = std::env::current_dir().map_err(Error::Io)?;
    Ok(dir)
}

fn ensure_index() -> Result<(), Error> {
    let dir = manifest_dir()?;
    if !registry::index_is_current(&dir)? {
        registry::generate_index(&dir)?;
    }
    Ok(())
}
