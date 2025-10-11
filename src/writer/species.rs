use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{helper::types::OutputFormat, mdd::species::SpeciesData};

pub struct SpeciesWriter<'a> {
    pub output_dir: &'a Path,
    pub output_filename: &'a str,
    pub format: &'a OutputFormat,
}

impl<'a> SpeciesWriter<'a> {
    pub fn from_path(
        output_dir: &'a Path,
        output_filename: &'a str,
        format: &'a OutputFormat,
    ) -> Self {
        Self {
            output_dir,
            output_filename,
            format,
        }
    }

    pub fn write(
        &self,
        species_data: &[SpeciesData],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_path = self.create_output_path();
        match self.format {
            OutputFormat::Json => self.to_json(species_data, &output_path)?,
            OutputFormat::Csv => self.to_csv(species_data, &output_path)?,
            _ => panic!("Unsupported format for SpeciesWriter"),
        }
        Ok(output_path)
    }

    fn create_output_path(&self) -> PathBuf {
        self.output_dir
            .join(&self.output_filename)
            .with_extension(self.format.to_str())
    }

    fn to_csv(
        &self,
        species_data: &[SpeciesData],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(output_path)?;
        for record in species_data {
            wtr.serialize(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn to_json(
        &self,
        species_data: &[SpeciesData],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(species_data)?;
        std::fs::write(output_path, json_data)?;
        Ok(())
    }
}
