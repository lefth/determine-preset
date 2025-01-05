use std::{env::args, io::{stderr, stdin, Read, Write}, process::exit};

use determine_preset::print_preset_from_str;

fn main() {
    let input = &mut String::new();
    let args: Vec<_> = args().collect();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        stderr().write_all(format!("Usage: {} reads ffmpeg encoding data in the arguments, or on STDIN if the first argument is '-'. If found, the preset is printed.\n", args[0]).as_bytes()).expect("Failed to write to stderr");
        exit(0);
    } else if args.iter().skip(1).next().map(|s| s == "-").unwrap_or_default() {
        // Input string (stdin) is key=value pairs, space delimited.
        // The format is somewhat flexible but not tested.
        stdin().read_to_string(input).expect("Failed to read from stdin");
    } else {
        // Read arguments into strings, and key=value pairs will be found
        // within the strings.
        let _ = std::mem::replace(input, args[1..].join(" "));
    }

    print_preset_from_str(input);
}
