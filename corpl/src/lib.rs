use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use common::ExitDisplay;

enum OptionEnabled {
    Yes,
    No,
}
impl OptionEnabled {
    pub fn from_bool(b: bool) -> Self {
        match b {
            true => Self::Yes,
            false => Self::No,
        }
    }
}

enum Segment<'a> {
    Section(&'a [u8]),
    Option(OptionEnabled),
    None,
}

pub fn process_file(path: &Path, comment: Option<&[u8]>, enabled: &[&str]) {
    let mut file = match OpenOptions::new().read(true).write(true).open(path) {
        Ok(f) => f,
        Err(_) => "Failed to open config file. Check input path.".print_exit(),
    };
    let mut config = Vec::with_capacity(4096);
    match file.read_to_end(&mut config) {
        Err(_) => "Failed to read file.".print_exit(),
        Ok(_) => {}
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
    let comment = match get_common_comments(&config).or(comment) {
        Some(c) => c,
        None => {
            let first_line = match lines.peek() {
                Some(l) => *l,
                None => "File too short; could not determine comment character.".print_exit(),
            };
            if first_line
                .split(|b| is_whitespace(*b))
                .next()
                .unwrap()
                .len()
                <= 4
            {
                let comment = first_line.split(|b| b == &32).next().unwrap();
                eprintln!(
                    "Using uncommon comment: '{}'",
                    String::from_utf8_lossy(comment)
                );
                comment
            } else {
                "Failed to get comment string. Please enter it, and only it, as the first line or supply the `-c` option with the comment string."
                    .print_exit()
            }
        }
    };

    let mut state = Segment::None;
    let mut output = Vec::with_capacity(config.len() * 2);

    for line in lines {
        let line_trimmed = trim(line);
        if line_trimmed.len() >= comment.len() + 1 + 5 + 1
            && line_trimmed.starts_with(comment)
            && line_trimmed[comment.len()..].starts_with(b" CORPL ")
        {
            let start = comment.len() + 1 + 5 + 1;

            if line_trimmed[start..].starts_with(b"end ") {
                state = Segment::None;
            } else if line_trimmed[start..].starts_with(b"section ") {
                let start = start + 8;

                let sec_str = trim_last(&line_trimmed[start..]);

                if sec_str.is_empty() {
                    eprintln!("Found a section with no replacement! Does no lines have anything in common, then append it to the section line and remove it from all the following.");
                }

                state = Segment::Section(sec_str);
            } else if line_trimmed[start..].starts_with(b"option ") {
                let start = start + 7;
                let option = &line_trimmed[start..];
                state = Segment::Option(OptionEnabled::from_bool(
                    enabled.iter().any(|e| e.as_bytes() == option),
                ));
            }
        } else {
            match state {
                Segment::Section(seg_str) => {
                    let last = get_last(line, comment);
                    let activate = enabled.iter().any(|e| e.as_bytes() == last);
                    let start = first_non_whitespace(line);
                    let currently_active =
                        !(line[start..].starts_with(comment) && line[start + comment.len()] == 32);

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
                Segment::Option(ref enabled) => {
                    let start = first_non_whitespace(line);
                    let currently_active =
                        !(line[start..].starts_with(comment) && line[start + comment.len()] == 32);

                    match enabled {
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
                            output.extend_from_slice(line_ending);
                            continue;
                        }
                        OptionEnabled::Yes => {
                            output.extend_from_slice(&line[..start]);
                            output.extend_from_slice(&line[start + comment.len() + 1..]);
                            output.extend_from_slice(line_ending);
                            continue;
                        }
                    }
                }
                Segment::None => {
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
    match byte {
        32 | 9..=13 => true,
        _ => false,
    }
}
fn trim(bytes: &[u8]) -> &[u8] {
    let mut s = 0;
    let mut e = bytes.len();

    for (pos, byte) in bytes.iter().enumerate() {
        if !is_whitespace(*byte) {
            s = pos;
            break;
        }
    }
    for (pos, byte) in bytes.iter().enumerate().rev() {
        if !is_whitespace(*byte) {
            e = pos + 1;
            break;
        }
    }
    &bytes[s..e]
}
fn trim_last(bytes: &[u8]) -> &[u8] {
    let mut e = 0;

    for (pos, byte) in bytes.iter().enumerate().rev() {
        if !is_whitespace(*byte) {
            e = pos + 1;
            break;
        }
    }

    &bytes[..e]
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

fn get_last<'a>(bytes: &'a [u8], to_match: &[u8]) -> &'a [u8] {
    for (pos, bytes_sub) in bytes.windows(to_match.len() + 1).enumerate().rev() {
        if &bytes_sub[..to_match.len()] == to_match && bytes_sub[to_match.len()] == 32 {
            return &bytes[pos + to_match.len() + 1..];
        }
    }
    return &bytes[0..0];
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
