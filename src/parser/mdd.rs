//! Parse MDD csv data into a structured format.

use serde::{Deserialize, Serialize};

/// Primary record representing a single species row from the Mammal Diversity Database (MDD)
/// CSV export.
///
/// Field names intentionally follow (or are mapped from) the original column
/// headings so downstream JSON can be matched to the source data. Most fields
/// are kept as `String` because the raw CSV frequently contains empty values,
/// free‑form text, mixed formatting, uncertain markers (e.g. trailing `?`), or
/// ranges that would otherwise require lossy normalization. Consumers can add
/// typed layers on top if needed.
///
/// Notes:
/// * All taxonomic rank fields (order/family/genus, etc.) contain the exact
///   verbatim strings from the source.
/// * `taxon_order` uses `#[serde(alias = "order")]` because `order` is a Rust
///   keyword; deserialization will still accept an `order` column.
/// * Boolean style flags (extinct/domestic/flagged) are encoded as `u8` (0/1)
///   to match the CSV and avoid custom (de)serialization.
/// * Coordinate and locality fields remain textual because the source may
///   contain composite, approximate, or blank entries.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MddData {
    /// Unique numeric identifier for the species record (MDD internal ID).
    pub id: u32,
    /// Full scientific binomial (potentially including infraspecific parts) as used in MDD.
    pub sci_name: String,
    /// Primary English common name selected by MDD editors.
    pub main_common_name: String,
    /// Alternate common names (pipe or comma separated in source); left verbatim.
    pub other_common_names: String,
    /// Phylogenetic sort index supplied by MDD for reproducible ordering.
    pub phylosort: u16,
    /// Taxonomic subclass (e.g., Theria).
    pub subclass: String,
    /// Taxonomic infraclass.
    pub infraclass: String,
    /// Taxonomic magnorder.
    pub magnorder: String,
    /// Taxonomic superorder.
    pub superorder: String,
    // order is a reserved keyword in Rust.
    // So, we need to rename it to taxonOrder.
    // #[serde(rename(serialize = "taxonOrder", deserialize = "order"))]
    /// Taxonomic order (aliased from `order` in the CSV to avoid the Rust keyword).
    #[serde(alias = "order")]
    pub taxon_order: String,
    /// Suborder rank.
    pub suborder: String,
    /// Infraorder rank.
    pub infraorder: String,
    /// Parvorder rank.
    pub parvorder: String,
    /// Superfamily rank.
    pub superfamily: String,
    /// Family rank.
    pub family: String,
    /// Subfamily rank.
    pub subfamily: String,
    /// Tribe rank.
    pub tribe: String,
    /// Genus name.
    pub genus: String,
    /// Subgenus (often blank).
    pub subgenus: String,
    /// Specific epithet portion of the binomial.
    pub specific_epithet: String,
    /// Author(s) of the original species description (verbatim formatting).
    pub authority_species_author: String,
    /// Year of the original species description (0 when unknown/missing).
    pub authority_species_year: u16,
    /// 1 if author & year are presented in parentheses (indicating original combination differs), else 0.
    pub authority_parentheses: u8,
    /// Original name combination string as published (verbatim).
    pub original_name_combination: String,
    /// Full citation for original species description.
    pub authority_species_citation: String,
    /// External link (URL/URI) for the species authority reference if supplied.
    pub authority_species_link: String,
    /// Type specimen voucher identifier (verbatim), if supplied.
    pub type_voucher: String,
    /// Type specimen kind (e.g., holotype, lectotype) if provided.
    pub type_kind: String,
    /// Resolved URI(s) for the type voucher (field header `typeVoucherURIs`).
    #[serde(rename = "typeVoucherURIs")]
    pub type_voucher_uri: String,
    /// Textual type locality description.
    pub type_locality: String,
    /// Latitude (string form; may include symbols or be blank).
    pub type_locality_latitude: String,
    /// Longitude (string form; may include symbols or be blank).
    pub type_locality_longitude: String,
    /// Nominal names / synonymy notes maintained by MDD (verbatim).
    pub nominal_names: String,
    /// Free‑form notes about taxonomic decisions.
    pub taxonomy_notes: String,
    /// Citations supporting `taxonomy_notes`.
    pub taxonomy_notes_citation: String,
    /// Free‑form notes about distribution peculiarities.
    pub distribution_notes: String,
    /// Citations supporting `distribution_notes`.
    pub distribution_notes_citation: String,
    /// Subregional distribution string (MDD controlled vocab / free text mix).
    pub subregion_distribution: String,
    /// Country distribution string (pipe separated list, or textual labels like "domesticated" / "NA").
    pub country_distribution: String,
    /// Continent distribution categories.
    pub continent_distribution: String,
    /// Biogeographic realm assignment.
    pub biogeographic_realm: String,
    /// IUCN Red List status code (verbatim at time of data export).
    pub iucn_status: String,
    /// 1 if species is considered extinct (recently extinct category), else 0.
    pub extinct: u8,
    /// 1 if species is domestic/domesticated form, else 0.
    pub domestic: u8,
    /// Internal flagged indicator (meaning defined by upstream MDD source) 0/1.
    pub flagged: u8,
    /// CMW (Coldwell / or another reference set) scientific name field (exact mapping from `CMW_sciName`).
    #[serde(rename = "CMW_sciName")]
    pub cmw_sci_name: String,
    /// Difference flag vs CMW reference (0/1) from `diffSinceCMW`.
    #[serde(rename = "diffSinceCMW")]
    pub diff_since_cmw: u8,
    /// Match type vs MSW3 taxonomy (`MSW3_matchtype`).
    #[serde(rename = "MSW3_matchtype")]
    pub msw3_match_type: String,
    /// MSW3 scientific name (`MSW3_sciName`).
    #[serde(rename = "MSW3_sciName")]
    pub msw3_sci_name: String,
    /// Description of differences relative to MSW3 (`diffSinceMSW3`).
    #[serde(rename = "diffSinceMSW3")]
    pub diff_since_msw3: String,
}

