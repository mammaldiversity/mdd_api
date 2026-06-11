//! MIL data preparation module implementation.
//!
//! Handles parsing MIL metadata (Excel or CSV), reading image folders
//! (including determining orientation), matching species against MDD records,
//! and writing the resulting prepared JSON files. Supports compressed MIL
//! release archives (.tar.gz and .zip).

use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use calamine::{Data, Reader, open_workbook_auto};
use flate2::read::GzDecoder;
use glob::glob;
use imagesize;
use serde::{Deserialize, Serialize};
use tar::Archive;
use tempdir::TempDir;
use zip::ZipArchive;

/// A record containing merged MIL and MDD data.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MilMetadata {
    pub mil_id: String,
    pub description: Option<String>,
    pub photographer: Option<String>,
    pub location: Option<String>,
    pub distribution: Option<String>,
    pub date_taken: Option<String>,
    pub is_uncertain_identification: bool,
    pub mdd_id: Option<u32>,
    pub orientation: Option<String>,
}

impl MilMetadata {
    pub fn new() -> Self {
        Self {
            mil_id: String::new(),
            description: None,
            photographer: None,
            location: None,
            distribution: None,
            date_taken: None,
            is_uncertain_identification: false,
            mdd_id: None,
            orientation: None,
        }
    }
}

pub struct MilParser<'a> {
    mil_file: &'a Path,
    mdd_file: &'a Path,
    mil_img_dir: Option<&'a Path>,
    output_path: &'a Path,
}

impl<'a> MilParser<'a> {
    pub fn new(
        mil_file: &'a Path,
        mdd_file: &'a Path,
        mil_img_dir: Option<&'a Path>,
        output_path: &'a Path,
    ) -> Self {
        Self {
            mil_file,
            mdd_file,
            mil_img_dir,
            output_path,
        }
    }

    /// Core function to prepare MIL metadata, matching species against MDD records, probing orientations,
    /// handling compressed inputs, and writing the final JSON.
    pub fn prepare_metadata(&self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_holder = TempDir::new("mil_prep")?;
        let (active_mil_file, active_img_dir) = self.resolve_input_paths(temp_holder.path())?;

        println!("Loading MIL metadata from: {:?}", active_mil_file);
        let mil_records = self.read_file_as_records(&active_mil_file)?;
        println!("Found {} MIL records.", mil_records.len());

        println!("Loading MDD metadata from: {:?}", self.mdd_file);
        let mdd_records = self.read_file_as_records(self.mdd_file)?;
        println!("Found {} MDD records.", mdd_records.len());

        let mdd_map = self.build_mdd_map(&mdd_records);

        println!(
            "Scanning image directory for dimensions: {:?}",
            active_img_dir
        );
        let mut image_map = HashMap::new();
        self.scan_images(&active_img_dir, &mut image_map);
        println!("Found {} image files with valid sizes.", image_map.len());

        let (merged_records, missing_image_ids) =
            self.process_mil_records(mil_records, &mdd_map, &image_map);

        self.write_output(merged_records, missing_image_ids)?;

        Ok(())
    }

    fn resolve_input_paths(
        &self,
        temp_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
        if let Some((metadata_path, img_dir_path)) =
            self.extract_archive_if_compressed(self.mil_file, temp_dir)?
        {
            return Ok((metadata_path, img_dir_path));
        }

        let img_dir = self.mil_img_dir.ok_or("MIL image directory is required when the MIL file is not a compressed archive.")?;
        Ok((self.mil_file.to_path_buf(), img_dir.to_path_buf()))
    }

    fn build_mdd_map(&self, mdd_records: &[HashMap<String, String>]) -> HashMap<String, u32> {
        let mut map = HashMap::new();
        for rec in mdd_records {
            if let (Some(id_str), Some(genus), Some(epithet)) =
                (rec.get("id"), rec.get("genus"), rec.get("specificEpithet"))
                && let Ok(id) = id_str.parse::<u32>() {
                    let name = format!("{}_{}", genus, epithet);
                    map.insert(name, id);
                }
        }
        map
    }

