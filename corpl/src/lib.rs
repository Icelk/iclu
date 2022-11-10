use std::collections::HashSet;
use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use common::ExitDisplay;

#[derive(Debug, Clone, Copy)]
pub struct Comment<'a> {
    open: &'a [u8],
    close: Option<&'a [u8]>,
}
impl<'a> Comment<'a> {
    pub fn maybe_whole(open: Option<&'a [u8]>, close: Option<&'a [u8]>) -> Option<Self> {
        open.map(|open| Self { open, close })
    }
    pub fn open(&self) -> &[u8] {
        self.open
    }
    pub fn close(&self) -> Option<&[u8]> {
        self.close
    }
}

#[derive(Debug, PartialEq, Eq)]
enum OptionEnabled {
    Yes,
    No,
    Ignore,
}

enum Segment<'a> {
    Section(&'a [u8]),
    Option(OptionEnabled),
    None,
}

pub fn process_file(
    path: &Path,
    comment: Option<Comment>,
    enabled: &HashSet<&[u8]>,
    disabled: &HashSet<&[u8]>,
    keep: bool,
    max_comment_len: Option<usize>,
) {
    let get_status = |option: &[u8]| {
        if keep {
            if disabled.contains(option) {
                Some(false)
            } else if enabled.contains(option) {
                Some(true)
            } else {
                None
            }
        } else {
            Some(enabled.contains(option))
        }
    };

    let mut file = match OpenOptions::new().read(true).write(true).open(path) {
        Ok(f) => f,
        Err(_) => "Failed to open config file. Check input path.".print_exit(),
    };
    let mut config = Vec::with_capacity(4096);
    if file.read_to_end(&mut config).is_err() {
        "Failed to read file.".print_exit()
    };
    let line_ending = get_line_ending(&config);
    let mut lines = get_lines(&config).peekable();
    fn get_common_comments(bytes: &[u8]) -> Option<&'static [u8]> {
        if bytes.starts_with(b"#") {
            Some(b"#")
        } else if bytes.starts_with(b"//") {
            Some(b"//")
        } else if bytes.starts_with(b";") {
            Some(b";")
        } else {
            None
        }
    }
    let end_comment = comment.as_ref().and_then(Comment::close);
    let comment = match get_common_comments(&config).or_else(|| comment.as_ref().map(Comment::open))
    {
        Some(c) => c,
        None => {
            let first_line = match lines.peek() {
                Some(l) => *l,
                None => "File too short; could not determine comment character.".print_exit(),
            };
            if max_comment_len.is_none()
                || first_line
                    .split(|b| is_whitespace(*b))
                    .next()
                    .unwrap()
                    .len()
                    <= max_comment_len.unwrap()
            {
                let comment = first_line.split(|b| b == &32).next().unwrap();
                eprintln!(
                    "Continuing with uncommon comment: '{}'",
                    String::from_utf8_lossy(comment)
                );
                comment
            } else {
                format!("Failed to get comment string in {}. Please enter it, and only it, as the first line or supply the `-c` option with the comment string.", path.display())
                    .print_exit()
            }
        }
    };

    let mut state = Segment::None;
    let mut output = Vec::with_capacity(config.len() * 2);

    for line in lines {
        let line_trimmed = trim(line);
        #[allow(clippy::int_plus_one)]
        if line_trimmed.len() >= comment.len() + 1 + 5 + 1
            && line_trimmed.starts_with(comment)
            && line_trimmed[comment.len()..].starts_with(b" CORPL ")
        {
            let start = comment.len() + 1 + 5 + 1;

            let is_end = match end_comment.as_ref() {
                None => &line_trimmed[start..] == b"end",
                Some(end_comment) => {
                    &line_trimmed[start..start + 4] == b"end "
                        && &line_trimmed[start + 4..] == *end_comment
                }
            };

            if is_end {
                state = Segment::None;
            } else if line_trimmed[start..].starts_with(b"section ") {
                let trimmed_start = first_non_whitespace(line);
                let start = trimmed_start + start + 8;

                let sec_str = &line[start..];

                if sec_str.is_empty() {
                    eprintln!("Found a section with no replacement! Does no lines have anything in common, then append it to the section line and remove it from all the following.");
                }

                state = Segment::Section(sec_str);
                if end_comment.is_some() {
                    eprintln!("End comment is not compatible with sections for now.");
                    state = Segment::None;
                }
            } else if line_trimmed[start..].starts_with(b"option ") {
                let start = start + 7;
                let end = match end_comment.as_ref() {
                    Some(c) => line_trimmed.len() - c.len() - 1,
                    None => line_trimmed.len(),
                };
                let option = &line_trimmed[start..end];
                let options = common::slice_split(option, b" && ");
                let mut option_enabled = OptionEnabled::Ignore;
                for option in options {
                    let trimmed_option = option.strip_prefix(b"!").unwrap_or(option);
                    let contains = get_status(trimmed_option);
                    let negate = option.starts_with(b"!");
                    let Some(contains) = contains else { continue };

                    if option_enabled == OptionEnabled::Ignore {
                        option_enabled = OptionEnabled::Yes;
                    }
                    if (negate && contains) || (!negate && !contains) {
                        option_enabled = OptionEnabled::No;
                        break;
                    }
                }
                state = Segment::Option(option_enabled);
            }
        } else {
            match state {
                Segment::Section(seg_str) if !line_trimmed.is_empty() => {
                    if end_comment.is_none() {
                        let last = get_last(line, comment);
                        let activate = last.map_or(Some(false), |last| get_status(last));
                        let Some(activate) = activate else {
                            // return early
                            output.extend_from_slice(line);
                            output.extend_from_slice(line_ending);
                            continue;
                        };
                        let start = first_non_whitespace(line);
                        let currently_active = !(line[start..].starts_with(comment)
                            && line.get(start + comment.len()).copied() == Some(32));

                        if currently_active == activate {
                            // Do noting!
                        } else if !activate {
                            if line[start..].starts_with(seg_str) {
                                // Do stuff
                                output.extend_from_slice(&line[..start]);
                                output.extend_from_slice(comment);
                                output.push(32);
                                output.extend_from_slice(&line[start + seg_str.len()..]);
                                output.extend_from_slice(line_ending);
                                continue;
                            } else {
                                eprintln!("Common string of section not present! Ignoring line.")
                            }
                        } else if activate {
                            output.extend_from_slice(&line[..start]);
                            output.extend_from_slice(seg_str);
                            output.extend_from_slice(&line[start + comment.len() + 1..]);
                            output.extend_from_slice(line_ending);
                            continue;
                        }
                    }
                }
                Segment::Option(ref enabled) if !line_trimmed.is_empty() => {
                    let start = first_non_whitespace(line);
                    let currently_active =
                        !(line[start..].starts_with(comment) && line[start + comment.len()] == 32);

                    match enabled {
                        OptionEnabled::Ignore => {
                            // Do nothing
                        }
                        OptionEnabled::Yes if currently_active => {
                            // Do nothing
                        }
                        OptionEnabled::No if !currently_active => {
                            // Same here
                        }
                        OptionEnabled::No => {
                            output.extend_from_slice(&line[..start]);
                            output.extend_from_slice(comment);
                            output.push(32);
                            output.extend_from_slice(&line[start..]);
                            if let Some(comment) = end_comment {
                                output.push(32);
                                output.extend_from_slice(comment);
                            }
                            output.extend_from_slice(line_ending);
                            continue;
                        }
                        OptionEnabled::Yes => {
                            output.extend_from_slice(&line[..start]);
                            match end_comment {
                                None => {
                                    output.extend_from_slice(&line[start + comment.len() + 1..])
                                }
                                Some(end_comment) => {
                                    let end = last_non_whitespace(line);
                                    output.extend_from_slice(
                                        &line[start + comment.len() + 1
                                            ..end - end_comment.len() - 1],
                                    );
                                    output.extend_from_slice(&line[end..]);
                                }
                            }
                            output.extend_from_slice(line_ending);
                            continue;
                        }
                    }
                }
                _ => {
                    // Do nothing
                }
            }
        }
        output.extend_from_slice(line);
        // Newline character
        output.extend_from_slice(line_ending);
    }
    match file.set_len(output.len() as u64) {
        Ok(_) => {}
        Err(_) => "Failed to set file length.".print_exit(),
    };
    match file.seek(SeekFrom::Start(0)) {
        Ok(_) => {}
        Err(_) => "Failed to seek in file.".print_exit(),
    }
    match file.write_all(&output[..]) {
        Ok(_) => {}
        Err(_) => "Failed to write to file.".print_exit(),
    }
}

