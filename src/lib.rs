//! mdd_api: Parse, aggregate, and serialize Mammal Diversity Database data.
//!
//! This crate provides lightweight, lossless parsing of the Mammal Diversity
//! Database (MDD) CSV / TOML release artifacts plus convenience aggregation for
//! country distribution and synonym bundling.
//!
//! ## Modules
//! * `parser` – Low-level record parsers (`MddData`, `SynonymData`) and higher
//!   level bundles (`ReleasedMddData`, `AllMddData`, `CountryMDDStats`).
//! * `helper` – Utility helpers (country code normalization, constants).
//! * `writer` – Output helpers for serializing and writing processed data.
//!
//! ## Design Principles
//! * Preserve original text fields verbatim (no lossy normalization).
//! * Defer opinionated typing (e.g., coordinates, enumerations) to downstream callers.
//! * Provide predictable JSON via `serde` rename rules (camelCase alignment).
//! * Keep dependencies minimal.
//!
//! ## Quick Start
//! ```rust,no_run
//! use mdd_api::parser::{mdd::MddData, synonyms::SynonymData, ReleasedMddData};
//!
//! // Read CSV file contents into JSON `mdd_csv` and `syn_csv`.
//! let species = MddData::new().from_csv_to_json(&mdd_csv);
//! let synonyms = SynonymData::new().from_csv_to_json(&syn_csv);
//! let release = ReleasedMddData::from_parser(species, synonyms, "2025.1", "2025-09-01");
//! println!("{}", release.to_json());
//! ```
//!
//! See the README for more detailed workflow guidance.
pub mod helper;
pub mod parser;
pub mod writer;
