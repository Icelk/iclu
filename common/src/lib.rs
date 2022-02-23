use std::borrow::Cow;
use std::fmt::{self, Display, Debug};
use std::io::{self, stdin, Read};

pub trait ExitDisplay {
    /// The function to print when exiting because of this type.
    /// Should not be called when getting the output.
    fn print(&self) -> String;

    /// Used as an alternative function where it consumes the type. Can be used for optimization.
    /// Should be called when getting the output.
    fn into_print(self) -> String
    where
        Self: Sized,
    {
        self.print()
    }

    fn print_exit(&self) -> ! {
        eprintln!("{}", self.print());
        std::process::exit(1)
    }
    fn into_print_exit(self) -> !
    where
        Self: Sized,
    {
        self.print_exit()
    }
}

pub struct ArgumentMissing {
    brief: &'static str,
}
impl ArgumentMissing {
    pub fn new(brief: &'static str) -> Self {
        Self { brief }
    }
}
impl ExitDisplay for ArgumentMissing {
    fn print(&self) -> String {
        format!("An argument is missing. {}\nUse the --help flag to get help with the usage of this program.", self.brief)
    }
}

impl ExitDisplay for String {
    fn print(&self) -> String {
        self.to_owned()
    }
    fn into_print(self) -> String {
        self
    }
}
impl ExitDisplay for &str {
    fn print(&self) -> String {
        self.to_string()
    }
}
impl ExitDisplay for io::Error {
    fn print(&self) -> String {
        self.to_string()
    }
}

pub fn confirm(message: &str, default: Option<bool>) -> bool {
    let y_n = match default {
        Some(default) => match default {
            true => "Y/n",
            false => "y/N",
        },
        None => "y/n",
    };
    println!("{} {}", message, y_n);
    let mut pipe = stdin();
    let mut answer = [0; 1];
    loop {
        match pipe.read(&mut answer) {
            Ok(_) => {}
            Err(_) => "Failed to read stdin. Use -f flag to force.".print_exit(),
        }
        match &answer[..] {
            b"y" | b"Y" => return true,
            b"n" | b"N" => return false,
            b"\n" | b"\r" if default.is_some() => return default.unwrap(),
            _ => {}
        }
        let default = match default {
            Some(default) => match default {
                true => " or enter to default to y.",
                false => " or enter to default to n.",
            },
            None => ".",
        };
        println!("Failed to get intent. Type 'y' or 'n'{}", default);
    }
}

