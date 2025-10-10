//! Command-line interface argument definitions for the `mdd_api` tooling.
//!
//! Uses `clap` derive macros to expose subcommands for converting raw Mammal
//! Diversity Database (MDD) release assets into JSON, database, or other
//! serialized forms.
//!
//! Subcommands:
//! * `json`  – Parse species + synonym CSVs and export JSON (optionally limit or prefix files).
//! * `db`    – (Planned/placeholder) ingest JSON into a SQLite database.
//! * `toml`  – Parse release metadata TOML plus referenced CSVs (future expansion).
//! * `zip`   – Parse directly from a zipped archive (future/support tooling).
//!
//! Most file path arguments default to relative names to simplify quick starts;
//! override them for production workflows.

use std::path::PathBuf;

use clap::{crate_authors, crate_description, crate_name, crate_version, Args, Parser};

/// Top-level CLI dispatcher enumerating supported subcommands.
#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
pub enum Cli {
    #[command(name = "unpack", about = "Parse MDD data and export different formats")]
    Unpack(UnpackArgs),
}

/// Arguments for the `zip` subcommand (compressed source processing).
#[derive(Args)]
pub struct UnpackArgs {
    /// Input ZIP archive containing release assets.
    #[arg(long, short, default_value = "MDD.zip", help = "Input MDD ZIP file")]
    pub input: PathBuf,
    /// File format to export (json, db, toml, etc).
    #[arg(long, short, default_value = "zip", help = "Output format")]
    pub format: String,
    /// Override MDD version string for metadata embedding.
    #[arg(long = "mdd", help = "MDD data version", require_equals = true)]
    pub mdd_version: Option<String>,
    /// Override MDD release date (ISO 8601 expected: YYYY-MM-DD).
    #[arg(long = "date", help = "MDD release date")]
    pub release_date: Option<String>,
    #[command(flatten)]
    pub output: CommonOutput,
}

#[derive(Args)]
pub struct CommonOutput {
    /// Output directory for decompressed / processed content.
    #[arg(long, short, default_value = ".", help = "Output directory")]
    pub output: PathBuf,
    /// Output format
    #[arg(long, short = 'F', default_value = "gzip", help = "Output file format")]
    pub output_format: String,
    /// Limit number of species records to parse.
    #[arg(long = "limit", help = "Limit number of records")]
    pub limit: Option<usize>,
    /// Add a file name prefix to all exported artifacts.
    #[arg(long, help = "Add prefix to output files")]
    pub prefix: Option<String>,
}
