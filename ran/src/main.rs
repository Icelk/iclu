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
They should not be overlapping or outside the range of a 128-bit signed integer.\n\
They can be comma- or space separated, contain a starting and ending number with a hyphen in between \
(e.g. '3..5,7..11' is equivalent to '3..5,  7..11')\n\
If you want negative numbers, make sure to include -- before the ranges \
(e.g. {prog} -- -3..-1)\n\
\n\
Several hard-coded ranges are present.\n\
ascii -> 32..127\n\
ascii-ext -> [32..127), [160..256)\n\
alphabet | letters | [a-zA-Z] -> [65, 91), [97, 123)\n\
capitals | uppercase | majuscule | [a-z] -> [97, 123)\n\
lowercase | minuscule | [A-Z] -> [65, 91)\n\
numbers | [0-9] -> [48, 58)\n\
password -> 33, [35..37], [39, 41], [43, 58], [63, 123], [125, 126]\n\
i8 -> [-128..128)\n\
u8 -> [0..256)\n\
i16 -> [-32768..32768)\n\
u16 -> [0..65536)\n\
i32 -> [-2147483648..2147483648)\n\
u32 -> [0..4294967296)\n\
i64 -> [-9223372036854775808..9223372036854775808)\n\
u64 -> [0..18446744073709551616)", prog=program,);
    let usage = opts.usage(&brief);
    usage.print_exit()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Range {
    from: i128,
    to: i128,
}
impl Range {
    /// Creates a range `[from..to)`
    pub const fn new(from: i128, to: i128) -> Range {
        Range { from, to }
    }
    /// Creates a range `[from..to]`
    pub const fn new_inclusive(from: i128, to: i128) -> Range {
        Range { from, to: to + 1 }
    }
    /// Creates a range of a single number. Same as `new(value, value + 1)`.
    pub const fn single(value: i128) -> Range {
        Range {
            from: value,
            to: value + 1,
        }
    }
    pub fn intersects(&self, other: &Self) -> bool {
        other.to > self.from && other.from < self.to
    }

    pub fn count(&self) -> i128 {
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
                "The intager is invalid. Make sure no other characters than 0-9 \
                are present and the integer is inside the range of 64 bits."
            }
            RangeError::Intersecting => "Two or more ranges are intersecting.",
        };
        format!("An error occurred while parsing a range. {}", error)
    }
}

fn parse_ranges<'a, I: Iterator<Item = &'a str>>(ranges: I) -> Vec<Range> {
    ranges
        .flat_map(|s| match s.trim() {
            "ascii" => vec![Range::new(32, 127)],
            "ascii-ext" => vec![Range::new(32, 127), Range::new(160, 256)],
            "alphabet" | "letters" | "[a-zA-Z]" => vec![Range::new(65, 91), Range::new(97, 123)],
            "capitals" | "uppercase" | "majuscule" | "[a-z]" => vec![Range::new(97, 123)],
            "lowercase" | "minuscule" | "[A-Z]" => vec![Range::new(65, 91)],
            "numbers" | "[0-9]" => vec![Range::new(48, 58)],
            "password" => vec![
                Range::single(33),
                Range::new_inclusive(35, 37),
                Range::new_inclusive(39, 41),
                Range::new_inclusive(43, 58),
                Range::new_inclusive(63, 123),
                Range::new_inclusive(125, 126),
            ],
            "i8" => vec![Range::new(-128, 128)],
            "u8" => vec![Range::new(0, 256)],
            "i16" => vec![Range::new(-32768, 32768)],
            "u16" => vec![Range::new(0, 65536)],
            "i32" => vec![Range::new(-2147483648, 2147483648)],
            "u32" => vec![Range::new(0, 4294967296)],
            "i64" => vec![Range::new(-9223372036854775808, 9223372036854775808)],
            "u64" => vec![Range::new(0, 18446744073709551615)],

            _ => match s.parse::<Range>() {
                Err(e) => e.print_exit(),
                Ok(r) => vec![r],
            },
        })
        .collect()
}

/// Returns the `value` clamped to the ranges.
/// `value` is assumed to be zero-indexed and have a
/// maximum of `ranges.fold(0, |acc, r| r.difference() + acc)` Will else return -1.
/// `ranges` are assumed to be in order, with the smallest first.
/// This does however not matter when the `value` is random.
fn clamp_to_ranges(value: i128, ranges: &[Range]) -> i128 {
    let mut left = value;
    for range in ranges {
        if left - range.count() < 0 {
            return left + range.from;
        } else {
            left -= range.count();
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

    let ranges = parse_ranges(
        matches
            .free
            .iter()
            .flat_map(|a| a.split(','))
            .filter(|s| !s.trim().is_empty()),
    );

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

    let total = ranges.iter().fold(0, |acc, r| acc + r.count());

    let mut rng = thread_rng();

    fn get_numbers(
        mut sample: impl FnMut() -> i128,
        amount: usize,
        ranges: &[Range],
        separator: Cow<str>,
    ) -> String {
        (0..amount)
            .into_iter()
            .map(|_| clamp_to_ranges(sample(), ranges))
            .fold(String::with_capacity(512), |mut s, n| {
                if !s.is_empty() {
                    s.push_str(separator.as_ref());
                }
                s.push_str(format!("{}", n).as_str());
                s
            })
    }

    let numbers = if let Ok(u) = u16::try_from(total) {
        let range = Uniform::new(0, u);
        get_numbers(
            || range.sample(&mut rng) as i128,
            amount,
            &ranges,
            separator,
        )
    } else if let Ok(u) = u32::try_from(total) {
        let range = Uniform::new(0, u);
        get_numbers(
            || range.sample(&mut rng) as i128,
            amount,
            &ranges,
            separator,
        )
    } else if let Ok(u) = u64::try_from(total) {
        let range = Uniform::new(0, u);
        get_numbers(
            || range.sample(&mut rng) as i128,
            amount,
            &ranges,
            separator,
        )
    } else {
        let range = Uniform::new(0, total);
        get_numbers(|| range.sample(&mut rng), amount, &ranges, separator)
    };

    // println!(
    //     "Ranges: {:?}! Total {} number: {} separator {:?}",
    //     &ranges, total, amount, separator
    // );
    let mut stdout = io::stdout();
    if stdout
        .write_all(numbers.as_bytes())
        .and(stdout.write(b"\n"))
        .and(stdout.flush())
        .is_err()
    {
        "Failed to write to stdout.".print_exit()
    }
}
