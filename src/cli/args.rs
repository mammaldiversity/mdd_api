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

use clap::{Args, Parser, Subcommand, crate_authors, crate_description, crate_name, crate_version};

use crate::helper::types::OutputFormat;

/// Top-level CLI dispatcher enumerating supported subcommands.
#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
pub enum Cli {
    #[command(name = "unpack", about = "Parse MDD data and export different formats")]
    Unpack(UnpackArgs),
    #[command(
        subcommand,
        name = "filter",
        about = "Filter MDD data by country codes"
    )]
    Filter(FilterSubcommand),
    #[command(name = "mil", about = "Merge MIL and MDD metadata and export as JSON")]
    Mil(MilArgs),
    #[command(name = "prepare", about = "Unpack MDD data and prepare MIL metadata")]
    Prepare(PrepareArgs),
}

#[derive(Subcommand)]
pub enum FilterSubcommand {
    #[command(name = "country", about = "Filter MDD data by country codes")]
    ByCountry(FilterByCountryArgs),
}

#[derive(Args)]
pub struct FilterByCountryArgs {
    #[command(flatten)]
    pub input: CommonInput,
    /// Output file path
    #[command(flatten)]
    pub output: CommonOutput,
    /// Country codes to filter by
    #[arg(
        long,
        short,
        value_delimiter = ',',
        help = "Country codes to filter by"
    )]
    pub country_codes: Vec<String>,
}

/// Arguments for the `zip` subcommand (compressed source processing).
#[derive(Args)]
pub struct UnpackArgs {
    #[command(flatten)]
    pub input: CommonInput,
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
    #[arg(
        value_enum,
        long,
        short = 'F',
        default_value = "csv",
        help = "Output file format"
    )]
    pub output_format: OutputFormat,
    /// Limit number of species records to parse.
    #[arg(long = "limit", help = "Limit number of records")]
    pub limit: Option<usize>,
    /// Add a file name prefix to all exported artifacts.
    #[arg(long, help = "Add prefix to output files")]
    pub prefix: Option<String>,
}

#[derive(Args)]
pub struct CommonInput {
    /// Input species CSV file path.
    #[arg(
        long,
        short,
        default_value = "MDD.csv",
        help = "Input MDD species CSV file"
    )]
    pub input: PathBuf,
    /// File format to export (json, db, toml, etc).
    #[arg(long, short = 'f', default_value = "zip", help = "Output format")]
    pub format: String,
}

/// Arguments for the `mil` subcommand.
#[derive(Args)]
pub struct MilArgs {
    /// Path to the MIL metadata file (CSV or Excel) or compressed archive (.tar.gz / .zip)
    #[arg(long, short = 'm', help = "Path to the MIL metadata file or compressed archive")]
    pub mil_file: PathBuf,

    /// Path to the MDD metadata file (CSV or Excel)
    #[arg(long, short = 'd', help = "Path to the MDD metadata file")]
    pub mdd_file: PathBuf,

    /// Path to the MIL image directory (optional if compressed archive is provided)
    #[arg(long, short = 'i', help = "Path to the MIL image directory")]
    pub mil_img_dir: Option<PathBuf>,

    /// Output path for the exported JSON file
    #[arg(long, short = 'o', help = "Output path for the exported JSON file")]
    pub output: PathBuf,
}

/// Arguments for the `prepare` subcommand.
#[derive(Args)]
pub struct PrepareArgs {
    /// Path to the MDD zip archive
    #[arg(long, short = 'z', help = "Path to the MDD zip archive")]
    pub mdd_zip: PathBuf,

    /// Path to the MIL metadata file (CSV or Excel) or compressed archive (.tar.gz / .zip)
    #[arg(long, short = 'm', help = "Path to the MIL metadata file or compressed archive")]
    pub mil_file: PathBuf,

    /// Path to the MIL image directory (optional if compressed archive is provided)
    #[arg(long, short = 'i', help = "Path to the MIL image directory")]
    pub mil_img_dir: Option<PathBuf>,

    /// Output directory for MDD unpacked files and prepared MIL JSON
    #[arg(long, short = 'o', help = "Output directory")]
    pub output: PathBuf,
}
