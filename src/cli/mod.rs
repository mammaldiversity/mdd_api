pub mod args;

use clap::Parser;

use crate::{filter::country::FilterByCountry, parser::zip::ZipParser};

pub fn parse_args() {
    let cli = args::Cli::parse();
    match &cli {
        args::Cli::Unpack(args) => {
            ZipParser::from_args(args).parse();
        }
        args::Cli::Filter(filter_cmd) => match filter_cmd {
            args::FilterSubcommand::ByCountry(args) => {
                FilterByCountry::from_args(args).filter();
            }
        },
    }
}
