use std::{fs::File, io::{Read, Write, stderr, stdin}, process::exit};

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;
#[cfg(windows)]
use std::{fs::Metadata, os::windows::fs::MetadataExt};

use clap::Parser;
use content_inspector::inspect;
use determine_preset::{Cli, Determiner};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut buffer = String::new();

    match cli.input {
        Some(ref input) if input != "-" => {
            let mut file = match File::open(input) {
                    Ok(file) => file,
                    Err(err) => {
                        writeln!(stderr(), "Failed to open file for reading: {}\n", err).expect("Could not write to stderr");
                        exit(1)
                    }
                };
            // Read the file start to determine if it is mediainfo output or a video
            let md = file.metadata()?;
            let is_text = if md.size() < 1024 {
                file.read_to_string(&mut buffer).expect("Could not read from input file");
                inspect(buffer.as_bytes()).is_text()
            } else {
                let mut header = [0; 1024];
                file.read_exact(&mut header)?;
                inspect(&header).is_text()
            };

            if is_text {
                file.read_to_string(&mut buffer).expect("Could not read from input file");
            } else {
                // run mediainfo and populate the buffer with the output
                let output = std::process::Command::new("mediainfo")
                    .arg(input)
                    .output()
                    .expect("Failed to execute mediainfo");
                buffer = String::from_utf8_lossy(&output.stdout).to_string();
            }
        }
        _ => {
            stdin().read_to_string(&mut buffer).expect("Could not read from stdin");
        }

    }

    Determiner::new(cli).print_preset_from_str(&buffer);

    Ok(())
}
