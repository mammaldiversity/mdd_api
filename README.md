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
   `MddData::from_csv` (returns typed structs).
2. Read the synonym CSV and parse into `Vec<SynonymData>` via
   `SynonymData::from_csv`.
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

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
mdd_api = "0.6"
```

Minimal example parsing inline CSV strings and building a release bundle:

```rust
use mdd_api::parser::{mdd::MddData, synonyms::SynonymData, ReleasedMddData};

let mdd_csv = "id,sciName,mainCommonName,otherCommonNames,phylosort,subclass,infraclass,magnorder,superorder,order,suborder,infraorder,parvorder,superfamily,family,subfamily,tribe,genus,subgenus,specificEpithet,authoritySpeciesAuthor,authoritySpeciesYear,authorityParentheses,originalNameCombination,authoritySpeciesCitation,authoritySpeciesLink,typeVoucher,typeKind,typeVoucherURIs,typeLocality,typeLocalityLatitude,typeLocalityLongitude,nominalNames,taxonomyNotes,taxonomyNotesCitation,distributionNotes,distributionNotesCitation,subregionDistribution,countryDistribution,continentDistribution,biogeographicRealm,iucnStatus,extinct,domestic,flagged,CMW_sciName,diffSinceCMW,MSW3_matchtype,MSW3_sciName,diffSinceMSW3\n1,Panthera leo,Lion,,1,Theria,Eutheria,,Laurasiatheria,Carnivora,,,,Felidae,,,Panthera,,leo,Linnaeus,1758,0,,citation,,voucher,,uri,Locality,,,names,notes,,distNotes,,Subregion,Kenya|Tanzania,Africa,Afrotropic,LC,0,0,0,Name,0,match,Name,diff";
let syn_csv = "MDD_syn_id,hesp_id,species_id,species,root_name,author,year,authority_parentheses,nomenclature_status,validity,original_combination,original_rank,authority_citation,unchecked_authority_citation,sourced_unverified_citations,citation_group,citation_kind,authority_page,authority_link,authority_page_link,unchecked_authority_page_link,old_type_locality,original_type_locality,unchecked_type_locality,emended_type_locality,type_latitude,type_longitude,type_country,type_subregion,type_subregion2,holotype,type_kind,type_specimen_link,order,family,genus,specific_epithet,subspecific_epithet,variant_of,senior_homonym,variant_name_citations,name_usages,comments\n1,0,1,Panthera leo,Panthera leo,Linnaeus,1758,0,,valid,,species,citation,,,,,,link,,,loc,loc2,,loc3,0,0,Country,Sub,Sub2,Holotype,Kind,SpecLink,Carnivora,Felidae,Panthera,leo,,,,,,";

let species = MddData::new().from_csv(mdd_csv);
let synonyms = SynonymData::new().from_csv(syn_csv);
let release = ReleasedMddData::from_parser(species, synonyms, "2025.1", "2025-09-01");
println!("{}", release.to_json());
```

CLI usage (after installing with `cargo install mdd_api` or running from source):

```powershell
# Parse CSVs and output JSON bundle
mdd json --input mdd.csv --synonym synonyms.csv --output ./out --mdd="v2.0" --date 2025-09-01
```

### Zip Quick Start

If you have an official MDD release archive (for example `MDD.zip`) that
contains the species CSV (named like `MDD_v*.csv`), the synonym CSV
(`Species_Syn_v*.csv`), and optionally a `release.toml`, you can parse it in a
single step. The `zip` subcommand currently serves as a convenience entry point
and example; programmatic parsing typically gives you more control.

Programmatic (minimal) example using the internal `ZipParser` logic found in
`main.rs` (API surface may stabilize later):

```rust
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;
use mdd_api::parser::{mdd::MddData, synonyms::SynonymData, ReleasedMddData};

fn parse_from_zip<P: AsRef<Path>>(zip_path: P) -> anyhow::Result<ReleasedMddData> {
    // Open the archive
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Locate the two core CSV entries (pattern-matching the expected prefixes)
    let mut mdd_csv = String::new();
    let mut syn_csv = String::new();
    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        let name = f.name().to_string();
        if name.starts_with("MDD_v") && name.ends_with(".csv") {
            use std::io::Read; f.read_to_string(&mut mdd_csv)?;
        } else if name.starts_with("Species_Syn_v") && name.ends_with(".csv") {
            use std::io::Read; f.read_to_string(&mut syn_csv)?;
        }
    }

    // Parse into typed rows
    let species = MddData::new().from_csv(&mdd_csv);
    let synonyms = SynonymData::new().from_csv(&syn_csv);
    Ok(ReleasedMddData::from_parser(species, synonyms, "2025.1", "2025-09-01"))
}
```

CLI (auto-detects matching CSV names inside the archive):

```powershell
# Extract and parse directly from a ZIP archive; outputs JSON to current directory
mdd zip --input MDD.zip --output ./out
```

Notes:

- The current `zip` subcommand focuses on demonstration; future versions may
  emit multiple artifacts (e.g. filtered JSON, stats) similar to `json`.
- You can still manually unzip then invoke `mdd json -i <species.csv> -s <synonyms.csv>`
  if you prefer an explicit pipeline.

## Testing

Run all tests:

```powershell
cargo test
```

## License

See [LICENSE](LICENSE).
