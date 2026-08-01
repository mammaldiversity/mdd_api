//! Entry point for parsing and aggregating Mammal Diversity Database (MDD) data.
//!
//! This module exposes higher-level bundled data structures used by releases:
//! * `ReleasedMddData` – concise species records + attached synonyms + release metadata.
//! * `AllMddData` – full raw `MddData` rows plus all synonym rows.
//! * `MetaData` – aggregate counts (species, genera, families, orders, etc.).
//!
//! It also provides helpers to construct these from parser outputs or from
//! serialized JSON / gzipped JSON for distribution.

use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};
use species::SpeciesData;
use synonyms::SynonymData;

use crate::mdd::metadata::ReleaseMetadata;

pub mod country;
pub mod metadata;
pub mod species;
pub mod synonyms;
pub mod usa;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReleasedMddData {
    pub metadata: Metadata,
    pub data: Vec<SimpleMDD>,
    pub synonym_only: Vec<SynonymData>,
}

impl ReleasedMddData {
    pub fn new() -> Self {
        Self {
            metadata: Metadata::new(),
            data: Vec::new(),
            synonym_only: Vec::new(),
        }
    }

    pub fn from_gz_bytes(bytes: &[u8]) -> Self {
        let data = GzDecoder::new(bytes);
        serde_json::from_reader(data).expect("Failed to deserialize")
    }

    pub fn from_json(json_data: &str) -> Self {
        serde_json::from_str(json_data).expect("Failed to deserialize")
    }

    pub fn from_parser(
        mdd_data: Vec<SpeciesData>,
        synonym_data: Vec<SynonymData>,
        metadata: &ReleaseMetadata,
    ) -> Self {
        let synonym_only: Vec<SynonymData> = synonym_data
            .iter()
            .filter(|s| s.species_id.is_none())
            .cloned()
            .collect();

        let metadata_struct = Metadata::from_mdd(&mdd_data, &synonym_data, metadata);
        let mut simple_mdd = Vec::with_capacity(mdd_data.len());
        for mdd in mdd_data {
            let synonyms: Vec<SynonymData> = synonym_data
                .iter()
                .filter(|s| s.species_id == Some(mdd.id))
                .cloned()
                .collect();
            simple_mdd.push(SimpleMDD::new(mdd, synonyms));
        }

        Self {
            data: simple_mdd,
            synonym_only,
            metadata: metadata_struct,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }

    pub fn get_data(&self) -> (Vec<String>, Vec<String>) {
        let mdd = self.data.iter().map(|d| d.to_json()).collect();
        let synonyms = self.synonym_only.iter().map(|s| s.to_json()).collect();
        (mdd, synonyms)
    }

    pub fn get_version(&self) -> &str {
        &self.metadata.version
    }

    pub fn get_release_date(&self) -> &str {
        &self.metadata.release_date
    }

    pub fn get_remarks(&self) -> Option<&str> {
        self.metadata.get_remarks()
    }

    pub fn get_doi(&self) -> Option<&str> {
        self.metadata.get_doi()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleMDD {
    mdd_id: u32,
    species_data: SpeciesData,
    synonyms: Vec<SynonymData>,
}

impl SimpleMDD {
    fn new(species: SpeciesData, synonyms: Vec<SynonymData>) -> Self {
        Self {
            mdd_id: species.id,
            species_data: species,
            synonyms,
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub version: String,
    pub release_date: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub species_count: u32,
    pub synonym_count: u32,
    pub recently_extinct: u32,
    pub living: u32,
    pub domestic: u32,
    pub living_wild: u32,
    pub genus_count: u32,
    pub family_count: u32,
    pub order_count: u32,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            version: "".to_string(),
            release_date: "".to_string(),
            remarks: None,
            doi: None,
            species_count: 0,
            synonym_count: 0,
            recently_extinct: 0,
            living: 0,
            domestic: 0,
            living_wild: 0,
            genus_count: 0,
            family_count: 0,
            order_count: 0,
        }
    }

    pub fn from_mdd(
        data: &[SpeciesData],
        synonyms: &[SynonymData],
        metadata: &ReleaseMetadata,
    ) -> Self {
        let species_count = data.len() as u32;
        let synonym_count = synonyms.len() as u32;
        let recently_extinct = data.iter().filter(|d| d.extinct == 1).count() as u32;
        let living = species_count - recently_extinct;
        let domestic = data.iter().filter(|d| d.domestic == 1).count() as u32;
        let living_wild = living - domestic;
        let genus_count = data
            .iter()
            .map(|d| d.genus.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
        let family_count = data
            .iter()
            .map(|d| d.family.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
        let order_count = data
            .iter()
            .map(|d| d.taxon_order.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
        let version = metadata.version.clone();
        let release_date = metadata.release_date.clone();
        let remarks = metadata.remarks.clone();
        let doi = metadata.doi.clone();

        Self {
            version,
            release_date,
            remarks,
            doi,
            species_count,
            synonym_count,
            recently_extinct,
            living,
            domestic,
            living_wild,
            genus_count,
            family_count,
            order_count,
        }
    }

    pub fn get_remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    pub fn get_doi(&self) -> Option<&str> {
        self.doi.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AllMddData {
    pub version: String,
    pub release_date: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub data: Vec<SpeciesData>,
    pub synonyms: Vec<SynonymData>,
}

impl AllMddData {
    pub fn new() -> Self {
        Self {
            version: "".to_string(),
            release_date: "".to_string(),
            remarks: None,
            doi: None,
            data: Vec::new(),
            synonyms: Vec::new(),
        }
    }

    pub fn from_json(json_data: &str) -> Self {
        serde_json::from_str(json_data).expect("Failed to deserialize")
    }

    /// Create a new AllMddData object from a Gzipped byte array.
    pub fn from_gz_bytes(bytes: &[u8]) -> Self {
        let data = GzDecoder::new(bytes);
        serde_json::from_reader(data).expect("Failed to deserialize")
    }

    pub fn from_parser(
        mdd_data: Vec<SpeciesData>,
        synonym_data: Vec<SynonymData>,
        metadata: &ReleaseMetadata,
    ) -> Self {
        Self {
            version: metadata.version.clone(),
            release_date: metadata.release_date.clone(),
            remarks: metadata.remarks.clone(),
            doi: metadata.doi.clone(),
            data: mdd_data,
            synonyms: synonym_data,
        }
    }

    pub fn add_data(&mut self, data: SpeciesData) {
        self.data.push(data);
    }

    pub fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }

    pub fn set_release_date(&mut self, release_date: &str) {
        self.release_date = release_date.to_string();
    }

    pub fn set_remarks(&mut self, remarks: Option<String>) {
        self.remarks = remarks;
    }

    pub fn set_doi(&mut self, doi: Option<String>) {
        self.doi = doi;
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }

    pub fn get_data(&self) -> (Vec<String>, Vec<String>) {
        let mdd = self.data.iter().map(|d| d.to_json()).collect();
        let synonyms = self.synonyms.iter().map(|s| s.to_json()).collect();
        (mdd, synonyms)
    }

    pub fn get_mdd_data(&self) -> Vec<String> {
        self.data.iter().map(|d| d.to_json()).collect()
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_release_date(&self) -> &str {
        &self.release_date
    }

    pub fn get_remarks(&self) -> Option<&str> {
        self.remarks.as_deref()
    }

    pub fn get_doi(&self) -> Option<&str> {
        self.doi.as_deref()
    }
}
