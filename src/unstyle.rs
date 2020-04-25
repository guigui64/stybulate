use std::fmt;

/// Trait to implement when a string length differs from its unstyled value length (ASCII escapes
/// for instance)
pub trait Unstyle: fmt::Display {
    /// Returns the same string without the style-related chars/codepoints
    fn unstyle(&self) -> String {
        self.to_string()
    }

    /// Returns the number of lines in the string
    fn nb_of_lines(&self) -> usize {
        self.to_string().matches('\n').count() + 1
    }
}

/// Simple string with ASCII escape sequences in it
///
/// # Example
/// ```
/// use stybulate::{AsciiEscapedString, Unstyle};
/// let s = AsciiEscapedString::from("This is \x1b[1;31;44mbold red with blue background\x1b[0m");
/// assert_eq!("This is bold red with blue background", s.unstyle());
/// ```
pub struct AsciiEscapedString(String);

impl AsciiEscapedString {
    /// Constructs an `AsciiEscapedString` from a &str
    pub fn from(s: &str) -> Self {
        Self(String::from(s))
    }
}

impl Unstyle for String {
    fn unstyle(&self) -> String {
        self.clone()
    }
}

impl fmt::Display for AsciiEscapedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Unstyle for AsciiEscapedString {
    fn unstyle(&self) -> String {
        String::from(
            std::str::from_utf8(&strip_ansi_escapes::strip(self.0.clone()).unwrap()).unwrap(),
        )
    }
}
