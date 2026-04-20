use clap::ValueEnum;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Csv,
    JsonGz,
    CsvGz,
}

impl OutputFormat {
    pub fn to_str(&self) -> &str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Csv => "csv",
            OutputFormat::JsonGz => "json.gz",
            OutputFormat::CsvGz => "csv.gz",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "json.gz" | "json_gz" | "gzip_json" | "gzip.json" => Ok(OutputFormat::JsonGz),
            "csv.gz" | "csv_gz" | "gzip_csv" | "gzip.csv" => Ok(OutputFormat::CsvGz),
            _ => Err(()),
        }
    }
}
