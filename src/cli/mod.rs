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
        args::Cli::Mil(args) => {
            if let Err(e) = crate::mil::prepare_metadata(
                &args.mil_file,
                &args.mdd_file,
                args.mil_img_dir.as_deref(),
                &args.output,
            ) {
                eprintln!("Error preparing MIL metadata: {:?}", e);
                std::process::exit(1);
            }
        }
        args::Cli::Prepare(args) => {
            // First run standard unpack to extract MDD files and produce standard JSONs
            println!("Unpacking MDD archive...");
            let unpack_args = crate::cli::args::UnpackArgs {
                input: crate::cli::args::CommonInput {
                    input: args.mdd_zip.clone(),
                    format: "zip".to_string(),
                },
                mdd_version: None,
                release_date: None,
                output: crate::cli::args::CommonOutput {
                    output: args.output.clone(),
                    output_format: crate::helper::types::OutputFormat::Json,
                    limit: None,
                    prefix: None,
                },
            };
            ZipParser::from_args(&unpack_args).parse();

            // Locate the unpacked MDD species CSV file in output/MDD/
            let pattern = format!("{}/MDD/MDD_v*.csv", args.output.display());
            let mdd_file = glob::glob(&pattern)
                .expect("Failed to parse glob pattern")
                .flatten()
                .next()
                .expect("Failed to locate unpacked MDD species CSV file");

            println!("Running MIL data preparation...");
            let output_json = args.output.join("mil_mdd.json");
            if let Err(e) = crate::mil::prepare_metadata(
                &args.mil_file,
                &mdd_file,
                args.mil_img_dir.as_deref(),
                &output_json,
            ) {
                eprintln!("Error preparing MIL metadata: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