impl MddData {
    pub fn new() -> Self {
        Self {
            id: 0,
            phylosort: 0,
            subclass: "".to_string(),
            infraclass: "".to_string(),
            magnorder: "".to_string(),
            superorder: "".to_string(),
            taxon_order: "".to_string(),
            suborder: "".to_string(),
            infraorder: "".to_string(),
            parvorder: "".to_string(),
            superfamily: "".to_string(),
            family: "".to_string(),
            subfamily: "".to_string(),
            tribe: "".to_string(),
            genus: "".to_string(),
            subgenus: "".to_string(),
            specific_epithet: "".to_string(),
            sci_name: "".to_string(),
            authority_species_author: "".to_string(),
            authority_species_year: 0,
            authority_parentheses: 0,
            main_common_name: "".to_string(),
            other_common_names: "".to_string(),
            original_name_combination: "".to_string(),
            authority_species_citation: "".to_string(),
            authority_species_link: "".to_string(),
            type_voucher: "".to_string(),
            type_kind: "".to_string(),
            type_voucher_uri: "".to_string(),
            type_locality: "".to_string(),
            type_locality_latitude: "".to_string(),
            type_locality_longitude: "".to_string(),
            nominal_names: "".to_string(),
            taxonomy_notes: "".to_string(),
            taxonomy_notes_citation: "".to_string(),
            distribution_notes: "".to_string(),
            distribution_notes_citation: "".to_string(),
            subregion_distribution: "".to_string(),
            country_distribution: "".to_string(),
            continent_distribution: "".to_string(),
            biogeographic_realm: "".to_string(),
            iucn_status: "".to_string(),
            extinct: 0,
            domestic: 0,
            flagged: 0,
            cmw_sci_name: "".to_string(),
            diff_since_cmw: 0,
            msw3_match_type: "".to_string(),
            msw3_sci_name: "".to_string(),
            diff_since_msw3: "".to_string(),
        }
    }

    /// Parse csv data to json.
    /// Return in String json format.
    pub fn from_csv_to_json(&self, csv_data: &str) -> Vec<MddData> {
        let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
        let mut records = Vec::new();
        for result in rdr.deserialize() {
            let record: Self = result.unwrap();
            records.push(record);
        }
        records
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_parse_to_json() {
        let csv_data = Path::new("tests/data/test_data.csv");
        let csv_data = std::fs::read_to_string(csv_data).unwrap();
        let parser = MddData::new();
        let json_data = parser.from_csv_to_json(&csv_data);
        // let data = AllMddData::from_json(&json_data);
        assert_eq!(json_data.len(), 112);
    }
}
