//! Output helpers for serializing parsed MDD data into JSON or CSV.
//!
//! The writers in this module accept JSON strings containing either an
//! aggregated `AllMddData` structure (species + synonym bundle) or a vector of
//! `MddData` rows and persist them to disk. Optional CSV output mode performs a
//! simple transformation to replace the `taxonOrder` JSON field name back to
//! `order` for interoperability with tools expecting the original column name.
//!
//! Design notes:
//! * Conversion routines keep memory usage modest by streaming writes via
//!   `csv::Writer` where applicable.
//! * Gzipped JSON (`*.json.gz`) can be ingested and converted using
//!   `AllMddWriter::write_from_gz`.
//! * Both writers expose a `to_csv` flag; when false, raw JSON is written
//!   unchanged.

use std::{
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use flate2::bufread::MultiGzDecoder;

use crate::parser::{mdd::MddData, AllMddData};

const CSV_EXTENSION: &str = "csv";
const JSON_EXTENSION: &str = "json";

/// Common behavior for writer implementations.
trait Writer {
    fn write(&self, json_data: &str) -> Result<PathBuf, Box<dyn std::error::Error>>;

    fn create_output_path(&self) -> PathBuf;

    fn get_extension(&self) -> &str;
}

/// Write data structure for full MDD + synonym bundle (`AllMddData`).
pub struct AllMddWriter<'a> {
    pub output_dir: &'a Path,
    pub output_filename: &'a str,
    pub to_csv: bool,
}

impl Writer for AllMddWriter<'_> {
    fn write(&self, json_data: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        // Replace taxonOrder with order to avoid conflict with parser.
        let data = json_data.replace("taxonOrder", "order");
        let output_path = self.create_output_path();
        if self.to_csv {
            self.to_csv(&data, &output_path)?;
        } else {
            self.to_json(&data, &output_path)?;
        }
        Ok(output_path)
    }

    fn create_output_path(&self) -> PathBuf {
        let extension = self.get_extension();
        self.output_dir
            .join(&self.output_filename)
            .with_extension(extension)
    }

    fn get_extension(&self) -> &str {
        if self.to_csv {
            CSV_EXTENSION
        } else {
            JSON_EXTENSION
        }
    }
}

impl<'a> AllMddWriter<'a> {
    /// Create a new writer.
    pub fn new(output_dir: &'a Path, output_filename: &'a str, to_csv: bool) -> Self {
        AllMddWriter {
            output_dir,
            output_filename,
            to_csv,
        }
    }

    /// Read a gzipped JSON file (e.g., produced by distribution pipeline),
    /// decompress, and write it out in the configured format (JSON or CSV).
    pub fn write_from_gz(&self, json_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let file = fs::File::open(json_path)?;
        let inner = BufReader::new(file);
        let mut json_data = MultiGzDecoder::new(inner);
        let mut buf = String::new();
        json_data.read_to_string(&mut buf)?;
        self.write(&buf)?;
        Ok(self.create_output_path())
    }

    fn to_csv(
        &self,
        json_data: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(output_path)?;
        let records: AllMddData = serde_json::from_str(&json_data)?;
        let data = records.get_mdd_data();
        for record in data {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn to_json(
        &self,
        json_data: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(output_path, json_data)?;
        Ok(())
    }
}

/// Write data structure for MDD data only (`Vec<MddData>` JSON array).
pub struct MddWriter<'a> {
    pub output_dir: &'a Path,
    pub output_filename: &'a str,
    pub to_csv: bool,
}

impl Writer for MddWriter<'_> {
    fn write(&self, json_data: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_path = self.create_output_path();
        // Replace taxonOrder with order to avoid conflict with parser.
        let data = json_data.replace("taxonOrder", "order");
        if self.to_csv {
            self.to_csv(&data, &output_path)?;
        } else {
            self.to_json(&data, &output_path)?;
        }
        Ok(output_path)
    }

    fn create_output_path(&self) -> PathBuf {
        let extension = self.get_extension();
        self.output_dir
            .join(&self.output_filename)
            .with_extension(extension)
    }

    fn get_extension(&self) -> &str {
        if self.to_csv {
            CSV_EXTENSION
        } else {
            JSON_EXTENSION
        }
    }
}

impl<'a> MddWriter<'a> {
    /// Construct a new writer for species-only datasets.
    pub fn new(output_dir: &'a Path, output_filename: &'a str, to_csv: bool) -> Self {
        Self {
            output_dir,
            output_filename,
            to_csv,
        }
    }

    /// Persist provided JSON (array of `MddData`) to disk in JSON or CSV form.
    pub fn write(&self, json_data: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_path = self.create_output_path();
        if self.to_csv {
            self.to_csv(&json_data, &output_path)?;
        } else {
            self.to_json(&json_data, &output_path)?;
        }
        Ok(output_path)
    }

    fn to_csv(
        &self,
        json_data: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(output_path)?;
        let records: Vec<MddData> = serde_json::from_str(&json_data)?;
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn to_json(
        &self,
        json_data: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(output_path, json_data)?;
        Ok(())
    }

    fn create_output_path(&self) -> PathBuf {
        let extension = self.get_extension();
        self.output_dir
            .join(&self.output_filename)
            .with_extension(extension)
    }

    fn get_extension(&self) -> &str {
        if self.to_csv {
            CSV_EXTENSION
        } else {
            JSON_EXTENSION
        }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_write_json() {
        let json_mdd: &str = r#"[{"id":1,"phylosort":1,"subclass":"Theria"}]"#;
        let output_dir = TempDir::new("output").unwrap();
        let output_dir = env::current_dir().unwrap().join(output_dir.path());
        let filename = "output";
        let parser = AllMddWriter::new(&output_dir, filename, false);
        parser.write(json_mdd).unwrap();
        let json_result = output_dir.join(filename).with_extension(JSON_EXTENSION);
        assert_eq!(json_result.exists(), true);
    }

    // #[test]
    // fn test_write_csv() {
    //     let input_path = Path::new("../assets/data/data.json.gz");
    //     let output_dir = TempDir::new("output").unwrap();
    //     let output_dir = env::current_dir().unwrap().join(output_dir.path());
    //     let filename = "output";
    //     let parser = AllMddWriter::new(&output_dir, filename, true);
    //     parser.write_from_gz(input_path).unwrap();
    // }

    #[test]
    fn check_filename() {
        let output_dir = TempDir::new("output").unwrap();
        let output_dir = env::current_dir().unwrap().join(output_dir.path());
        let filename = "output";
        let parser = AllMddWriter::new(&output_dir, filename, false);
        let output_path = parser.create_output_path();
        assert_eq!(output_path, output_dir.join("output.json"));
    }
}
