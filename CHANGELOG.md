# CHANGELOGS

> NOTE: Early version history (<= 0.3.x) reconstructed retroactively from commit messages; some granular changes may be grouped.

## [0.6.0] - 2025-09-28

- Added: Parse release TOML file and integrate metadata into output.
- Added: Support locating `release.toml` inside zip archives.
- Added: Basic DOI field support in release metadata.
- Changed: Updated API around release metadata bundling.
- Fixed: Merge conflict resolutions and internal refactors to stabilize parsing.
- Docs: Expanded crate, module, and field-level documentation; added README usage guidance and examples.

## [0.5.0] - 2025-08-15

- Added: Include release metadata in export bundles.
- Fixed: Correct association of species and synonym-only parsing edge cases.

## [0.4.3] - 2025-07-31

- Fixed: Parsing failure for synonym-only data (missing species_id handling).

## [0.4.2] - 2025-06-14

- Fixed: Prefix path handling for output file naming.

## [0.4.1] - 2025-06-14

- Fixed: Default save path now current working directory when no path specified.

## [0.4.0] - 2025-06-14

- Added: Parse `MDD.zip` directly (zip extraction + CSV ingest).

## [0.3.0] - 2025-04-07

- Added: Synonym parsing improvements and test assets.
- Added: CLI improvements (display additional info, prefix support).
- Changed: Restructured data model for metadata inclusion.
- Fixed: Synonym parsing bug.

## [0.2.0] - 2025-02-15

- Added: Prefix flag for exported artifacts.
- Added: Metadata support (author/year/related fields) ingestion.
- Added: Ability to limit export size (debugging aid).
- Changed: Restructured data layout and output formatting.
- Fixed: CamelCase handling for column normalization.

## [0.1.0] - 2024-10-06

- Added: Initial parser for MDD species CSV (taxonomic + distribution fields).
- Added: Synonym parser and integration pipeline.
- Added: Database writer (SQLite) prototype with version + release date stamping.
- Added: JSON writer and CSV export utilities.
- Added: Basic CLI (`mdd`) with JSON export.
