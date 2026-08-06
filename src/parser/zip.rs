use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::mdd::species::SpeciesData;
use crate::{cli::args::UnpackArgs, mdd::metadata::ReleaseToml, parser::json::JsonParser};

/// A parser for extracting MDD data from a zip file.
pub struct ZipParser<'a> {
    /// The path to the input zip file.
    input_path: &'a Path,
    /// The path to the output directory.
    output_path: &'a Path,
}

impl<'a> ZipParser<'a> {
    /// Creates a new `ZipParser` from the command-line arguments.
    pub fn from_args(args: &'a UnpackArgs) -> Self {
        Self {
            input_path: &args.input.input,
            output_path: &args.output.output,
        }
    }

    /// Parses the MDD data from the zip file and converts it to a JSON file.
    pub fn parse(&self) {
        self.extract_zip_file();
        // We will find the MDD file prefix with MDD_v in the file name.
        // and synonym file with prefix "Species_Syn_v"
        println!("Extracting files...");
        let glob_files = glob::glob(&format!(
            "{}/MDD/*.csv",
            self.output_path
                .to_str()
                .expect("Failed to convert Path to str")
        ));
        let files = match glob_files {
            Ok(files) => files.filter_map(Result::ok).collect::<Vec<PathBuf>>(),
            Err(e) => panic!("Failed to find MDD files with pattern: {}", e),
        };
        println!("Found {} MDD files.", files.len());
        let meta_path = self.find_release_toml_file(self.output_path);
        let meta = if let Some(meta_path) = meta_path {
            let metadata =
                ReleaseToml::from_file(&meta_path).expect("Failed to read release.toml file");
            println!("Found release.toml file.\n");
            Some(metadata)
        } else {
            println!("No release.toml file found. Using default metadata.\n");
            None
        };

        let mdd_file = self.find_mdd_file(&files);
        println!("Found MDD file: {:?}", mdd_file);
        let syn_file = self.find_synonym_file(&files);
        println!("Found synonym file: {:?}", syn_file);
        if mdd_file.is_none() || syn_file.is_none() {
            panic!("MDD or synonym file not found in the zip archive. Please check the zip file.");
        }

        let mut json_parser = JsonParser::from_path(
            mdd_file.as_ref().expect("MDD file not found"),
            syn_file.as_ref().expect("Synonym file not found"),
            self.output_path,
        );
        if let Some(meta) = meta {
            json_parser.set_metadata(meta.metadata);
        }
        json_parser.parse_to_json();
    }

    /// Extracts the contents of the zip file to the output directory.
    fn extract_zip_file(&self) {
        let zip = std::fs::File::open(self.input_path).expect("Failed to open zip file");
        let mut archive = zip::ZipArchive::new(zip).expect("Failed to read zip file");
        // We extract the file for now to keep it simple.
        archive
            .extract(self.output_path)
            .expect("Failed to extract zip file");
    }

    /// Finds the release.toml file in the extracted files.
    fn find_release_toml_file(&self, output_path: &Path) -> Option<PathBuf> {
        if let Some(file) = glob::glob(&format!("{}/**/release.toml", output_path.display()))
            .expect("Failed to find release.toml file")
            .flatten()
            .next()
        {
            return Some(file);
        }
        None
    }

    /// Finds the MDD file in the extracted files.
    fn find_mdd_file(&self, files: &[PathBuf]) -> Option<PathBuf> {
        for file in files {
            if file
                .file_name()
                .expect("Failed to get file name")
                .to_str()
                .expect("Failed to convert OsStr to str")
                .starts_with("MDD_v")
            {
                return Some(file.to_path_buf());
            }
        }
        None
    }

    /// Finds the synonym file in the extracted files.
    fn find_synonym_file(&self, files: &[PathBuf]) -> Option<PathBuf> {
        for file in files {
            let file_name = file
                .file_name()
                .expect("Failed to get file name")
                .to_str()
                .expect("Failed to convert OsStr to str");
            if file_name.starts_with("Species_Syn_") {
                return Some(file.to_path_buf());
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct MddArchive {
    pub release_meta: Option<String>,
    pub species_data: Vec<SpeciesData>,
    pub synonym_data: Vec<SpeciesData>,
}

impl MddArchive {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_species_data(&mut self, zip_path: &Path) {
        let mut archive = self.open_file(zip_path);
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to get file by index");
            let file_name = file.name().to_string();
            if file_name.contains("MDD_v") && file_name.ends_with(".csv") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Failed to read file contents");
                let parser = SpeciesData::new();
                self.species_data = parser.from_csv(&contents);
                break;
            }
        }
    }

    pub fn open_file(&self, zip_path: &Path) -> ZipArchive<File> {
        let file = File::open(zip_path).expect("Failed to open zip file");
        zip::ZipArchive::new(file).expect("Failed to read zip file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_synonym_file() {
        let parser = ZipParser {
            input_path: Path::new("dummy.zip"),
            output_path: Path::new("dummy_out"),
        };

        let files1 = vec![PathBuf::from("/tmp/MDD/Species_Syn_v2.2.csv")];
        let files2 = vec![PathBuf::from("/tmp/MDD/Species_Syn_Current_v2.5.csv")];

        assert_eq!(
            parser.find_synonym_file(&files1),
            Some(PathBuf::from("/tmp/MDD/Species_Syn_v2.2.csv"))
        );

        assert_eq!(
            parser.find_synonym_file(&files2),
            Some(PathBuf::from("/tmp/MDD/Species_Syn_Current_v2.5.csv"))
        );
    }
}
