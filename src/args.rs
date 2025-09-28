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
    /// Parse MDD + synonym CSV files and export structured JSON (and optionally plain text outputs).
    #[command(name = "json", about = "Parse and export MDD data to JSON")]
    ToJson(JsonArgs),
    /// Convert parsed JSON into a SQLite database (implementation may still be evolving).
    #[command(name = "db", about = "Parse and export MDD data to SQLite database")]
    ToDb(DbArgs),
    /// Parse release metadata from a TOML file (and potentially drive batch exports).
    #[command(name = "toml", about = "Parse and export MDD data from TOML file")]
    FromToml(FromTomlArgs),
    /// Read compressed (zip) inputs (placeholder / help documentation stub).
    #[command(name = "zip", about = "Display help information")]
    FromZip(FromZipArgs),
}

/// Arguments for the `json` subcommand.
#[derive(Args)]
pub struct JsonArgs {
    /// Input MDD species CSV file.
    #[arg(long, short, default_value = "data.csv", help = "Input MDD CSV file")]
    pub input: PathBuf,
    /// Input synonym CSV file.
    #[arg(
        long,
        short,
        default_value = "synonyms.csv",
        help = "Input synonyms CSV file"
    )]
    pub synonym: PathBuf,
    /// Output directory for generated files.
    #[arg(
        long,
        short,
        default_value = "../assets/data",
        help = "Output directory"
    )]
    pub output: PathBuf,
    /// Whether to also export plain text data (if supported by writers).
    #[arg(long, short, help = "Export plain text data")]
    pub plain_text: bool,
    /// Override MDD version string for metadata embedding.
    #[arg(long = "mdd", help = "MDD data version", require_equals = true)]
    pub mdd_version: Option<String>,
    /// Override MDD release date (ISO 8601 expected: YYYY-MM-DD).
    #[arg(long = "date", help = "MDD release date")]
    pub release_date: Option<String>,
    /// Limit number of species records parsed (for debugging/testing).
    #[arg(long = "limit", help = "Limit number of records")]
    pub limit: Option<usize>,
    /// Add a file name prefix to all exported artifacts.
    #[arg(long, help = "Add prefix to output files")]
    pub prefix: Option<String>,
}

/// Arguments for the `db` subcommand (JSON to SQLite pipeline).
#[derive(Args)]
pub struct DbArgs {
    /// Input JSON file containing previously parsed MDD data.
    #[arg(long, short, default_value = "data.json", help = "Input MDD CSV file")]
    pub input: PathBuf,
}

/// Arguments for the `toml` subcommand (release metadata driven parsing).
#[derive(Args)]
pub struct FromTomlArgs {
    /// Input release TOML file path.
    #[arg(long, short, default_value = "data.toml", help = "Input MDD TOML file")]
    pub input: PathBuf,
    /// Output directory for generated artifacts.
    #[arg(long, short, default_value = ".", help = "Output directory")]
    pub output: PathBuf,
    /// Whether to export plain text along with JSON (if supported).
    #[arg(long, short, help = "Export plain text data")]
    pub plain_text: bool,
}

/// Arguments for the `zip` subcommand (compressed source processing).
#[derive(Args)]
pub struct FromZipArgs {
    /// Input ZIP archive containing release assets.
    #[arg(long, short, default_value = "MDD.zip", help = "Input MDD ZIP file")]
    pub input: PathBuf,
    /// Output directory for decompressed / processed content.
    #[arg(long, short, default_value = ".", help = "Output directory")]
    pub output: PathBuf,
}
