use std::path::Path;

use crate::{
    cli::args::FilterByCountryArgs,
    helper::types::OutputFormat,
    mdd::{
        country::{CountryData, CountryStats},
        species::SpeciesData,
    },
    parser::zip::MddArchive,
    writer::species::SpeciesWriter,
};

const DEFAULT_PREFIX: &str = "mdd_filtered_by_countries";

pub struct FilterByCountry<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub prefix: Option<&'a str>,
    pub output_format: &'a OutputFormat,
    pub country_codes: &'a [String],
}

impl<'a> FilterByCountry<'a> {
    pub fn new(
        input_path: &'a Path,
        output_path: &'a Path,
        output_format: &'a OutputFormat,
        country_codes: &'a [String],
    ) -> Self {
        Self {
            input_path,
            output_path,
            output_format,
            country_codes,
            prefix: None,
        }
    }

    pub fn from_args(args: &'a FilterByCountryArgs) -> Self {
        Self {
            input_path: &args.input.input,
            output_path: &args.output.output,
            output_format: &args.output.output_format,
            country_codes: &args.country_codes,
            prefix: args.output.prefix.as_deref(),
        }
    }

    pub fn filter(&self) {
        println!("Extracting archive from: {:?}", self.input_path);

        let mut species_data = self.parse_species_data(self.input_path);
        let country_data = self.get_country_species_list(&species_data);
        self.filter_species_data_by_ids(&mut species_data, &country_data);
        self.write_filtered_data(&species_data);
    }

    fn parse_species_data(&self, path: &Path) -> Vec<SpeciesData> {
        let mut mdd_data = MddArchive::new();
        mdd_data.get_species_data(path);
        mdd_data.species_data

        // // We show the path if it fails.
        // println!("Parsing species data from: {:?}", path);
        // let mdd_data = std::fs::read_to_string(path).expect("Failed to read MDD file");
        // let parser = SpeciesData::new();
        // let mdd_data = parser.from_csv(&mdd_data);
        // println!("Total MDD records: {}", mdd_data.len());
        // return mdd_data;
    }

    fn get_country_species_list(&self, data: &[SpeciesData]) -> Vec<String> {
        let mut country_data = CountryStats::new();
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
        let prefix = self.get_output_prefix();
        let writer = SpeciesWriter::from_path(self.output_path, &prefix, self.output_format);
        let output_file = writer
            .write(data)
            .expect("Failed to write filtered species data");
        println!("Filtered species data written to: {:?}", output_file);
    }

    fn get_output_prefix(&self) -> String {
        match self.prefix {
            Some(p) => p.to_string(),
            None => DEFAULT_PREFIX.to_string(),
        }
    }
}