    fn process_mil_records(
        &self,
        mil_records: Vec<HashMap<String, String>>,
        mdd_map: &HashMap<String, u32>,
        image_map: &HashMap<String, String>,
    ) -> (Vec<MilMetadata>, Vec<String>) {
        let mut merged_records = Vec::new();
        let mut missing_image_ids = Vec::new();

        for rec in mil_records {
            let mil_id = match rec.get("milNo") {
                Some(id) => id.clone(),
                None => continue,
            };

            let genus = rec.get("genus").cloned().unwrap_or_default();
            let raw_epithet = rec.get("specificEpithet").cloned().unwrap_or_default();

            let is_uncertain = raw_epithet.ends_with('?');
            let clean_epithet = if is_uncertain {
                raw_epithet
                    .strip_suffix('?')
                    .unwrap_or(&raw_epithet)
                    .to_string()
            } else {
                raw_epithet.clone()
            };

            let scientific_name = format!("{}_{}", genus, clean_epithet);
            let mdd_id = mdd_map.get(&scientific_name).cloned();

            let description = rec.get("descriptionOfImage").cloned();
            let photographer = rec.get("photographer").cloned();
            let location = rec.get("locationWhereImageTaken").cloned();
            let distribution = rec.get("distributionOfSpecies").cloned();
            let date_taken = rec.get("dateImageTaken").cloned();

            let orientation = image_map.get(&mil_id).cloned();
            if orientation.is_none() {
                missing_image_ids.push(mil_id.clone());
            }

            merged_records.push(MilMetadata {
                mil_id,
                description,
                photographer,
                location,
                distribution,
                date_taken,
                is_uncertain_identification: is_uncertain,
                mdd_id,
                orientation,
            });
        }

        (merged_records, missing_image_ids)
    }

    fn write_output(
        &self,
        merged_records: Vec<MilMetadata>,
        mut missing_image_ids: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !missing_image_ids.is_empty() {
            missing_image_ids.sort();
            println!(
                "Missing {} images: {:?}",
                missing_image_ids.len(),
                missing_image_ids
            );

            let valid_records: Vec<MilMetadata> = merged_records
                .into_iter()
                .filter(|r| r.orientation.is_some())
                .collect();

            let json_data = serde_json::to_string(&valid_records)?;
            fs::write(self.output_path, json_data)?;
            println!(
                "Missing images, metadata exported to {:?} with missing image data",
                self.output_path
            );
        } else {
            let json_data = serde_json::to_string(&merged_records)?;
            fs::write(self.output_path, json_data)?;
            println!("Successfully exported metadata to {:?}", self.output_path);
        }

        Ok(())
    }

    fn is_supported_image_ext(ext: &str) -> bool {
        matches!(ext, "jpg" | "jpeg" | "png" | "webp" | "tif" | "tiff")
    }

    fn scan_images(&self, dir: &Path, map: &mut HashMap<String, String>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                self.scan_images(&p, map);
                continue;
            }

            if !p.is_file() {
                continue;
            }

            let ext = match p.extension() {
                Some(e) => e.to_string_lossy().to_lowercase(),
                None => continue,
            };

            if !Self::is_supported_image_ext(&ext) {
                continue;
            }

