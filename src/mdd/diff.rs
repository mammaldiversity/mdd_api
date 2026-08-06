//! Methods and data structure for MDD diff files.

use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};

pub struct AllMddDiffs {
    pub data: Vec<DiffData>,
}

impl AllMddDiffs {
    pub fn from_file(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        let data: Vec<DiffData> = serde_json::from_reader(reader)?;
        Ok(Self { data })
    }

    pub fn get_diff(&self, version: &str) -> Option<&DiffData> {
        self.data.iter().find(|d| d.version == version)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffData {
    pub version: String,
    pub prev_version: String,
    pub taxonomy_changes: Vec<DiffTaxonomyChanges>,
    pub all_changes: Vec<DiffAllChanges>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffTaxonomyChanges {
    pub old_name: String,
    pub new_name: String,
    pub comments: String,
    pub catagory: String,
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffAllChanges {
    pub species_id: u32,
    pub old_name: String,
    pub new_name: String,
    pub category: String,
    pub column: String,
    pub old_value: String,
    pub new_value: String,
}
