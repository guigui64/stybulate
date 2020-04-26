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

#[cfg(feature = "ansi_term_style")]
impl Unstyle for ansi_term::ANSIStrings<'_> {
    fn unstyle(&self) -> String {
        ansi_term::unstyle(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string() {
        let s = String::from("string with \nnewlines and some \ttabs\t\n\t");
        assert_eq!(s.to_string(), s.unstyle());
        assert_eq!(3, s.nb_of_lines());
    }

    #[test]
    fn ascii_escaped_string() {
        let s =
            AsciiEscapedString::from("This is \x1b[1;31;44mbold red with blue background\x1b[0m");
        assert_eq!("This is bold red with blue background", s.unstyle());
    }

    #[cfg(feature = "ansi_term_style")]
    #[test]
    fn ansi_term_unstyle() {
        use ansi_term::Colour::Red;
        use ansi_term::{ANSIString, ANSIStrings};

        let some_value = format!("{:b}", 42);
        let strings: &[ANSIString<'static>] =
            &[Red.paint("["), Red.bold().paint(some_value), Red.paint("]")];

        assert_eq!("[101010]", ANSIStrings(&strings).unstyle());
    }
}