            let stem = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match imagesize::size(&p) {
                Ok(size) => {
                    let orientation = if size.width > size.height {
                        "landscape"
                    } else if size.width < size.height {
                        "portrait"
                    } else {
                        "square"
                    };
                    map.insert(stem, orientation.to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read dimensions of image {:?}: {:?}",
                        p, e
                    );
                }
            }
        }
    }

    /// Extracts a compressed ZIP or TAR.GZ archive, returning the found MIL metadata file and MIL image directory paths.
    fn extract_archive_if_compressed(
        &self,
        path: &Path,
        temp_dir: &Path,
    ) -> Result<Option<(PathBuf, PathBuf)>, Box<dyn std::error::Error>> {
        let path_str = path.to_string_lossy().to_lowercase();
        let is_zip = path_str.ends_with(".zip");
        let is_tar_gz = path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz");

        if !is_zip && !is_tar_gz {
            return Ok(None);
        }

        if is_zip {
            let file = File::open(path)?;
            let mut archive = ZipArchive::new(file)?;
            archive.extract(temp_dir)?;
        } else if is_tar_gz {
            let file = File::open(path)?;
            let tar = GzDecoder::new(file);
            let mut archive = Archive::new(tar);
            archive.unpack(temp_dir)?;
        }

        let mil_metadata = self.find_mil_metadata_file(temp_dir).ok_or_else(|| {
            "Could not find MIL metadata file in the extracted archive".to_string()
        })?;

        let mil_img_dir = self.find_mil_image_dir(temp_dir).ok_or_else(|| {
            "Could not find MIL image directory in the extracted archive".to_string()
        })?;

        Ok(Some((mil_metadata, mil_img_dir)))
    }

    /// Reads a CSV or Excel spreadsheet and extracts all rows as a Vec of HashMaps.
    /// Column headers are automatically cleaned and converted to camelCase.
    fn read_file_as_records(
        &self,
        path: &Path,
    ) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if ext == "xlsx" || ext == "xls" {
            let mut workbook = open_workbook_auto(path)?;
            let sheet_name = workbook
                .sheet_names()
                .first()
                .cloned()
                .ok_or("No sheets found in Excel file")?;
            let range = workbook.worksheet_range(&sheet_name)?;

            let mut rows = range.rows();
            let headers: Vec<String> = match rows.next() {
                Some(r) => r
                    .iter()
                    .map(|cell| self.to_camel(&self.data_to_string(cell)))
                    .collect(),
                None => return Ok(Vec::new()),
            };

            let mut records = Vec::new();
            for row in rows {
                let mut record = HashMap::new();
                for (i, cell) in row.iter().enumerate() {
                    if i < headers.len() {
                        let val = self.data_to_string(cell);
                        if !val.is_empty() {
                            record.insert(headers[i].clone(), val);
                        }
                    }
                }
                records.push(record);
            }
            Ok(records)
        } else {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_path(path)?;

            let headers: Vec<String> = rdr
                .headers()?
                .iter()
                .map(|h| self.to_camel(h).to_string())
                .collect();

            let mut records = Vec::new();
            for result in rdr.records() {
                let record_row = result?;
                let mut record = HashMap::new();
                for (i, val) in record_row.iter().enumerate() {
                    if i < headers.len() {
                        let val = val.trim().to_string();
                        if !val.is_empty() {
                            record.insert(headers[i].clone(), val);
                        }
                    }
                }
                records.push(record);
            }
            Ok(records)
        }
    }

    /// Helper function to clean a column name and convert it to camelCase.
    fn to_camel(&self, col: &str) -> String {
        let col = col.trim().replace('#', "No");

        let cleaned: String = col
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || c.is_whitespace())
            .collect();

        // Insert space between lowercase and uppercase
        let mut spaced = String::new();
        let chars: Vec<char> = cleaned.chars().collect();
        for i in 0..chars.len() {
            spaced.push(chars[i]);
            if i + 1 < chars.len()
                && chars[i].is_lowercase() && chars[i + 1].is_uppercase() {
                    spaced.push(' ');
                }
        }

        let words: Vec<&str> = spaced.split_whitespace().collect();
        if words.is_empty() {
            return String::new();
        }

        let mut result = words[0].to_lowercase();
        for word in &words[1..] {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                for c in chars {
                    result.push(c.to_ascii_lowercase());
                }
            }
        }
        result
    }

    /// Helper to convert calamine Cell Data to a String, formatting integers cleanly.
    fn data_to_string(&self, data: &Data) -> String {
        match data {
            Data::String(s) => s.trim().to_string(),
            Data::Float(f) => {
                if f.fract() == 0.0 {
                    (*f as i64).to_string()
                } else {
                    f.to_string()
                }
            }
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::Empty => String::new(),
            _ => String::new(),
        }
    }

    /// Helper to recursively search for a MIL metadata file (.xlsx, .xls, or .csv) inside a directory.
    fn find_mil_metadata_file(&self, dir: &Path) -> Option<PathBuf> {
        if let Some(file) = glob(&format!("{}/**/*.xlsx", dir.display()))
            .ok()
            .and_then(|mut entries| entries.next())
            .and_then(Result::ok)
        {
            return Some(file);
        }
        if let Some(file) = glob(&format!("{}/**/*.xls", dir.display()))
            .ok()
            .and_then(|mut entries| entries.next())
            .and_then(Result::ok)
        {
            return Some(file);
        }
        // Next, search for CSV files, excluding standard MDD species or synonym files
        if let Ok(entries) = glob(&format!("{}/**/*.csv", dir.display())) {
            for entry in entries.flatten() {
                let name = entry.file_name().unwrap_or_default().to_string_lossy();
                if !name.starts_with("MDD_v") && !name.starts_with("Species_Syn_v") {
                    return Some(entry);
                }
            }
        }
        None
    }

    /// Recursively scan for a folder containing images
    fn walk_image_dir(&self, path: &Path) -> Option<PathBuf> {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return None,
        };

        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(found) = self.walk_image_dir(&p) {
                    return Some(found);
                }
            } else if p.is_file()
                && let Some(ext) = p.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if Self::is_supported_image_ext(&ext_lower) {
                        return Some(path.to_path_buf());
                    }
                }
        }
        None
    }

    /// Helper to recursively search for the MIL image directory inside a directory.
    fn find_mil_image_dir(&self, dir: &Path) -> Option<PathBuf> {
        let webp_dir = dir.join("images-540px-webp");
        if webp_dir.is_dir() {
            return Some(webp_dir);
        }

        self.walk_image_dir(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_to_camel() {
        let parser = MilParser::new(Path::new(""), Path::new(""), None, Path::new(""));
        assert_eq!(parser.to_camel("MIL #"), "milNo");
        assert_eq!(parser.to_camel("Genus"), "genus");
        assert_eq!(parser.to_camel("Specific Epithet"), "specificEpithet");
        assert_eq!(
            parser.to_camel("Description of Image"),
            "descriptionOfImage"
        );
        assert_eq!(parser.to_camel("Photographer"), "photographer");
        assert_eq!(
            parser.to_camel("Location Where Image Taken"),
            "locationWhereImageTaken"
        );
        assert_eq!(parser.to_camel("Date Image Taken"), "dateImageTaken");
        assert_eq!(parser.to_camel("Original File Name"), "originalFileName");
        assert_eq!(parser.to_camel("specificEpithet"), "specificEpithet");
    }

    #[test]
    fn test_orientation_parsing() {
        let tmp = TempDir::new("test_images").unwrap();

        // Create 2x1 landscape PNG
        let landscape_png = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG Header
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR chunk signature
            0x00, 0x00, 0x00, 0x02, // width = 2
            0x00, 0x00, 0x00, 0x01, // height = 1
            0x08, 0x02, 0x00, 0x00, 0x00, // color options
            0xF1, 0xFF, 0x6F, 0xD3, // IHDR CRC
        ];
        let landscape_path = tmp.path().join("img1.png");
        fs::write(&landscape_path, &landscape_png).unwrap();

        // Create 1x2 portrait PNG
        let portrait_png = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG Header
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR chunk signature
            0x00, 0x00, 0x00, 0x01, // width = 1
            0x00, 0x00, 0x00, 0x02, // height = 2
            0x08, 0x02, 0x00, 0x00, 0x00, // color options
            0xCF, 0xD3, 0x7E, 0x22, // IHDR CRC
        ];
        let portrait_path = tmp.path().join("img2.png");
        fs::write(&portrait_path, &portrait_png).unwrap();

        // Probing sizes
        let size1 = imagesize::size(&landscape_path).unwrap();
        assert_eq!(size1.width, 2);
        assert_eq!(size1.height, 1);
        assert!(size1.width > size1.height); // landscape

        let size2 = imagesize::size(&portrait_path).unwrap();
        assert_eq!(size2.width, 1);
        assert_eq!(size2.height, 2);
        assert!(size2.width < size2.height); // portrait
    }

    #[test]
    fn test_prepare_metadata() {
        let tmp = TempDir::new("test_prep").unwrap();

        // 1. Create MIL CSV
        let mil_csv = "Order,Family,Common Name of Family,Genus,Specific Epithet,Common name of Species,Distribution of Species,MIL #,Description of Image,Date Image Taken,Photographer,Location Where Image Taken,Original File Name\n\
        Rodentia,Muridae,Mice,Mus,musculus,House Mouse,Cosmopolitan,MIL1001,A mouse,2026-01-01,John Doe,USA,file1.png\n\
        Rodentia,Muridae,Mice,Mus,caroli?,Ryukyu Mouse,Asia,MIL1002,Ryukyu mouse,2026-02-02,Jane Doe,Japan,file2.png\n\
        Rodentia,Muridae,Mice,Mus,cervicolor,Fawn-colored Mouse,Asia,MIL1003,Fawn mouse,2026-03-03,Bob Smith,Thailand,file3.png\n";

        let mil_path = tmp.path().join("mil.csv");
        fs::write(&mil_path, mil_csv).unwrap();

        // 2. Create MDD CSV
        let mdd_csv = "id,genus,specificEpithet\n\
        100,Mus,musculus\n\
        200,Mus,caroli\n";

        let mdd_path = tmp.path().join("mdd.csv");
        fs::write(&mdd_path, mdd_csv).unwrap();

        // 3. Create mock image directory
        let img_dir = tmp.path().join("images");
        fs::create_dir(&img_dir).unwrap();

        let png_data = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x02, // width = 2
            0x00, 0x00, 0x00, 0x01, // height = 1
            0x08, 0x02, 0x00, 0x00, 0x00, 0xF1, 0xFF, 0x6F, 0xD3,
        ];
        fs::write(img_dir.join("MIL1001.png"), &png_data).unwrap();

        let portrait_png = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, // width = 1
            0x00, 0x00, 0x00, 0x02, // height = 2
            0x08, 0x02, 0x00, 0x00, 0x00, 0xCF, 0xD3, 0x7E, 0x22,
        ];
        fs::write(img_dir.join("MIL1002.png"), &portrait_png).unwrap();

        // 4. Run prepare_metadata
        let output_json = tmp.path().join("output.json");
        let parser = MilParser::new(&mil_path, &mdd_path, Some(&img_dir), &output_json);
        parser.prepare_metadata().unwrap();

        // 5. Read output and verify
        let content = fs::read_to_string(&output_json).unwrap();
        let records: Vec<MilMetadata> = serde_json::from_str(&content).unwrap();

        assert_eq!(records.len(), 2);

        // MIL1001 checks
        let r1 = records.iter().find(|r| r.mil_id == "MIL1001").unwrap();
        assert_eq!(r1.description.as_deref(), Some("A mouse"));
        assert_eq!(r1.photographer.as_deref(), Some("John Doe"));
        assert_eq!(r1.location.as_deref(), Some("USA"));
        assert_eq!(r1.distribution.as_deref(), Some("Cosmopolitan"));
        assert_eq!(r1.date_taken.as_deref(), Some("2026-01-01"));
        assert_eq!(r1.is_uncertain_identification, false);
        assert_eq!(r1.mdd_id, Some(100));
        assert_eq!(r1.orientation.as_deref(), Some("landscape"));

        // MIL1002 checks
        let r2 = records.iter().find(|r| r.mil_id == "MIL1002").unwrap();
        assert_eq!(r2.is_uncertain_identification, true);
        assert_eq!(r2.mdd_id, Some(200));
        assert_eq!(r2.orientation.as_deref(), Some("portrait"));
    }
}
