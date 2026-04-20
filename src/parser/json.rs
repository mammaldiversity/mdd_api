use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::DateTime;
use regex::Regex;

use crate::{
    helper::country_code::CountryRegionCode,
    mdd::{
        ReleasedMddData, country::CountryStats, species::SpeciesData, synonyms::SynonymData,
        usa::UsaStats,
    },
};

/// The default output file name for the JSON data.
const DEFAULT_OUTPUT_FNAME: &str = "data";
/// The default output file name for the country statistics.
const DEFAULT_COUNTRY_STATS_FNAME: &str = "country_stats";
/// The default output file name for the country region codes.
const DEFAULT_COUNTRY_REGION_FNAME: &str = "country_region_code";
/// The default output file name for the USA statistics.
const DEFAULT_USA_STATE_DATA_FNAME: &str = "usa_states";
/// The default JSON file extension.
const JSON_EXT: &str = "json";
/// The default gzip file extension.
const GZIP_EXT: &str = "json.gz";
/// The default prefix for the output file name.
const DEFAULT_PREFIX: &str = "mdd";

/// A parser for converting MDD data from a CSV file to a JSON file.
pub struct JsonParser<'a> {
    /// The path to the input MDD CSV file.
    pub input_path: &'a Path,
    /// The path to the input synonym CSV file.
    pub synonym_path: &'a Path,
    /// The path to the output directory.
    pub output_path: &'a Path,
    /// Whether to write the output as plain text.
    pub plain_text: bool,
    /// The version of the MDD data.
    pub mdd_version: Option<String>,
    /// The release date of the MDD data.
    pub release_date: Option<String>,
    /// The maximum number of records to parse.
    pub limit: Option<usize>,
    /// The prefix for the output file name.
    pub prefix: Option<&'a str>,
}

impl<'a> JsonParser<'a> {
    /// Creates a new `JsonParser` from the given paths.
    pub fn from_path(input_path: &'a Path, synonym_path: &'a Path, output_path: &'a Path) -> Self {
        Self {
            input_path,
            synonym_path,
            output_path,
            plain_text: true,
            mdd_version: None,
            release_date: None,
            limit: None,
            prefix: Some(DEFAULT_PREFIX),
        }
    }

    /// Updates the release data of the `JsonParser`.
    pub fn update_release_data(&mut self, date: &str, version: &str) {
        self.release_date = Some(date.to_string());
        self.mdd_version = Some(version.to_string());
    }

    // /// Creates a new `JsonParser` from the command-line arguments.
    // fn from_args(args: &'a JsonArgs) -> Self {
    //     Self {
    //         input_path: &args.input,
    //         synonym_path: &args.synonym,
    //         output_path: &args.output,
    //         plain_text: args.plain_text,
    //         mdd_version: args.mdd_version.clone(),
    //         release_date: args.release_date.clone(),
    //         limit: args.limit,
    //         prefix: args.prefix.as_deref(),
    //     }
    // }

    /// Parses the MDD data from the CSV file and converts it to a JSON file.
    pub fn parse_to_json(&self) {
        let mut mdd_data = self.parse_mdd_data();
        let mut synonym_data = self.parse_synonym_data();
        // State-level data for USA

        if synonym_data.is_empty() {
            println!("No synonym data found");
        }

        let country_stats = self.parse_country_stats(&mdd_data);
        let usa_data = self.parse_usa_data(&mdd_data, &country_stats);

        if let Some(limit) = self.limit {
            self.limit_mdd_data(&mut mdd_data, limit);
            self.limit_synonym_data(&mut synonym_data, limit);
        }

        let mdd_version = self.get_version();
        let release_date = self.get_release_date();

        println!(
            "\nUsing MDD version: {}, release date: {}\n",
            mdd_version, release_date
        );

        let released_json = self.merge_data(mdd_data, synonym_data);
        self.create_dir_if_not_exist();

        if self.plain_text {
            self.write_plain_text(&released_json);
            self.write_gzip(&released_json);
            println!("Output written to: {:?}", self.get_output_path(false));
        } else {
            self.write_gzip(&released_json);
        }

        self.write_country_stats(&country_stats);
        self.write_country_code();
        self.write_usa_stats(&usa_data);
    }

    fn parse_mdd_data(&self) -> Vec<SpeciesData> {
        let mdd_data = std::fs::read_to_string(self.input_path).expect("Failed to read MDD file");
        println!("Parsing MDD data from: {:?}", self.input_path);
        let parser = SpeciesData::new();
        let mdd_data = parser.from_csv(&mdd_data);
        println!("Found MDD data records: {}", mdd_data.len());
        mdd_data
    }

    fn parse_synonym_data(&self) -> Vec<SynonymData> {
        let syn_data =
            std::fs::read_to_string(self.synonym_path).expect("Failed to read synonym file");
        println!("Parsing synonym data from: {:?}", self.synonym_path);
        let synonyms = SynonymData::new();
        let synonym_data = synonyms.from_csv(&syn_data);
        println!("Found synonym data records: {}", synonym_data.len());
        synonym_data
    }

