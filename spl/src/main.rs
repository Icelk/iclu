//! `SPLit`
#![deny(clippy::pedantic, clippy::perf)]

use std::borrow::Cow;
use std::io::{stdin, stdout, Read, Write};

use clap::{Arg, ArgGroup};
use common::ExitDisplay;

fn main() {
    let command = clap::command!();
    let command = command
        .long_about("Splits incoming data and joins it with a specified separator.\n\
        Uses streams for best performance.")
        .arg(
            Arg::new("null")
                .short('0')
                .long("null")
                .help("Splits incoming data on null bytes."),
        )
        .arg(Arg::new("split").help("Separator to split incoming data with. Escapes (e.g. \\n, \\t, \\0, \\) are allowed."))
        .arg(
            Arg::new("join")
                .help("The string to join each entry by. Escapes (e.g. \\n, \\t, \\0, \\) are allowed.")
                .default_value("\\n"),
        ).group(ArgGroup::new("split_input").arg("null").arg("split").required(true));

    let matches = command.get_matches();

    let split = if matches.is_present("null") {
        Cow::Borrowed("\0")
    } else {
        // UNWRAP: we require either this or `null` to be present.
        let split = matches.value_of("split").unwrap();
        match common::parse_escaped(split) {
            Ok(split) => split,
            Err(err) => format!("Parsing of separator failed: {}", err).print_exit(),
        }
    };
    let join = {
        // UNWRAP: there is always a default value.
        let join = matches.value_of("join").unwrap();
        match common::parse_escaped(join) {
            Ok(split) => split,
            Err(err) => format!("Parsing of separator failed: {}", err).print_exit(),
        }
    };

    let stdin = stdin();
    let stdout = stdout();
    let mut buf = vec![0; split.len() + 4096]; // `split.len() + ` so we don't have `buf.is_empty` at `read`.
    let mut left = 0;

    let process = |to_process: &[u8]| {
        let mut lock = stdout.lock();
        let count = common::slice_split(to_process, split.as_bytes()).count();
        for (pos, slice) in common::slice_split(to_process, split.as_bytes()).enumerate() {
            let slice = slice.strip_suffix(split.as_bytes()).unwrap_or(slice);
            lock.write_all(slice)
                .map_err(ExitDisplay::into_print_exit)
                .unwrap();

            if pos + 1 != count {
                lock.write_all(join.as_bytes())
                    .map_err(ExitDisplay::into_print_exit)
                    .unwrap();
            }
        }
        lock.flush().map_err(ExitDisplay::into_print_exit).unwrap();
    };

    loop {
        let data = {
            let mut lock = stdin.lock();
            let read = lock
                .read(&mut buf[left..])
                .map_err(ExitDisplay::into_print_exit)
                .unwrap();
            if read == 0 {
                break;
            }
            &buf[..read]
        };
        // don't process last bytes if they are on a split boundary
        let to_process = &data[..data.len().saturating_sub(split.len().saturating_sub(1))];

        process(to_process);

        {
            let len = data.len();
            let data_idx = len.saturating_sub(split.len().saturating_sub(1));
            let buf_idx = data_idx;
            buf.copy_within(buf_idx.., 0);
            left = len - data_idx;
        }
    }

    process(&buf[..left]);
}
