//! SHell Converter

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use clap::{self, App, Arg, ArgGroup};
use common::ExitDisplay;

fn main() {
    let app = App::new("shc")
        .version("0.1.0")
        .author("Icelk <main@icelk.dev>")
        .about("Converts shell files to other formats")
        .arg(
            Arg::with_name("FILES")
                .help("Files to convert to selected format")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("f")
                .help("Force file overrides")
                .short("f")
                .long("force"),
        )
        .arg(
            Arg::with_name("b")
                .help("Sets the output format to batch")
                .short("b")
                .long("batch"),
        )
        .group(ArgGroup::with_name("output_type").arg("b").required(true));

    let matches = app.get_matches();

    for file in matches.values_of("FILES").unwrap() {
        let file_path = Path::new(file);
        let new_file_path = {
            // Here, more types can be added.
            file_path.with_extension("bat")
        };
        match new_file_path.exists() {
            true if matches.is_present("f") => {
                // Continue with override
            }
            true => {
                // Ask for permission
                let allowed = common::confirm(
                    format!("Do you want to override the file {}?", file).as_str(),
                    Some(true),
                );
                if !allowed {
                    println!("Will not override file. Continuing to next file.");
                    continue;
                }
            }
            false => {
                // Continue; it does not override anything
            }
        }
        let mut input = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => "Failed to open specified file. Is the path correct?".print_exit(),
        };
        let mut output = match File::create(&new_file_path) {
            Ok(f) => f,
            Err(_) => "Failed to create or override output file.".print_exit(),
        };

        let mut text = Vec::with_capacity(4096);
        match input.read_to_end(&mut text) {
            Ok(_) => {}
            Err(_) => "Failed to read from input file.".print_exit(),
        }
        let text = match String::from_utf8(text) {
            Ok(s) => s,
            Err(_) => "Input config file contains invalid UTF-8.".print_exit(),
        };

        let output_text = shell2batch::convert(&text);

        match output.write_all(output_text.as_bytes()) {
            Ok(_) => {}
            Err(_) => "Failed to write output to file.".print_exit(),
        }
    }
}
