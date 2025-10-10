use std::{fs, path::Path};

use crate::{
    cli::args::FilterByCountryArgs,
    mdd::{
        country::{CountryData, CountryMDDStats},
        species::SpeciesData,
    },
    parser::zip::MddArchive,
};

pub struct FilterByCountry<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub output_format: &'a str,
    pub country_codes: &'a [String],
}

impl<'a> FilterByCountry<'a> {
    pub fn new(
        input_path: &'a Path,
        output_path: &'a Path,
        output_format: &'a str,
        country_codes: &'a [String],
    ) -> Self {
        Self {
            input_path,
            output_path,
            output_format,
            country_codes,
        }
    }

    pub fn from_args(args: &'a FilterByCountryArgs) -> Self {
        Self {
            input_path: &args.input.input,
            output_path: &args.output.output,
            output_format: &args.output.output_format,
            country_codes: &args.country_codes,
        }
    }

    pub fn filter(&self) {
        println!("Extracting archive from: {:?}", self.input_path);
        let data = MddArchive::from_path(self.input_path, self.output_path);
        match data.species_file {
            Some(path) => {
                let mut species_data = self.parse_species_data(&path);
                let country_data = self.get_country_species_list(&species_data);
                self.filter_species_data_by_ids(&mut species_data, &country_data);
                self.write_filtered_data(&species_data);
            }
            None => {
                panic!("No species data file found in the archive.");
            }
        }
    }

    fn parse_species_data(&self, path: &Path) -> Vec<SpeciesData> {
        let mdd_data = std::fs::read_to_string(path).expect("Failed to read MDD file");
        let parser = SpeciesData::new();
        let mdd_data = parser.from_csv(&mdd_data);
        println!("Total MDD records: {}", mdd_data.len());
        return mdd_data;
    }

    fn get_country_species_list(&self, data: &[SpeciesData]) -> Vec<String> {
        let mut country_data = CountryMDDStats::new();
        country_data.parse_country_data(data);
        country_data
            .country_data
            .retain(|code, _| self.country_codes.contains(code));
        // We only care about list of mdd ids for the countries.
        // This list is stored as in the CountryData struct as species_list.
        // We will collect all for all the countries into a single list.
        let mut filtered_ids: Vec<String> = Vec::new();
        country_data
            .country_data
            .values()
            .for_each(|country: &CountryData| {
                filtered_ids.extend(country.species_list.iter().cloned());
            });
        filtered_ids.sort();
        filtered_ids.dedup();
        println!(
            "Filtered species records for countries {:?}: {}",
            self.country_codes,
            filtered_ids.len()
        );
        filtered_ids
    }

    fn filter_species_data_by_ids(&self, data: &mut Vec<SpeciesData>, filtered_ids: &[String]) {
        data.retain(|species| filtered_ids.contains(&species.id.to_string()));
    }

    fn write_filtered_data(&self, data: &[SpeciesData]) {
        fs::create_dir_all(self.output_path).unwrap_or_else(|_| {
            panic!("Failed to create output directory: {:?}", self.output_path)
        });
        let output_file = self.output_path.join(format!(
            "mdd_filtered_by_countries.{}",
            if self.output_format == "csv" {
                "csv"
            } else {
                "json"
            }
        ));
        if self.output_format == "csv" {
            let writer = SpeciesData::new();
            writer
                .to_csv(&output_file)
                .expect("Failed to write filtered data to CSV");
            println!("Filtered data written to: {:?}", output_file);
        } else {
            let json_data = serde_json::to_string(data).expect("Failed to serialize filtered data");
            std::fs::write(&output_file, json_data).expect("Failed to write filtered data to JSON");
            println!("Filtered data written to: {:?}", output_file);
        }
    }
}
