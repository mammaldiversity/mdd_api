# mdd_api

Application programming interface to interact with MDD (Mammal Diversity Database) data.

## Overview

This crate provides parsers and lightweight aggregation utilities for turning raw
MDD CSV / TOML release assets into structured Rust data types or JSON suitable
for downstream API or web delivery.

### Core Data Structures

- `MddData` – Single species row from the main MDD CSV (verbatim textual
  preservation of taxonomic + distribution + authority fields). Field‑level docs
  explain each column.
- `SynonymData` – Historical / alternative names and associated bibliographic
  metadata from the synonym CSV. The `MDD_` prefix is removed and headers are
  converted to camelCase.
- `CountryMDDStats` – Aggregated per‑country distribution statistics excluding
  domesticated and widespread placeholder entries. Predicted occurrences are
  marked with a trailing `?` on species IDs.
- `ReleasedMddData` – Compact bundle of simplified species + attached synonyms
  (only synonyms that resolve to an accepted species) plus summary `MetaData`.
- `AllMddData` – Full raw species + all synonyms without filtering.

### Release Metadata

A `release.toml` file (see `tests/data/release.toml` for an example) is parsed
into `ReleaseToml` / `ReleaseMetadata` and can be used to drive versioned output.

### Typical Workflow

1. Read the MDD species CSV and parse into `Vec<MddData>` using
   `MddData::from_csv_to_json` (despite the name it returns typed structs).
2. Read the synonym CSV and parse into `Vec<SynonymData>` via
   `SynonymData::from_csv_to_json`.
3. (Optional) Aggregate into a `ReleasedMddData` with
   `ReleasedMddData::from_parser` providing the desired version + release date.
4. Serialize to JSON or gzip using standard tooling.
5. (Optional) Build `CountryMDDStats` for geographic summaries.

### Country Statistics

The parser normalizes country / region names via helper code. Unrecognized
names are kept verbatim and a warning is emitted. Species with a distribution of
`domesticated` or `NA` are collected separately and excluded from per‑country
counts.

### Extensibility

The crate intentionally keeps most columns as `String` to avoid lossy
assumptions. Applications needing strict numeric coordinates or enumerated
status codes can layer additional domain models on top.

## Testing

Run all tests:

```powershell
cargo test
```

## License

See [LICENSE](LICENSE).
