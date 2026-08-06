//! Methods and data structure for MDD diff files.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

/// A collection of release-to-release MDD diffs.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AllMddDiffs {
    pub data: Vec<DiffData>,
}

impl AllMddDiffs {
    pub fn from_file(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_path(Path::new(file))
    }

    /// Read a plain JSON or gzip-compressed JSON diff file.
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = fs::read(path)?;
        let json = if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(bytes.as_slice());
            let mut json = String::new();
            decoder.read_to_string(&mut json)?;
            json
        } else {
            String::from_utf8(bytes)?
        };
        let data = serde_json::from_str(&json)?;
        Ok(Self { data })
    }

    pub fn get_diff(&self, version: &str) -> Option<&DiffData> {
        self.data.iter().find(|d| d.version == version)
    }

    /// Replace a diff with the same release version, or append it if new.
    pub fn append_or_replace(&mut self, diff: DiffData) {
        if let Some(index) = self.data.iter().position(|d| d.version == diff.version) {
            self.data[index] = diff;
        } else {
            self.data.push(diff);
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }

    /// Write a diff collection as plain JSON or gzip JSON.
    pub fn write_json(&self, path: &Path, gzip: bool) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        if gzip {
            let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            encoder.write_all(self.to_json()?.as_bytes())?;
            encoder.finish()?;
        } else {
            let mut file = file;
            file.write_all(self.to_json()?.as_bytes())?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffData {
    pub version: String,
    pub prev_version: String,
    pub taxonomy_changes: Vec<DiffTaxonomyChanges>,
    pub all_changes: Vec<DiffAllChanges>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffTaxonomyChanges {
    pub old_name: String,
    pub new_name: String,
    pub comments: String,
    #[serde(rename = "category", alias = "catagory")]
    pub catagory: String,
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffAllChanges {
    pub species_id: u32,
    pub old_name: String,
    pub new_name: String,
    pub category: String,
    pub column: String,
    pub old_value: String,
    pub new_value: String,
}
