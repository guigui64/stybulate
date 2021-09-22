use crate::unstyle::Unstyle;

/// The content of each cell of the table (either a string or a number)
pub enum Cell<'a> {
    /// Integer variant
    Int(i32),
    /// Float variant
    Float(f64),
    /// _Unstylable_ Text variant
    Text(Box<dyn Unstyle + 'a>),
}

impl<'a> Cell<'a> {
    /// Creates a Text Cell from a simple &str
    ///
    /// # Warning
    /// If the given `&str` contains ASCII escape sequences, they will mess with the generated
    /// layout. Use a `Box<dyn Unstyle>` like
    /// [`AsciiEscapedString`](struct.AsciiEscapedString.html).
    pub fn from(s: &str) -> Self {
        Self::Text(Box::new(String::from(s)))
    }

    /// Is it a number ?
    pub fn is_a_number(&self) -> bool {
        matches!(self, Self::Int(_) | Self::Float(_))
    }

    /// Returns the unstylable content if it is a Text Variant, None otherwise
    #[allow(clippy::borrowed_box)]
    pub fn to_unstylable(&self) -> Option<&Box<dyn Unstyle + 'a>> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the string representation of a number, None if it is a Text Variant
    pub fn to_string(&self) -> Option<String> {
        match self {
            Self::Int(i) => Some(i.to_string()),
            Self::Float(f) => Some(f.to_string()),
            _ => None,
        }
    }

    /// Same as [`to_string`](to_string) but formatted with a precision
    pub fn to_string_with_precision(&self, digits: usize) -> Option<String> {
        match self {
            Self::Int(i) => Some(format!("{:.prec$}", *i as f64, prec = digits)),
            Self::Float(f) => Some(format!("{:.prec$}", f, prec = digits)),
            _ => None,
        }
    }

    /// Number of digits after the dot in a float, 0 otherwise
    pub fn digits_len(&self) -> usize {
        if let Self::Float(f) = self {
            let s = f.to_string();
            if let Some(pos) = s.find('.') {
                s.len() - (pos + 1)
            } else {
                0
            }
        } else {
            0
        }
    }
}
