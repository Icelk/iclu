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
