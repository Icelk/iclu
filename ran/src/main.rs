//! RANdom

use common::*;
use getopts::Options;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use std::{borrow::Cow, env, io, io::prelude::*};

pub fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {prog} RANGE... [options]\n\n\
RANGE defines which ranges to randomise integers in.\n\
They should not be overlapping or outside the range of a 64-bit integer.\n\
They can be comma- or space separated, contain a starting and ending number with a hyphen in between \
(e.g. '3..5,7..11' is equivalent to '3..5,  7..11')\n\
If you want negative numbers, make sure to include -- before the ranges \
(e.g. {prog} -- -3..-1)"
, prog=program,);
    let usage = opts.usage(&brief);
    usage.print_exit()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Range {
    from: i64,
    to: i64,
}
impl Range {
    pub fn intersects(&self, other: &Self) -> bool {
        other.to > self.from && other.from < self.to
    }

    pub fn difference(&self) -> i64 {
        self.to - self.from
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum RangeError {
    /// Range is backwards
    Backwards,
    /// Failed to parse number. May be because range is outside limits of i64
    InvalidInteger,
    /// Syntactic error
    Syntax,
    /// Two or more ranges intersect.
    Intersecting,
}
impl std::str::FromStr for Range {
    type Err = RangeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = 0;
        let mut positions = [0_usize; 4];
        let mut dots = 0;

        for (pos, c) in s.char_indices() {
            match c {
                ',' => continue,
                ' ' if state % 2 == 0 => continue,
                '.' if state % 2 == 1 => dots += 1,
                '.' if state % 2 == 0 => continue,
                '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
                    if state % 2 == 1 =>
                {
                    continue
                }
                _ => (),
            }
            if c != '.' {
                dots = 0;
            }
            if dots != 2 && state % 2 == 1 {
                continue;
            }
            if state == 1 {
                positions[state] = pos - 1;
            } else {
                positions[state] = pos;
            }
            state += 1;
        }
        if positions[3] == 0 {
            positions[3] = s.len();
        }
        let from = s
            .get(positions[0]..positions[1])
            .ok_or(RangeError::Syntax)?
            .parse()
            .ok()
            .ok_or(RangeError::InvalidInteger)?;
        let to = s
            .get(positions[2]..positions[3])
            .ok_or(RangeError::Syntax)?
            .parse()
            .ok()
            .ok_or(RangeError::InvalidInteger)?;

        if from >= to {
            return Err(RangeError::Backwards);
        }

        Ok(Range { from, to })
    }
}
impl ExitDisplay for RangeError {
    fn print(&self) -> String {
        let error = match self {
            RangeError::Backwards => "The range is entered backwards.",
            RangeError::Syntax => "The syntax is wrong. See the usage (--help).",
            RangeError::InvalidInteger => {
                "The intager is invalid. Make sure no other characters than 0-9 are present and the integer is inside the range of 64 bits."
            }
            RangeError::Intersecting => "Two or more ranges are intersecting.",
        };
        format!("An error occurred while parsing a range. {}", error)
    }
}

/// Returns the `value` clamped to the ranges.
/// `value` is assumed to be zero-indexed and have a maximum of `ranges.fold(0, |acc, r| r.difference() + acc)` Will else return -1.
/// `ranges` are assumed to be in order, with the smallest first. This does however not matter when the `value` is random.
fn clamp_to_ranges(value: i64, ranges: &[Range]) -> i64 {
    let mut left = value;
    for range in ranges {
        if left - range.difference() < 0 {
            return left + range.from;
        } else {
            left -= range.difference();
        }
    }
    -1
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].as_str();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help menu");
    opts.optopt(
        "s",
        "separator",
        "set the separator between the output.",
        "",
    );
    opts.optopt("n", "number", "amount of random numbers", "");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => f.to_string().print_exit(),
    };
    if matches.opt_present("h") {
        print_usage(program, opts);
    }
    if matches.free.is_empty() {
        ArgumentMissing::new("Please supply at least one range.").print_exit()
    }

    let separator = matches
        .opt_default("s", "\n")
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed("\n"));

    let amount = match matches.opt_get_default("n", 10_usize) {
        Ok(a) => a,
        Err(_) => "Failed to parse amount of random numbers. See --help for usage.".print_exit(),
    };

    let ranges: Vec<Range> = matches
        .free
        .iter()
        .map(|a| a.split(","))
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .map(|s| match s.parse::<Range>() {
            Err(e) => e.print_exit(),
            Ok(r) => r,
        })
        .collect();

    for (pos, range) in ranges.iter().enumerate() {
        for (cmp_pos, cmp_range) in ranges.iter().enumerate() {
            if pos == cmp_pos {
                continue;
            }
            if range.intersects(cmp_range) {
                RangeError::Intersecting.print_exit()
            }
        }
    }

    let total = ranges.iter().fold(0, |acc, r| acc + r.difference());

    let mut rng = thread_rng();

    let range = Uniform::new(0, total);

    let numbers = (0..amount)
        .into_iter()
        .map(|_| clamp_to_ranges(range.sample(&mut rng), &ranges))
        .fold(String::with_capacity(512), |mut s, n| {
            if !s.is_empty() {
                s.push_str(separator.as_ref());
            }
            s.push_str(format!("{}", n).as_str());
            s
        });

    // println!(
    //     "Ranges: {:?}! Total {} number: {} separator {:?}",
    //     &ranges, total, amount, separator
    // );
    let mut stdout = io::stdout();
    match stdout
        .write_all(numbers.as_bytes())
        .and(stdout.write(b"\n"))
        .and(stdout.flush())
    {
        Err(_) => "Failed to write to stdout.".print_exit(),
        Ok(()) => (),
    }
}
