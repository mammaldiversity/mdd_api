//! Parser utilities for MDD synonym CSV exports.
//!
//! The synonym file contains historical / alternative taxonomic names and
//! associated bibliographic metadata. Column headers in the raw CSV are in
//! snake_case and prefixed with `MDD_`. We strip that prefix and convert the
//! remaining portion to camelCase so that serialized JSON aligns with other
//! structs in this crate.

use convert_case::Casing;
use serde::{Deserialize, Serialize};

/// Representation of a single synonym (or name usage) row from the MDD synonyms
/// CSV.
///
/// Implementation details:
/// * Most fields are `String` because the source frequently includes empty
///   strings, mixed formatting, or free text that would be lossy if coerced.
/// * `species_id` is optional: some synonym rows may not resolve to a current
///   MDD species (e.g., unused combinations or uncertain placements). Those
///   entries will appear in `ReleasedMddData.synonym_only` during aggregation.
/// * Authority / citation fields retain upstream capitalization and punctuation.
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SynonymData {
    /// Unique synonym identifier (internal to MDD synonym table).
    pub syn_id: u32,
    /// HESP (Historical Ecology / or upstream) external ID when provided.
    pub hesp_id: u32,
    /// Foreign key linking to an MDD species record; absent if not attached to a living/accepted taxon.
    pub species_id: Option<u32>,
    // Below are raw text columns retained verbatim. We keep them private and provide JSON via serde.
    species: String,
    root_name: String,
    author: String,
    year: String,
    /// 1 when the authority is cited in parentheses, 0 when not, and `None`
    /// when unknown (the CSV marks a few rows with "?").
    #[serde(default, deserialize_with = "deserialize_parentheses")]
    authority_parentheses: Option<u8>,
    nomenclature_status: String,
    validity: String,
    original_combination: String,
    original_rank: String,
    authority_citation: String,
    unchecked_authority_citation: String,
    sourced_unverified_citations: String,
    citation_group: String,
    citation_kind: String,
    authority_page: String,
    authority_link: String,
    authority_page_link: String,
    unchecked_authority_page_link: String,
    old_type_locality: String,
    original_type_locality: String,
    unchecked_type_locality: String,
    emended_type_locality: String,
    type_latitude: String,
    type_longitude: String,
    type_country: String,
    type_subregion: String,
    type_subregion2: String,
    holotype: String,
    type_kind: String,
    type_specimen_link: String,
    #[serde(alias = "order")]
    taxon_order: String,
    family: String,
    genus: String,
    specific_epithet: String,
    subspecific_epithet: String,
    variant_of: String,
    senior_homonym: String,
    variant_name_citations: String,
    name_usages: String,
    comments: String,
}

impl SynonymData {
    pub fn new() -> Self {
        Self {
            syn_id: 0,
            hesp_id: 0,
            species_id: None,
            species: "".to_string(),
            root_name: "".to_string(),
            author: "".to_string(),
            year: "".to_string(),
            authority_parentheses: None,
            nomenclature_status: "".to_string(),
            validity: "".to_string(),
            original_combination: "".to_string(),
            original_rank: "".to_string(),
            authority_citation: "".to_string(),
            unchecked_authority_citation: "".to_string(),
            sourced_unverified_citations: "".to_string(),
            citation_group: "".to_string(),
            citation_kind: "".to_string(),
            authority_page: "".to_string(),
            authority_link: "".to_string(),
            authority_page_link: "".to_string(),
            unchecked_authority_page_link: "".to_string(),
            old_type_locality: "".to_string(),
            original_type_locality: "".to_string(),
            unchecked_type_locality: "".to_string(),
            emended_type_locality: "".to_string(),
            type_latitude: "".to_string(),
            type_longitude: "".to_string(),
            type_country: "".to_string(),
            type_subregion: "".to_string(),
            type_subregion2: "".to_string(),
            holotype: "".to_string(),
            type_kind: "".to_string(),
            type_specimen_link: "".to_string(),
            taxon_order: "".to_string(),
            family: "".to_string(),
            genus: "".to_string(),
            specific_epithet: "".to_string(),
            subspecific_epithet: "".to_string(),
            variant_of: "".to_string(),
            senior_homonym: "".to_string(),
            variant_name_citations: "".to_string(),
            name_usages: "".to_string(),
            comments: "".to_string(),
        }
    }

    pub fn from_csv(&self, csv_data: &str) -> Vec<SynonymData> {
        let data = self.clean_colnames(csv_data);
        let mut rdr = csv::Reader::from_reader(data.as_slice());
        let mut records = Vec::new();
        for (index, result) in rdr.deserialize().enumerate() {
            match result {
                Ok(record) => records.push(record),
                // Skip rather than keep an empty default record, which would
                // look like a real synonym with no data.
                Err(err) => eprintln!("Skipping synonym CSV record {}: {}", index + 1, err),
            }
        }
        records
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }

