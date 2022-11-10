use std::collections::HashSet;
use std::path::Path;

use clap::{self, Arg, ArgAction, Command};

use corpl::Comment;

fn main() {
    let app = Command::new("corpl")
        .version("0.1.0")
        .author("Icelk <main@icelk.dev>")
        .about("Changes exposed values in config files")
        .long_about(
            "Changes exposed values in config files.\n\
            Tries to find the appropriate comment string (e.g. '#' and '//') in the first line. \
            A good practise for the first line to only contain the comment string.",
        )
        .arg(
            Arg::new("CONFIG")
                .help(
                    "Sets the config files to change. \
                It is recommended to only use one config \
                file per instance of this program, \
                since the `-c` option overrides all \
                unrecognised comment strings.",
                )
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("enabled")
                .help("Which section to enable. Can be multiple.")
                .short('e')
                .long("enabled")
                .num_args(1..),
        )
        .arg(
            Arg::new("long-comment")
                .help(
                    "Enables long comments. Must be used if comment \
                    string in file is greater than 4 bytes",
                )
                .short('l')
                .action(ArgAction::SetTrue)
                .long("long-comment"),
        )
        .arg(
            Arg::new("comment")
                .help(
                    "Override comment string found in file. \
                    Can be used if the program failed to register it.",
                )
                .short('c')
                .long("comment")
                .num_args(1),
        )
        .arg(
            Arg::new("closing-comment")
                .help("An optional closing comment, for comments of type /* */")
                .long("closing-comment")
                .num_args(1),
        )
        .arg(
            Arg::new("keep")
                .short('k')
                .long("keep")
                .action(ArgAction::SetTrue)
                .help("Keep current settings"),
        )
        .arg(
            Arg::new("disabled")
                .short('d')
                .long("disabled")
                .help("Sections to explicitly disable. Implies `keep`")
                .num_args(1..),
        );

    let matches = app.get_matches();

    let enabled: HashSet<_> = match matches.get_many::<String>("enabled") {
        Some(enabled) => enabled.map(|s| s.as_bytes()).collect(),
        None => HashSet::new(),
    };
    let disabled: HashSet<_> = match matches.get_many::<String>("disabled") {
        Some(disabled) => disabled.map(|s| s.as_bytes()).collect(),
        None => HashSet::new(),
    };
    let keep = matches.get_flag("keep");

    let comment = {
        let primary = matches.get_one::<String>("comment").map(|s| s.as_bytes());
        let closing = matches
            .get_one::<String>("closing-comment")
            .map(|s| s.as_bytes());
        Comment::maybe_whole(primary, closing)
    };
    let comment_len = if matches.get_flag("long-comment") {
        None
    } else {
        Some(4)
    };

    for file in matches.get_many::<String>("CONFIG").unwrap() {
        corpl::process_file(
            Path::new(file),
            comment,
            &enabled,
            &disabled,
            keep,
            comment_len,
        );
    }
}
