use std::io::{stdin, Read};

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
