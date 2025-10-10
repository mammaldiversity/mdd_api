//! Module to parse metadata information in the MDD release files.
use serde::{Deserialize, Serialize};

/// Metadata about the MDD release.
/// This metadata parse the version, release date, and other information
/// from TOML file.
/// # Example TOML format
/// ```toml
/// name = "MDD"
/// version = "2024.1"
/// release_date = "2024-06-01"
/// mdd_file = "mdd_2024_1.csv"
/// synonym_file = "synonyms_2024_1.csv"
/// remarks = "This is a sample release."
/// ```
///
/// Additional notes:
/// * `doi` and `remarks` are optional and will deserialize to `None` if absent.
/// * The parent struct (`ReleaseToml`) wraps this under the `[metadata]` table.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseToml {
    pub metadata: ReleaseMetadata,
}

impl ReleaseToml {
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let metadata: Self = toml::from_str(&content)?;
        Ok(metadata)
    }

    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("Failed to serialize to TOML")
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ReleaseMetadata {
    /// The name of the release.
    pub name: String,
    /// The version of the release, following semantic versioning.
    pub version: String,
    /// The date of the release.
    pub release_date: String,
    /// The file containing the MDD data for this release.
    pub mdd_file: String,
    /// The file containing synonyms for this release.
    pub synonym_file: String,
    /// Optional DOI for the release, if available.
    pub doi: Option<String>,
    /// Optional remarks or description for the release.
    pub remarks: Option<String>,
}

impl ReleaseMetadata {
    pub fn new(
        name: String,
        version: String,
        release_date: String,
        mdd_file: String,
        synonym_file: String,
        doi: Option<String>,
        remarks: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            release_date,
            mdd_file,
            synonym_file,
            doi,
            remarks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_metadata() {
        let toml_str = r#"
        [metadata]
        name = "MDD"
        version = "2024.1"
        release_date = "2024-06-01"
        mdd_file = "mdd_2024_1.csv"
        synonym_file = "synonyms_2024_1.csv"
        doi = "10.1234/mdd.2024.1"
        remarks = "This is a sample release."
        "#;

        let metadata = ReleaseToml::from_toml(toml_str).expect("Failed to parse TOML");
        assert_eq!(metadata.metadata.name, "MDD");
        assert_eq!(metadata.metadata.version, "2024.1");
        assert_eq!(metadata.metadata.release_date, "2024-06-01");
        assert_eq!(metadata.metadata.mdd_file, "mdd_2024_1.csv");
        assert_eq!(metadata.metadata.synonym_file, "synonyms_2024_1.csv");
        assert_eq!(
            metadata.metadata.remarks,
            Some("This is a sample release.".into())
        );
        assert_eq!(metadata.metadata.doi, Some("10.1234/mdd.2024.1".into()));
    }
}