pub enum SliceSplitPredicateResult {
    Continue,
    Match {
        /// The count of elements to ignore after the match.
        length: usize,
    },
}
pub trait SliceSplitPredicate<T> {
    fn matches(&mut self, rest: &[T]) -> SliceSplitPredicateResult;
}
impl<T: PartialEq> SliceSplitPredicate<T> for &[T] {
    fn matches(&mut self, rest: &[T]) -> SliceSplitPredicateResult {
        if rest.starts_with(self) {
            SliceSplitPredicateResult::Match { length: self.len() }
        } else {
            SliceSplitPredicateResult::Continue
        }
    }
}
impl<T: PartialEq, const C: usize> SliceSplitPredicate<T> for &[T; C] {
    fn matches(&mut self, rest: &[T]) -> SliceSplitPredicateResult {
        let mut slice = &self[..];
        slice.matches(rest)
    }
}
impl<T, F: FnMut(&[T]) -> SliceSplitPredicateResult> SliceSplitPredicate<T> for F {
    fn matches(&mut self, rest: &[T]) -> SliceSplitPredicateResult {
        self(rest)
    }
}
/// Enables splitting of any [`SliceSplitPredicate`] for the `slice`.
///
/// This enables splitting a slice when another slice occurs, as you do with [`str::split`].
pub fn slice_split<T, P: SliceSplitPredicate<T>>(
    slice: &[T],
    predicate: P,
) -> SliceSplit<'_, T, P> {
    SliceSplit {
        index: 0,
        last_split: 0,
        predicate,
        slice,
    }
}
pub struct SliceSplit<'a, T, Predicate: SliceSplitPredicate<T>> {
    index: usize,
    last_split: usize,
    predicate: Predicate,
    slice: &'a [T],
}
impl<'a, T: Debug, Predicate: SliceSplitPredicate<T>> Iterator for SliceSplit<'a, T, Predicate> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index + 1 >= self.slice.len() {
                if self.last_split <= self.slice.len() {
                    // println!("Self slice {:?}",  self.slice);
                    let slice = &self.slice[self.last_split..];

                    self.last_split = self.slice.len()+1;

                    return Some(slice);
                }
                return None;
            }
            if self.last_split <= self.index {
                match self.predicate.matches(&self.slice[self.index..]) {
                    SliceSplitPredicateResult::Match { length } => {
                        let slice = &self.slice[self.last_split..self.index];

                        self.last_split = self.index+length;
                        return Some(slice);
                    }
                    SliceSplitPredicateResult::Continue => {}
                }
            }
            self.index += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscapeError {
    InvalidCharacter,
    InvalidEscape,
}
impl Display for EscapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEscape => {
                write!(f, "escape was invalid. Available are: \\n, \\t, \\r, \\\\, \\0, \\', \\\", \\x<2 hex digits>, \\u{{<up to 6 hex digits>}}")
            }
            Self::InvalidCharacter => {
                write!(
                    f,
                    "the character you inputted (after \\x or \\u) isn't valid in UTF-8"
                )
            }
        }
    }
}
/// Parses `s` and resolves any `\<special character>`.
///
/// - `\n` -> newline
/// - `\t` -> tab
/// - `\r` -> carriage return
/// - `\\` -> backslash
/// - `\0` -> null
/// - `\'` -> '
/// - `\"` -> "
/// - `\x<2 hex digits>` -> corresponding ASCII character
/// - `\u{<1..=6 hex digits>}` -> corresponding Unicode character
///
/// This will return [`EscapeError::InvalidCharacter`] if a character couldn't be constructed from
/// the input.
/// [`EscapeError::InvalidEscape`] is returned is an invalid character was found after a `\`.
pub fn parse_escaped(s: &str) -> Result<Cow<'_, str>, EscapeError> {
    if s.contains('\\') {
        let mut string = String::with_capacity(s.len());
        let mut skip = 0;
        for (idx, c) in s.char_indices() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if c == '\\' {
                let s = &s[idx..];
                let first_char = s.chars().nth(1).ok_or(EscapeError::InvalidEscape)?;
                skip = match first_char {
                    'n' => {
                        string.push('\n');
                        1
                    }
                    't' => {
                        string.push('\t');
                        1
                    }
                    'r' => {
                        string.push('\r');
                        1
                    }
                    '\\' => {
                        string.push('\\');
                        1
                    }
                    '0' => {
                        string.push('\0');
                        1
                    }
                    '\'' => {
                        string.push('\'');
                        1
                    }
                    '"' => {
                        string.push('"');
                        1
                    }
                    'x' => {
                        let digits = s.get(2..=3).ok_or(EscapeError::InvalidCharacter)?;
                        debug_assert_eq!(digits.as_bytes().len(), 2);
                        let number = u32::from_str_radix(digits, 16)
                            .map_err(|_| EscapeError::InvalidCharacter)?;
                        let c = char::from_u32(number).ok_or(EscapeError::InvalidCharacter)?;
                        string.push(c);
                        3
                    }
                    'u' => {
                        let closing = s.find('}');
                        let closing = if let Some(c) = closing {
                            c - 2 // the opening and `u` are two bytes
                        } else {
                            return Err(EscapeError::InvalidCharacter);
                        };
                        if s.as_bytes().get(2) != Some(&b'{') || closing > 6 {
                            return Err(EscapeError::InvalidCharacter);
                        }
                        // ok, since we found `closing` in slice.
                        let digits = &s[3..closing + 2];
                        let number = u32::from_str_radix(digits, 16)
                            .map_err(|_| EscapeError::InvalidCharacter)?;
                        let c = char::from_u32(number).ok_or(EscapeError::InvalidCharacter)?;
                        string.push(c);
                        2 + closing
                    }

                    _ => return Err(EscapeError::InvalidEscape),
                };
            } else {
                string.push(c)
            }
        }
        Ok(Cow::Owned(string))
    } else {
        Ok(Cow::Borrowed(s))
    }
}
