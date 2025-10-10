pub mod args;

use clap::Parser;

pub fn parse_args() {
    let cli = args::Cli::parse();
    match &cli {
        args::Cli::Unpack(args) => {
            if args.format != "zip" {
                eprintln!("Currently, only 'zip' format is supported.");
                std::process::exit(1);
            }
        }
    }
}
