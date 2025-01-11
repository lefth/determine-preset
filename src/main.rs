use std::{fs::File, io::{stderr, stdin, Read, Write}, process::exit};

use clap::Parser;
use determine_preset::{Cli, Determiner};

fn main() {
    let cli = Cli::parse();

    let mut buffer = String::new();

    if let Some(ref input) = cli.input {
        let mut file = match File::open(input) {
            Ok(file) => file,
            Err(err) => {
                writeln!(stderr(), "Failed to open file for reading: {}\n", err).expect("Could not write to stderr");
                exit(1)
            }
        };
        file.read_to_string(&mut buffer).expect("Could not read from input file");
    } else {
        stdin().read_to_string(&mut buffer).expect("Could not read from stdin");
    }

    Determiner::new(cli).print_preset_from_str(&buffer);
}