    fn parse_country_stats(&self, mdd_data: &[SpeciesData]) -> CountryStats {
        println!("Creating country mammal diversity statistics from MDD records");
        let mut country_stats = CountryStats::new();
        country_stats.parse_country_data(mdd_data);
        println!(
            "Total countries and regions: {}, Total domesticated species: {}, Total widespread species: {}",
            country_stats.total_countries,
            country_stats.domesticated.len(),
            country_stats.widespread.len()
        );
        country_stats
    }

    fn parse_usa_data(&self, mdd_data: &[SpeciesData], country_stats: &CountryStats) -> String {
        let species_list = country_stats.get_species_list_by_country("US");
        let usa_data: Vec<&SpeciesData> = mdd_data
            .iter()
            .filter(|species| species_list.contains(&species.id.to_string()))
            .collect();
        let mut usa_stats = UsaStats::new();
        usa_stats.from_country_data(&usa_data);
        println!("USA data parsed successfully");
        println!(
            "Total USA state/territory records: {}",
            usa_stats.total_states
        );
        usa_stats.to_json()
    }

    fn merge_data(&self, mdd_data: Vec<SpeciesData>, synonym_data: Vec<SynonymData>) -> String {
        let all_data = ReleasedMddData::from_parser(
            mdd_data,
            synonym_data,
            &self.get_version(),
            &self.get_release_date(),
        );
        println!("MDD {} data parsed successfully", self.get_version());
        println!("Total MDD records: {}", all_data.data.len());
        println!(
            "Total synonym only records: {}",
            all_data.synonym_only.len()
        );
        all_data.to_json()
    }

    /// Returns the version of the MDD data.
    ///
    /// We use the version if specified.
    /// Otherwise, we will infer from the file name.
    /// MDD species file_stem example: MDD_v2.2_6815species.
    /// In this case, the version is 2.2.
    fn get_version(&self) -> String {
        match &self.mdd_version {
            Some(version) => version.clone(),
            None => {
                let file_stem = self
                    .input_path
                    .file_stem()
                    .expect("Invalid file name")
                    .to_str()
                    .expect("Failed to convert OsStr to str");
                // Use regex to capture the version number
                let re =
                    Regex::new(r"MDD_v(\d+\.\d+)").expect("Failed to compile MDD version regex");
                if let Some(caps) = re.captures(file_stem) {
                    caps.get(1)
                        .map_or("unknown".to_string(), |m| m.as_str().to_string())
                } else {
                    "unknown".to_string()
                }
            }
        }
    }

    /// Returns the release date of the MDD data.
    ///
    /// We infer release date from the metadata if not specified.
    fn get_release_date(&self) -> String {
        match &self.release_date {
            Some(date) => date.clone(),
            None => {
                let file_meta =
                    fs::metadata(self.input_path).expect("Failed to read file metadata");
                let modified_time = file_meta
                    .created()
                    .expect("Failed to get file modified time");
                let date = DateTime::<chrono::Local>::from(modified_time);
                date.format("%B %e, %Y").to_string()
            }
        }
    }

    /// Limits the number of MDD data records.
    fn limit_mdd_data(&self, data: &mut Vec<SpeciesData>, limit: usize) {
        data.truncate(limit);
    }

    /// Limits the number of synonym data records.
    fn limit_synonym_data(&self, data: &mut Vec<SynonymData>, limit: usize) {
        data.truncate(limit);
    }

    /// Writes the given data to a plain text file.
    fn write_plain_text(&self, data: &str) {
        let output = self.get_output_path(false);
        std::fs::write(output, data).expect("Unable to write file");
    }

    fn write_country_stats(&self, country_stats: &CountryStats) {
        // Write country statistics to JSON file
        country_stats.write_to_json_file(
            &self
                .output_path
                .join(DEFAULT_COUNTRY_STATS_FNAME)
                .with_extension(JSON_EXT),
        );
    }

    fn write_country_code(&self) {
        let country_region_code = CountryRegionCode::new();
        country_region_code.write_to_file(
            self.output_path
                .join(DEFAULT_COUNTRY_REGION_FNAME)
                .with_extension(JSON_EXT),
        );
    }

    fn write_usa_stats(&self, usa_data: &str) {
        let output = self
            .output_path
            .join(DEFAULT_USA_STATE_DATA_FNAME)
            .with_extension(JSON_EXT);
        std::fs::write(output, usa_data).expect("Unable to write file");
    }

    /// Writes the given data to a gzip file.
    fn write_gzip(&self, data: &str) {
        let output = self.get_output_path(true);
        let file = std::fs::File::create(output).expect("Unable to create file");
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, data.as_bytes()).expect("Unable to write file");
    }

    /// Returns the output path for the JSON file.
    fn get_output_path(&self, is_gunzip: bool) -> PathBuf {
        let fname = match self.prefix {
            Some(prefix) => prefix,
            None => DEFAULT_OUTPUT_FNAME,
        };
        let output = self.output_path.join(fname);
        if is_gunzip {
            output.with_extension(GZIP_EXT)
        } else {
            output.with_extension(JSON_EXT)
        }
    }

    fn create_dir_if_not_exist(&self) {
        fs::create_dir_all(self.output_path).unwrap_or_else(|_| {
            panic!("Failed to create output directory: {:?}", self.output_path)
        });
    }
}
