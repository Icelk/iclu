//! BYtes Convertor

use common::ExitDisplay;
use getopts::Options;
use std::{borrow::Cow, char, env, io, io::prelude::*};

pub fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!(
        "Usage: {prog} RANGE... [options]\n\n\
Will read input from stdin \
(often piped from another program, such as ran using the ascii range) \
and convert numbers to characters according to UTF-8.\n",
        prog = program,
    );
    let usage = opts.usage(&brief);
    usage.print_exit()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].as_str();

    let mut opts = getopts::Options::new();
    opts.optflag("", "help", "prints this help menu");
    opts.optopt(
        "s",
        "separator",
        "set the separator to split stdin with.",
        "",
    );
    opts.optflag("b", "binary", "parses the input as binary");
    opts.optflag("h", "hex", "parses the input as hexadecimal");
    opts.optflag("d", "decimal", "parses the input as decimal");
    opts.optopt("r", "base", "parses the input as the given base", "BASE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => f.to_string().print_exit(),
    };
    if matches.opt_present("help") {
        print_usage(program, opts);
    }
    let separator = matches
        .opt_default("s", "\n")
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed("\n"));
    let base = {
        let b = if matches.opt_present("b") { 1 } else { 0 };
        let h = if matches.opt_present("h") { 1 } else { 0 };
        let d = if matches.opt_present("d") { 1 } else { 0 };
        let r = if matches.opt_present("r") { 1 } else { 0 };
        if b + h + d + r > 1 {
            "Flag `binary`, `hex`, `decimal`, and `base` are exclusive. See --help for more info."
                .print_exit()
        }
        if b == 1 {
            2
        } else if h == 1 {
            16
        } else if d == 1 {
            10
        } else {
            match matches.opt_get_default("r", 10) {
                Err(_) => "Failed to parse custom base.".print_exit(),
                Ok(b) if b < 2 || b > 36 => "Base is not in range 2-36.".print_exit(),
                Ok(b) => b,
            }
        }
    };

    let mut chars = String::with_capacity(512);
    let mut buffer = Vec::with_capacity(4096);

    match io::stdin().read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(_) => "Failed to read stdin.".print_exit(),
    };

    let string = match String::from_utf8(buffer) {
        Err(_) => "Failed to convert to utf-8".print_exit(),
        Ok(s) => s,
    };
    for byte in string.split(separator.as_ref()) {
        let byte = byte.trim();
        if byte.is_empty() {
            continue;
        }
        let int = match u32::from_str_radix(byte, base) {
            Ok(i) => i,
            Err(_) => format!(
                "Failed to parse '{}' to a integer. Check the base you're using.",
                byte
            )
            .print_exit(),
        };
        let char = match char::from_u32(int) {
            Some(c) => c,
            None => format!("Failed to convert '{}' to a character.", int).print_exit(),
        };
        chars.push(char);
    }

    let mut stdout = io::stdout();
    match stdout
        .write_all(chars.as_bytes())
        .and(stdout.write(b"\n"))
        .and(stdout.flush())
    {
        Err(_) => "Failed to write to stdout.".print_exit(),
        Ok(()) => (),
    }
}
