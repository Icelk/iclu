use std::path::Path;

use clap::{self, App, Arg};

mod lib;
use lib::Comment;

fn main() {
    let app = App::new("corpl")
        .version("0.1.0")
        .author("Icelk <main@icelk.dev>")
        .about("Changes exposed values in config files")
        .long_about("Changes exposed values in config files.\n\
Tries to find the appropriate comment string (e.g. '#' and '//') in the first line. \
A good practise for the first line to only contain the comment string.")
        .arg(
            Arg::with_name("CONFIG")
                .help("Sets the config files to change. It is recommended to only use one config file per instance of this program, \
since the `-c` option overrides all unrecognised comment strings.")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("enabled")
            .help("Which section to enable. Can be multiple.")
                .short("e")
                .long("enabled")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("long-comment")
            .help("Enables long comments. Must be used if comment string in file is greater than 4 bytes")
                .short("l")
                .long("long-comment"),
        )
        .arg(
            Arg::with_name("c")
            .help("Override comment string found in file. Can be used if the program failed to register it.")
                .short("c")
                .long("comment-string")
                .takes_value(true))
        .arg(
            Arg::with_name("closing-comment")
                .help("An optional closing comment, for comments of type /* */")
                .long("closing-comment")
                .takes_value(true)
        );

    let matches = app.get_matches();

    let enabled: Vec<&str> = match matches.values_of("enabled") {
        Some(enabled) => enabled.collect(),
        None => Vec::new(),
    };

    let comment = {
        let primary = matches.value_of("c").map(str::as_bytes);
        let closing = matches.value_of("closing-comment").map(str::as_bytes);
        Comment::maybe_whole(primary, closing)
    };

    for file in matches.values_of("CONFIG").unwrap() {
        lib::process_file(Path::new(file), comment, &enabled);
    }
}