    // The mdd csv files contain a prefix with MDD_.
    // We need to remove before we can parse the files.
    // We will return is as byte string
    fn clean_colnames(&self, data: &str) -> Vec<u8> {
        let mut cleaned_data = Vec::new();
        let cols = data.lines().next().unwrap_or("").to_string();
        let mut colnames = cols
            .trim()
            .split(',')
            .map(|col| col.trim_start_matches("MDD_").to_string())
            .map(|col| col.to_case(convert_case::Case::Camel))
            .collect::<Vec<String>>()
            .join(",");
        colnames.push('\n');
        cleaned_data.push(colnames.as_bytes().to_vec());
        for line in data.lines().skip(1) {
            // Remove trailing whitespace and add newline
            let line = line.trim_end().to_string() + "\n";
            cleaned_data.push(line.as_bytes().to_vec());
        }
        cleaned_data.concat()
    }
}

/// Reads the authority-parentheses flag from CSV text ("0", "1", "?") or
/// from JSON (a number or null). Values other than a number become `None`
/// instead of failing the whole record.
fn deserialize_parentheses<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Flag {
        Number(u8),
        Text(String),
    }

    Ok(match Option::<Flag>::deserialize(deserializer)? {
        Some(Flag::Number(value)) => Some(value),
        Some(Flag::Text(text)) => text.trim().parse().ok(),
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_mdd_prefix() {
        let synonym_data = SynonymData::new();
        let input = "MDD_syn_id,hesp_id,species_id,species,root_name,author,year";
        let output = synonym_data.clean_colnames(input);
        assert_eq!(
            output,
            b"synId,hespId,speciesId,species,rootName,author,year\n".to_vec()
        );
    }

    /// The column header of the MDD v2.5 synonym CSV.
    const HEADER: &str = "MDD_syn_ID,MDD_species,MDD_root_name,MDD_author,MDD_year,MDD_authority_parentheses,MDD_nomenclature_status,MDD_validity,MDD_original_combination,MDD_normalized_original_combination,MDD_original_rank,MDD_authority_citation,MDD_unchecked_authority_citation,MDD_sourced_unverified_citations,MDD_citation_group,MDD_citation_kind,MDD_authority_page,MDD_authority_link,MDD_authority_page_link,MDD_unchecked_authority_page_link,MDD_old_type_locality,MDD_original_type_locality,MDD_unchecked_type_locality,MDD_emended_type_locality,MDD_type_latitude,MDD_type_longitude,MDD_type_country,MDD_type_subregion,MDD_type_subregion2,MDD_holotype,MDD_type_kind,MDD_type_specimen_link,MDD_order,MDD_family,MDD_genus,MDD_specificEpithet,MDD_subspecificEpithet,MDD_variant_of,MDD_senior_homonym,MDD_variant_name_citations,Hesp_id,MDD_species_id,MDD_name_usages,MDD_comments";

    /// A CSV with [HEADER] and one row per entry, filling columns not given
    /// with empty values.
    fn csv(rows: &[&[(&str, &str)]]) -> String {
        let columns: Vec<&str> = HEADER.split(',').collect();
        let mut csv = format!("{HEADER}\n");
        for row in rows {
            let values: Vec<&str> = columns
                .iter()
                .map(|col| {
                    row.iter()
                        .find(|(name, _)| name == col)
                        .map_or("", |(_, value)| value)
                })
                .collect();
            csv.push_str(&values.join(","));
            csv.push('\n');
        }
        csv
    }

    const PLATYPUS: &[(&str, &str)] = &[
        ("MDD_syn_ID", "100014411"),
        ("Hesp_id", "1"),
        ("MDD_species_id", "1000001"),
        ("MDD_root_name", "anatinus"),
        ("MDD_authority_parentheses", "0"),
    ];

    #[test]
    fn test_unknown_authority_parentheses_keeps_record() {
        let records = SynonymData::new().from_csv(&csv(&[
            &[
                ("MDD_syn_ID", "100056884"),
                ("Hesp_id", "55"),
                ("MDD_species_id", "1003013"),
                ("MDD_root_name", "kurdistanicus"),
                ("MDD_authority_parentheses", "?"),
            ],
            PLATYPUS,
        ]));
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].syn_id, 100056884);
        assert_eq!(records[0].species_id, Some(1003013));
        assert_eq!(records[0].root_name, "kurdistanicus");
        assert_eq!(records[0].authority_parentheses, None);
        assert_eq!(records[1].authority_parentheses, Some(0));
    }

    #[test]
    fn test_malformed_record_is_skipped() {
        let records = SynonymData::new().from_csv(&csv(&[
            &[("MDD_syn_ID", "not_a_number"), ("Hesp_id", "55")],
            PLATYPUS,
        ]));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].syn_id, 100014411);
    }

    #[test]
    fn test_authority_parentheses_json_round_trip() {
        let mut record = SynonymData::new();
        record.syn_id = 1;
        let json = record.to_json();
        assert!(json.contains("\"authorityParentheses\":null"));
        let back: SynonymData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.authority_parentheses, None);

        let numeric = json.replace(
            "\"authorityParentheses\":null",
            "\"authorityParentheses\":1",
        );
        let back: SynonymData = serde_json::from_str(&numeric).unwrap();
        assert_eq!(back.authority_parentheses, Some(1));
    }

    #[test]
    fn test_parsing_synonym_csv() {
        let path = "tests/data/syndata.csv";
        let data = std::fs::read_to_string(path).unwrap();
        let synonym_data = SynonymData::new();
        let records = synonym_data.from_csv(&data);
        assert!(!records.is_empty());
    }
}
