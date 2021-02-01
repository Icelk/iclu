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
