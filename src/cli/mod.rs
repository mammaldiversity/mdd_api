pub mod args;

use clap::Parser;

use crate::{
    filter::country::FilterByCountry,
    mil::prep::MilParser,
    parser::{diff::DiffParser, zip::ZipParser},
};

const DEFAULT_MIL_JSON: &str = "mil.json";

pub fn parse_args() {
    let cli = args::Cli::parse();
    match &cli {
        args::Cli::Unpack(args) => {
            ZipParser::from_args(args).parse();
        }
        args::Cli::Diff(args) => {
            let diff =
                DiffParser::parse_files(&args.input, &args.all_changes, args.release_date.clone())
                    .unwrap_or_else(|error| {
                        eprintln!("Error parsing diff files: {error}");
                        std::process::exit(1);
                    });
            let outputs =
                DiffParser::write_diff(diff, &args.output, args.append.as_deref(), args.plain_text)
                    .unwrap_or_else(|error| {
                        eprintln!("Error writing diff JSON: {error}");
                        std::process::exit(1);
                    });
            for output in outputs {
                println!("Diff output written to: {:?}", output);
            }
        }
        args::Cli::Filter(filter_cmd) => match filter_cmd {
            args::FilterSubcommand::ByCountry(args) => {
                FilterByCountry::from_args(args).filter();
            }
        },
        args::Cli::Mil(args) => {
            let parser = MilParser::new(
                &args.mil_file,
                &args.mdd_file,
                args.mil_img_dir.as_deref(),
                &args.output,
            );
            if let Err(e) = parser.prepare_metadata() {
                eprintln!("Error preparing MIL metadata: {:?}", e);
                std::process::exit(1);
            }
        }
        args::Cli::Prepare(args) => {
            // First run standard unpack to extract MDD files and produce standard JSONs
            println!("Unpacking MDD archive...");
            let unpack_args = args::UnpackArgs {
                input: args::CommonInput {
                    input: args.mdd_zip.clone(),
                    format: "zip".to_string(),
                },
                mdd_version: None,
                release_date: None,
                append_diff: None,
                plain_text: false,
                output: args::CommonOutput {
                    output: args.output.clone(),
                    output_format: args::OutputFormat::Json,
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
            let output_json = args.output.join(DEFAULT_MIL_JSON);
            let parser = MilParser::new(
                &args.mil_file,
                &mdd_file,
                args.mil_img_dir.as_deref(),
                &output_json,
            );
            if let Err(e) = parser.prepare_metadata() {
                eprintln!("Error preparing MIL metadata: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
