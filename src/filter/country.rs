use std::path::Path;

use crate::{cli::args::FilterByCountryArgs, parser::zip::MddArchive};

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
        println!(
            "MDD species path: {:?}",
            data.species_file.unwrap_or_default()
        );
        println!(
            "MDD synonym path: {:?}",
            data.synonym_file.unwrap_or_default()
        );
        println!("Output path: {:?}", self.output_path);
        println!("Output format: {:?}", self.output_format);
    }
}
