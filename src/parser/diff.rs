//! Parsers and writers for MDD release diff CSV files.

use std::error::Error;
use std::path::{Path, PathBuf};

use csv::StringRecord;
use regex::Regex;

use crate::mdd::diff::{AllMddDiffs, DiffAllChanges, DiffData, DiffTaxonomyChanges};

const DEFAULT_DIFF_OUTPUT: &str = "diffs.json.gz";

pub struct DiffParser;

impl DiffParser {
    pub fn parse_files(
        taxonomy_path: &Path,
        all_changes_path: &Path,
        release_date: Option<String>,
    ) -> Result<DiffData, Box<dyn Error>> {
        let (prev_version, version) = Self::parse_version_pair(taxonomy_path)?;
        let all_versions = Self::parse_version_pair(all_changes_path)?;
        if (prev_version.clone(), version.clone()) != all_versions {
            return Err(format!(
                "Diff files describe different version transitions: {:?} and {:?}",
                taxonomy_path.file_name(),
                all_changes_path.file_name()
            )
            .into());
        }

        Ok(DiffData {
            version,
            prev_version,
            taxonomy_changes: Self::parse_taxonomy_changes(taxonomy_path)?,
            all_changes: Self::parse_all_changes(all_changes_path)?,
            release_date,
        })
    }

    pub fn write_diff(
        diff: DiffData,
        output_path: &Path,
        append_path: Option<&Path>,
        plain_text: bool,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut diffs = match append_path {
            Some(path) => AllMddDiffs::from_path(path)?,
            None => AllMddDiffs::default(),
        };
        diffs.append_or_replace(diff);

        let compressed_path = compressed_output_path(output_path);
        diffs.write_json(&compressed_path, true)?;
        let mut outputs = vec![compressed_path];

        if plain_text {
            let plain_path = plain_output_path(output_path);
            diffs.write_json(&plain_path, false)?;
            outputs.push(plain_path);
        }
        Ok(outputs)
    }

    pub fn default_output_path(output_dir: &Path) -> PathBuf {
        output_dir.join(DEFAULT_DIFF_OUTPUT)
    }

    /// Find a matching taxonomy/all-changes pair in extracted archive files.
    pub fn find_pair(files: &[PathBuf]) -> Option<(PathBuf, PathBuf)> {
        for taxonomy_path in files {
            if !Self::is_taxonomy_file(taxonomy_path) {
                continue;
            }
            let Ok(taxonomy_versions) = Self::parse_version_pair(taxonomy_path) else {
                continue;
            };
            for all_changes_path in files {
                if taxonomy_path == all_changes_path || !Self::is_all_changes_file(all_changes_path)
                {
                    continue;
                }
                if Self::parse_version_pair(all_changes_path).ok()
                    == Some(taxonomy_versions.clone())
                {
                    return Some((taxonomy_path.clone(), all_changes_path.clone()));
                }
            }
        }
        None
    }

    pub fn is_taxonomy_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| taxonomy_regex().is_match(name))
            .unwrap_or(false)
    }

    pub fn is_all_changes_file(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| all_changes_regex().is_match(name))
            .unwrap_or(false)
    }

    fn parse_version_pair(path: &Path) -> Result<(String, String), Box<dyn Error>> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Invalid diff filename: {}", path.display()))?;
        let captures = taxonomy_regex()
            .captures(file_name)
            .or_else(|| all_changes_regex().captures(file_name))
            .ok_or_else(|| format!("Unable to infer diff versions from {}", file_name))?;
        Ok((
            captures["prev"].to_string(),
            captures["version"].to_string(),
        ))
    }

    fn parse_taxonomy_changes(path: &Path) -> Result<Vec<DiffTaxonomyChanges>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut records = Vec::new();
        for result in reader.records() {
            let record = result?;
            records.push(DiffTaxonomyChanges {
                old_name: field(&record, 0, path)?,
                new_name: field(&record, 1, path)?,
                comments: field(&record, 2, path)?,
                catagory: field(&record, 3, path)?,
                reference: field(&record, 4, path)?,
            });
        }
        Ok(records)
    }

    fn parse_all_changes(path: &Path) -> Result<Vec<DiffAllChanges>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut records = Vec::new();
        for result in reader.records() {
            let record = result?;
            records.push(DiffAllChanges {
                species_id: field(&record, 0, path)?.parse()?,
                old_name: field(&record, 1, path)?,
                new_name: field(&record, 2, path)?,
                category: field(&record, 3, path)?,
                column: field(&record, 4, path)?,
                old_value: field(&record, 5, path)?,
                new_value: field(&record, 6, path)?,
            });
        }
        Ok(records)
    }
}

fn taxonomy_regex() -> Regex {
    Regex::new(r"^Diff_v(?P<prev>\d+(?:\.\d+)*)-v?(?P<version>\d+(?:\.\d+)*)\.csv$")
        .expect("valid taxonomy diff regex")
}

fn all_changes_regex() -> Regex {
    Regex::new(r"^Diff-AllChanges_v(?P<prev>\d+(?:\.\d+)*)-v?(?P<version>\d+(?:\.\d+)*)\.csv$")
        .expect("valid all-changes diff regex")
}

fn field(record: &StringRecord, index: usize, path: &Path) -> Result<String, Box<dyn Error>> {
    record
        .get(index)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Missing column {} in {}", index, path.display()).into())
}

fn normalized_stem(path: &Path) -> String {
    let value = path.to_string_lossy();
    value
        .strip_suffix(".json.gz")
        .or_else(|| value.strip_suffix(".json"))
        .unwrap_or(&value)
        .to_string()
}

fn compressed_output_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.json.gz", normalized_stem(path)))
}

fn plain_output_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.json", normalized_stem(path)))
}

pub fn collect_csv_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let pattern = format!("{}/**/*.csv", root.display());
    Ok(glob::glob(&pattern)?.filter_map(Result::ok).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_diff_filename_shapes() {
        assert_eq!(
            DiffParser::parse_version_pair(Path::new("Diff_v2.2-v2.3.csv")).unwrap(),
            ("2.2".to_string(), "2.3".to_string())
        );
        assert_eq!(
            DiffParser::parse_version_pair(Path::new("Diff-AllChanges_v2.2-2.3.csv")).unwrap(),
            ("2.2".to_string(), "2.3".to_string())
        );
    }

    #[test]
    fn normalizes_json_output_suffixes() {
        assert_eq!(
            compressed_output_path(Path::new("out/diffs.json")).to_string_lossy(),
            "out/diffs.json.gz"
        );
        assert_eq!(
            plain_output_path(Path::new("out/diffs.json.gz")).to_string_lossy(),
            "out/diffs.json"
        );
    }
}