#[derive(Debug, Clone)]
struct Lines<'a> {
    bytes: &'a [u8],
    current_pos: usize,
}
impl<'a> Iterator for Lines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos == self.bytes.len() {
            return None;
        }
        let mut end = None;
        for (pos, byte) in self.bytes[self.current_pos..].iter().enumerate() {
            if *byte == 13 || *byte == 10 {
                end = Some(pos);
                break;
            }
        }
        let end = match end {
            Some(e) => e + self.current_pos,
            None => self.bytes.len(),
        };
        let bytes = &self.bytes[self.current_pos..end];
        let new_start = {
            if self.bytes[end] == 13 {
                end + 2
            } else {
                end + 1
            }
        };
        self.current_pos = new_start;
        Some(bytes)
    }
}
fn get_lines(bytes: &[u8]) -> Lines {
    Lines {
        bytes,
        current_pos: 0,
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, 32 | 9..=13)
}
fn trim(bytes: &[u8]) -> &[u8] {
    &bytes[first_non_whitespace(bytes)..last_non_whitespace(bytes)]
}
fn first_non_whitespace(bytes: &[u8]) -> usize {
    let mut s = 0;

    for (pos, byte) in bytes.iter().enumerate() {
        if !is_whitespace(*byte) {
            s = pos;
            break;
        }
    }
    s
}
fn last_non_whitespace(bytes: &[u8]) -> usize {
    let mut e = 0;

    for (pos, byte) in bytes.iter().enumerate().rev() {
        if !is_whitespace(*byte) {
            e = pos + 1;
            break;
        }
    }
    e
}

fn get_last<'a>(bytes: &'a [u8], to_match: &[u8]) -> Option<&'a [u8]> {
    for (pos, bytes_sub) in bytes.windows(to_match.len() + 1).enumerate().rev() {
        if &bytes_sub[..to_match.len()] == to_match && bytes_sub[to_match.len()] == 32 {
            return Some(&bytes[pos + to_match.len() + 1..]);
        }
    }
    None
}

fn get_line_ending(bytes: &[u8]) -> &'static [u8] {
    for byte in bytes.iter().copied() {
        match byte {
            13 => return b"\r\n",
            10 => return b"\n",
            _ => {}
        }
    }
    b"\n"
}
