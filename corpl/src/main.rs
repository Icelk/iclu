use std::collections::HashSet;
use std::path::Path;

use clap::{self, Arg, ArgAction, Command};

use common::ExitDisplay;
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
            Arg::new("enable")
                .help("Which section to enable. Can be multiple.")
                .short('e')
                .long("enable")
                .action(ArgAction::Append)
                .num_args(1),
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
            Arg::new("disable")
                .short('d')
                .long("disable")
                .help("Sections to explicitly disable. Implies `keep`")
                .action(ArgAction::Append)
                .num_args(1),
        );

    let matches = app.get_matches();

    let enable: HashSet<_> = match matches.get_many::<String>("enable") {
        Some(enable) => enable
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().as_bytes())
            .collect(),
        None => HashSet::new(),
    };
    let disable: HashSet<_> = match matches.get_many::<String>("disable") {
        Some(disable) => disable
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().as_bytes())
            .collect(),
        None => HashSet::new(),
    };
    let keep = matches.get_flag("keep") || !disable.is_empty();

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

    let mut errors = vec![];
    for file in matches.get_many::<String>("CONFIG").unwrap() {
        if let Err(err) = corpl::process_file(
            Path::new(file),
            comment,
            &enable,
            &disable,
            keep,
            comment_len,
        ) {
            errors.push((err, file))
        };
    }
    let last = errors.pop();
    for (err, path) in errors {
        eprintln!("{err} Error when processing {path}")
    }
    if let Some((err, path)) = last {
        format!("{err} Error when processing {path}").print_exit()
    }
}
