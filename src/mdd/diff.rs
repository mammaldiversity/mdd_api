//! Methods and data structure for MDD diff files.

use serde::{Deserialize, Serialize};

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
