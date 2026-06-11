//! MIL data preparation module.
//!
//! Exposes functions and types for parsing MIL metadata and matching against MDD records.

pub mod prep;

pub use prep::{
    extract_archive_if_compressed, prepare_metadata, read_file_as_records, to_camel, MilMddRecord,
};
