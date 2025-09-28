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
//! ```rust
//! use mdd_api::parser::{mdd::MddData, synonyms::SynonymData, ReleasedMddData};
//!
//! // Minimal inline mock CSV data (headers trimmed for brevity)
//! let mdd_csv = "id,sciName,mainCommonName,otherCommonNames,phylosort,subclass,infraclass,magnorder,superorder,order,suborder,infraorder,parvorder,superfamily,family,subfamily,tribe,genus,subgenus,specificEpithet,authoritySpeciesAuthor,authoritySpeciesYear,authorityParentheses,originalNameCombination,authoritySpeciesCitation,authoritySpeciesLink,typeVoucher,typeKind,typeVoucherURIs,typeLocality,typeLocalityLatitude,typeLocalityLongitude,nominalNames,taxonomyNotes,taxonomyNotesCitation,distributionNotes,distributionNotesCitation,subregionDistribution,countryDistribution,continentDistribution,biogeographicRealm,iucnStatus,extinct,domestic,flagged,CMW_sciName,diffSinceCMW,MSW3_matchtype,MSW3_sciName,diffSinceMSW3\n1,Panthera leo,Lion,,1,Theria,Eutheria,,Laurasiatheria,Carnivora,,,,Felidae,,,
//! Panthera,,leo,Linnaeus,1758,0,,citation,,voucher,,uri,Locality,,,names,notes,,distNotes,,Subregion,"Kenya|Tanzania",Africa,Afrotropic,LC,0,0,0,Name,0,match,Name,diff";
//!
//! let syn_csv = "MDD_syn_id,hesp_id,species_id,species,root_name,author,year,authority_parentheses,nomenclature_status,validity,original_combination,original_rank,authority_citation,unchecked_authority_citation,sourced_unverified_citations,citation_group,citation_kind,authority_page,authority_link,authority_page_link,unchecked_authority_page_link,old_type_locality,original_type_locality,unchecked_type_locality,emended_type_locality,type_latitude,type_longitude,type_country,type_subregion,type_subregion2,holotype,type_kind,type_specimen_link,order,family,genus,specific_epithet,subspecific_epithet,variant_of,senior_homonym,variant_name_citations,name_usages,comments\n1,0,1,Panthera leo,Panthera leo,Linnaeus,1758,0,,valid,,species,citation,,,,,,link,,,loc,loc2,,loc3,0,0,Country,Sub,Sub2,Holotype,Kind,SpecLink,Carnivora,Felidae,Panthera,leo,,,,,,";
//!
//! let species = MddData::new().from_csv(mdd_csv);
//! let synonyms = SynonymData::new().from_csv(syn_csv);
//! let release = ReleasedMddData::from_parser(species, synonyms, "2025.1", "2025-09-01");
//! println!("{}", release.to_json());
//! ```
//!
//! See the README for more detailed workflow guidance.
pub mod helper;
pub mod parser;
pub mod writer;
