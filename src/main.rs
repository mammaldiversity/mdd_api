//! # MDD API CLI
//!
//! Command-line interface for parsing Mammal Diversity Database (MDD) release
//! assets into structured JSON and related artifacts. See the crate-level docs
//! and README for fuller examples.
//!
//! ## Subcommands
//! * `json` – Parse species + synonym CSV files directly.
//! * `zip`  – Extract an MDD release archive (`MDD_v*.csv`, `Species_Syn_v*.csv`, optional `release.toml`) then parse.
//! * `toml` – (Placeholder) drive parsing via a release metadata TOML file.
//! * `db`   – (Placeholder) export into a SQLite database.
//!
//! ## JSON (`json`) Arguments
//! * `--input/-i` species CSV path (default: `data.csv`)
//! * `--synonym/-s` synonym CSV path (default: `synonyms.csv`)
//! * `--output/-o` output directory (default: `../assets/data`)
//! * `--plain-text/-p` also emit plain‑text (if supported)
//! * `--mdd=<ver>` override MDD version
//! * `--date <YYYY-MM-DD>` override release date
//! * `--limit <n>` limit number of species (debugging)
//! * `--prefix <str>` prefix output filenames
//!
//! ## ZIP (`zip`) Arguments
//! * `--input/-i` release archive path (default: `MDD.zip`)
//! * `--output/-o` extraction + output directory (default: `.`)
//!
//! ## Zip Quick Start
//! Minimal end‑to‑end example (also shown in README):
//!
//! ```text
//! mdd zip --input MDD_2025_1.zip --output ./out
//! # Produces JSON + stats (as implemented) under ./out
//! ```
//!
//! Programmatic parsing mirrors the `ZipParser` steps: open archive, locate the
//! `MDD_v*.csv` and `Species_Syn_v*.csv` entries, read to string, then feed into
//! `MddData::from_csv` and `SynonymData::from_csv` followed by
//! `ReleasedMddData::from_parser`.
//!
//! (Future work may stabilize a public helper around this flow.)
//!
use mdd_api::cli;

fn main() {
    cli::parse_args();
}
