//! # Stybulate : Tabulate with Style!
//! This library creates tables in ASCII with styled borders
//!
//! # References
//! It was inspired by the Python package <https://pypi.org/project/tabulate/>
//!
//! # Examples
//! ```
//! use stybulate::{Table, Style, Cell, Headers};
//! let result = Table::new(
//!     Style::Fancy,
//!     vec![
//!         vec![Cell::from("answer"), Cell::Int(42)],
//!         vec![Cell::from("pi"), Cell::Float(3.1415)],
//!     ],
//!     Some(Headers::from(vec!["strings", "numbers"])),
//! ).tabulate();
//! let expected = vec![
//!     "╒═══════════╤═══════════╕",
//!     "│ strings   │   numbers │",
//!     "╞═══════════╪═══════════╡",
//!     "│ answer    │   42      │",
//!     "├───────────┼───────────┤",
//!     "│ pi        │    3.1415 │",
//!     "╘═══════════╧═══════════╛",
//! ].join("\n");
//! assert_eq!(expected, result);
//! ```

#![warn(missing_docs)]

use std::cmp;
use std::collections::HashMap;

use unicode_width::UnicodeWidthStr;

mod style;
pub use style::{Align, Style};

mod unstyle;
pub use unstyle::{AsciiEscapedString, Unstyle};

mod cell;
pub use cell::Cell;

// constants
const MIN_PADDING: usize = 2;

/// The Headers structure is a list of headers (per column)
/// # Example
/// ```
/// use stybulate::{AsciiEscapedString, Headers};
/// // simple example with only strings
/// let simple = Headers::from(vec!["foo", "bar"]);
/// // more elaborated example with a mix of a styled string and a simple string
/// let mut with_style = Headers::with_capacity(2);
/// with_style
///     .push(AsciiEscapedString::from("\x1b[1;35mfoo\x1b[0m bar"))
///     .push(String::from("baz"));
/// ```
#[derive(Default)]
pub struct Headers {
    headers: Vec<Box<dyn Unstyle>>,
}

impl Headers {
    /// Headers constructor: creates an empty header
    pub fn new() -> Self {
        Default::default()
    }

    /// Headers constructor from a vec of `&str`
    pub fn from(headers: Vec<&str>) -> Self {
        let mut unstyle_headers: Vec<Box<dyn Unstyle>> = Vec::with_capacity(headers.len());
        for header in headers.iter() {
            unstyle_headers.push(Box::new(String::from(*header)));
        }
        Self {
            headers: unstyle_headers,
        }
    }

    /// Headers constructor with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            headers: Vec::with_capacity(capacity),
        }
    }

    /// Add a header to the Headers
    pub fn push<H: Unstyle + 'static>(&mut self, header: H) -> &mut Self {
        self.headers.push(Box::new(header));
        self
    }

    fn len(&self) -> usize {
        self.headers.len()
    }

    #[allow(clippy::borrowed_box)]
    fn get(&self, i: usize) -> Option<&Box<dyn Unstyle>> {
        self.headers.get(i)
    }

    #[allow(clippy::borrowed_box)]
    fn to_ref_vec(&self) -> Vec<&Box<dyn Unstyle>> {
        self.headers.iter().map(|b| b).collect()
    }
}

/// The Table structure
/// # Example
/// ```
/// use::stybulate::*;
/// let mut table = Table::new(
///     Style::Fancy,
///     vec![
///         vec![Cell::from("answer"), Cell::Int(42)],
///         vec![Cell::from("pi"), Cell::Float(3.1415)],
///     ],
///     Some(Headers::from(vec!["strings", "numbers"]))
/// );
/// table.set_align(Align::Center, Align::Right);
/// ```
pub struct Table<'a> {
    style: Style,
    str_align: Align,
    num_align: Align,
    contents: Vec<Vec<Cell<'a>>>,
    headers: Option<Headers>,

    #[cfg(feature = "ansi_term_style")]
    border_style: Option<ansi_term::Style>,
}

impl<'a> Table<'a> {
    /// Table constructor with default alignments (`Align::Left` for strings and `Align::Decimal` for numbers)
    pub fn new(style: Style, contents: Vec<Vec<Cell<'a>>>, headers: Option<Headers>) -> Self {
        Self {
            style,
            str_align: Align::Left,
            num_align: Align::Decimal,
            #[cfg(feature = "ansi_term_style")]
            border_style: None,
            contents,
            headers,
        }
    }

    /// Set the table alignments (defaults are `Align::Left` for strings and `Align::Decimal` for numbers)
    /// # Panics
    /// Panics if str_align is equal to `Align::Decimal`
    pub fn set_align(&mut self, str_align: Align, num_align: Align) {
        if str_align == Align::Decimal {
            panic!("str_align should not be set to Decimal, only num_align can");
        }
        self.str_align = str_align;
        self.num_align = num_align;
    }

    #[cfg(feature = "ansi_term_style")]
    /// Set the borders style
    /// # Feature
    /// Needs feature `ansi_term_style`.
    pub fn set_border_style(&mut self, style: ansi_term::Style) {
        self.border_style = Some(style);
    }

    /// Creates the table as a `String`
    pub fn tabulate(&self) -> String {
        let style = &self.style;
        let headers = &self.headers;
        let contents = &self.contents;
        let str_align = &self.str_align;
        let num_align = &self.num_align;
        #[allow(unused_mut)]
        let mut fmt = style.to_format();
        #[cfg(feature = "ansi_term_style")]
        {
            if let Some(style) = self.border_style {
                fmt.apply_style(style);
            }
        }
        // number of columns
        let header_len = if let Some(h) = headers { h.len() } else { 0 };
        let col_nb = cmp::max(
            header_len,
            *contents.iter().map(Vec::len).max().get_or_insert(0),
        );
        // column specs = [0]: true if only made of numbers & [1]: digits offset
        let col_spec = get_col_specs(col_nb, contents);
        // max width of the content of each column
        let col_width = get_col_width(col_nb, headers, contents, &col_spec, num_align);
        // Build the lines
        let mut lines = vec![];
        // lineabove
        if !(headers.is_some() && fmt.hidelineaboveifheader) {
            if let Some(lineabove) = fmt.lineabove {
                lines.push(create_line(&lineabove, &col_width));
            }
        }
        if let Some(headers) = headers {
            // headerrow
            let headers: Vec<&Box<dyn Unstyle>> = headers.to_ref_vec();
            for data in create_data_lines(&headers, &str_align, &num_align, &col_width, &col_spec) {
                lines.push(create_data_line(&fmt.headerrow, col_nb, &data));
            }
            // linebelowheader
            if let Some(linebelowheader) = fmt.linebelowheader {
                lines.push(create_line(&linebelowheader, &col_width));
            }
        }
        // loop on contents
        for (i, content) in contents.iter().enumerate() {
            // linebetweenrows
            if i != 0 {
                if let Some(linebetweenrows) = fmt.linebetweenrows.clone() {
                    lines.push(create_line(&linebetweenrows, &col_width));
                }
            }
            // datarow
            let mut unstylable_content = Vec::with_capacity(content.len());
            let mut temp_unstyle_store = HashMap::new();
            let mut temp_strings_store = HashMap::new();
            for (col, cell) in content.iter().enumerate() {
                if let Some(u) = cell.to_unstylable() {
                    temp_unstyle_store.insert(col, u);
                } else {
                    temp_strings_store.insert(
                        col,
                        Box::new(cell.to_string_with_precision(col_spec[col].1).unwrap())
                            as Box<dyn Unstyle>,
                    );
                }
            }
            for col in 0..col_nb {
                if let Some(u) = temp_unstyle_store.get(&col) {
                    unstylable_content.push(*u);
                } else {
                    unstylable_content.push(temp_strings_store.get(&col).unwrap());
                }
            }
            for data in create_data_lines(
                &unstylable_content,
                &str_align,
                &num_align,
                &col_width,
                &col_spec,
            ) {
                lines.push(create_data_line(&fmt.datarow, col_nb, &data));
            }
        }
        // linebelow
        if !(headers.is_some() && fmt.hidelinebelowifheader) {
            if let Some(linebelow) = fmt.linebelow {
                lines.push(create_line(&linebelow, &col_width));
            }
        }
        // finally join all lines
        lines.join("\n")
    }
}

// --------------------------- Private ---------------------------

fn get_col_width<'a>(
    col_nb: usize,
    headers: &Option<Headers>,
    contents: &[Vec<Cell<'a>>],
    col_spec: &[(bool, usize)],
    num_align: &Align,
) -> Vec<usize> {
    let mut col_width = vec![0; col_nb];
    for col in 0..col_nb {
        let mut max = 0;
        if let Some(headers) = headers {
            if let Some(h) = headers.get(col) {
                max = *h
                    .unstyle()
                    .split('\n')
                    .map(|s| UnicodeWidthStr::width(&s as &str))
                    .max()
                    .get_or_insert(0)
                    + MIN_PADDING;
            }
        }
        for row in contents.iter() {
            if let Some(c) = row.get(col) {
                let width = if col_spec[col].0 /* a number */ && num_align == &Align::Decimal && col_spec[col].1 > 0
                {
                    c.to_string_with_precision(col_spec[col].1).unwrap().len()
                } else if let Some(u) = c.to_unstylable() {
                    *u.unstyle()
                        .split('\n')
                        .map(|s| UnicodeWidthStr::width(&s as &str))
                        .max()
                        .get_or_insert(0)
                } else {
                    c.to_string().unwrap().len()
                };
                max = cmp::max(width, max);
            }
        }
        col_width[col] = max;
    }
    col_width
}

fn get_col_specs<'a>(col_nb: usize, contents: &[Vec<Cell<'a>>]) -> Vec<(bool, usize)> {
    let mut col_spec = vec![(false, 0); col_nb];
    for (col, spec) in col_spec.iter_mut().enumerate().take(col_nb) {
        let mut max = 0;
        let mut all = true;
        for row in contents.iter() {
            if let Some(cell) = row.get(col) {
                if cell.is_a_number() {
                    max = cmp::max(max, cell.digits_len());
                } else {
                    all = false;
                }
            }
        }
        *spec = (all, max);
    }
    col_spec
}

fn create_line(line: &style::Line, col_width: &[usize]) -> String {
    (line.begin.clone()
        + &col_width
            .iter()
            .map(|w| line.hline.repeat(*w))
            .collect::<Vec<String>>()
            .join(&line.sep)
        + &line.end)
        .trim_end()
        .to_string()
}

fn create_data_line(row: &style::DataRow, col_nb: usize, content: &[String]) -> String {
    let mut v = Vec::with_capacity(col_nb);
    for col in 0..col_nb {
        if let Some(c) = content.get(col) {
            v.push(String::from(c));
        } else {
            v.push(String::from(""));
        }
    }
    (row.begin.clone() + &v.join(&row.sep) + &row.end)
        .trim_end()
        .to_string()
}

#[allow(clippy::borrowed_box)]
fn format_unstylable<'a>(
    word: &Box<dyn Unstyle + 'a>,
    line_idx: usize,
    align: &Align,
    width: usize,
) -> String {
    if let Some(unstyled_word) = word.unstyle().split('\n').nth(line_idx) {
        let word = word.to_string();
        let word = word
            .split('\n')
            .nth(line_idx)
            .expect("unstyled word can't have more \\n than styled one");
        let width = width - (unstyled_word.len() - UnicodeWidthStr::width(&unstyled_word as &str));
        let formatted = match align {
            Align::Right => format!("{:>width$}", unstyled_word, width = width),
            Align::Left => format!("{:<width$}", unstyled_word, width = width),
            Align::Center => format!("{:^width$}", unstyled_word, width = width),
            Align::Decimal => {
                let mut out = format!("{:>width$}", unstyled_word, width = width);
                if let Some(dot) = out.rfind('.') {
                    if out[(dot + 1)..].bytes().all(|c| c == b'0') {
                        out.replace_range(dot.., &" ".repeat(out.len() - dot));
                    }
                }
                out
            }
        };
        if unstyled_word != word {
            formatted.replace(&unstyled_word, &word)
        } else {
            formatted
        }
    } else {
        " ".repeat(width)
    }
}

#[allow(clippy::borrowed_box)]
fn create_data_lines<'a>(
    content: &[&Box<dyn Unstyle + 'a>],
    str_align: &Align,
    num_align: &Align,
    col_width: &[usize],
    col_spec: &[(bool, usize)],
) -> Vec<Vec<String>> {
    let lines_nb = content.iter().map(|u| u.nb_of_lines()).max().unwrap();
    let mut lines = Vec::with_capacity(lines_nb);
    for i in 0..lines_nb {
        let formatted: Vec<_> = content
            .iter()
            .enumerate()
            .map(|(col, text)| {
                let align = if col_spec[col].0 {
                    // numbers only
                    &num_align
                } else {
                    // strings only
                    &str_align
                };
                format_unstylable(text, i, &align, col_width[col])
            })
            .collect();
        lines.push(formatted);
    }
    lines
}

// --------------------------- Tests ---------------------------

#[cfg(test)]
mod tests {

    use super::*;

    fn headerless(style: Style) -> Table<'static> {
        Table::new(
            style,
            vec![
                vec![Cell::from("spam"), Cell::Float(41.9999)],
                vec![Cell::from("eggs"), Cell::Int(451)],
            ],
            None,
        )
    }

    fn table(style: Style) -> Table<'static> {
        Table::new(
            style.clone(),
            headerless(style).contents,
            Some(Headers::from(vec!["strings", "numbers"])),
        )
    }

    fn multiline_headerless(style: Style) -> Table<'static> {
        let mut table = Table::new(
            style,
            vec![
                vec![Cell::from("foo bar\nbaz\nbau"), Cell::from("hello")],
                vec![Cell::from(""), Cell::from("multiline\nworld")],
            ],
            None,
        );
        table.set_align(Align::Center, Align::Right);
        table
    }

    fn multiline(style: Style) -> Table<'static> {
        Table::new(
            style,
            vec![vec![Cell::Int(2), Cell::from("foo\nbar")]],
            Some(Headers::from(vec!["more\nspam eggs", "more spam\n& eggs"])),
        )
    }

    fn multiline_empty_cells(style: Style) -> Table<'static> {
        Table::new(
            style.clone(),
            vec![
                vec![Cell::Int(1), Cell::from(""), Cell::from("")],
                vec![
                    Cell::Int(2),
                    Cell::from("very long data"),
                    Cell::from("fold\nthis"),
                ],
            ],
            Some(Headers::from(vec!["hdr", "data", "fold"])),
        )
    }

    fn multiline_empty_cells_headerless(style: Style) -> Table<'static> {
        Table::new(
            style,
            vec![
                vec![Cell::Int(0), Cell::from(""), Cell::from("")],
                vec![Cell::Int(1), Cell::from(""), Cell::from("")],
                vec![
                    Cell::Int(2),
                    Cell::from("very long data"),
                    Cell::from("fold\nthis"),
                ],
            ],
            None,
        )
    }

    #[test]
    fn plain() {
        //Output: plain with headers
        let result = table(Style::Plain).tabulate();
        let expected = vec![
            "strings      numbers",
            "spam         41.9999",
            "eggs        451",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_headerless() {
        //Output: plain without headers
        let result = headerless(Style::Plain).tabulate();
        let expected = vec!["spam   41.9999", "eggs  451"].join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_headerless() {
        //Output: plain with multiline cells without headers
        let result = multiline_headerless(Style::Plain).tabulate();
        let expected = vec![
            "foo bar    hello",
            "  baz",
            "  bau",
            "         multiline",
            "           world",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline() {
        //Output: plain with multiline cells with headers
        let result = multiline(Style::Plain).tabulate();
        let expected = vec![
            "       more  more spam",
            "  spam eggs  & eggs",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_with_empty_cells() {
        //Output: plain with multiline cells and empty cells with headers
        let result = multiline_empty_cells(Style::Plain).tabulate();
        let expected = vec![
            "  hdr  data            fold",
            "    1",
            "    2  very long data  fold",
            "                       this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_with_empty_cells_headerless() {
        //Output: plain with multiline cells and empty cells without headers
        let result = multiline_empty_cells_headerless(Style::Plain).tabulate();
        let expected = vec![
            "0",
            "1",
            "2  very long data  fold",
            "                   this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple() {
        //Output: simple with headers
        let result = table(Style::Simple).tabulate();
        let expected = vec![
            "strings      numbers",
            "---------  ---------",
            "spam         41.9999",
            "eggs        451",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_2() {
        //Output: simple with multiline cells
        let mut table = Table::new(
            Style::Simple,
            vec![
                vec![Cell::from("foo"), Cell::from("bar")],
                vec![Cell::from("spam"), Cell::from("multiline\nworld")],
            ],
            Some(Headers::from(vec!["key", "value"])),
        );
        table.set_align(Align::Center, Align::Right);
        let result = table.tabulate();
        let expected = vec![
            " key     value",
            "-----  ---------",
            " foo      bar",
            "spam   multiline",
            "         world",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_headerless() {
        //Output: simple without headers
        let result = headerless(Style::Simple).tabulate();
        let expected = vec![
            "----  --------",
            "spam   41.9999",
            "eggs  451",
            "----  --------",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_headerless() {
        //Output: simple with multiline cells without headers
        let result = multiline_headerless(Style::Simple).tabulate();
        let expected = vec![
            "-------  ---------",
            "foo bar    hello",
            "  baz",
            "  bau",
            "         multiline",
            "           world",
            "-------  ---------",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline() {
        //Output: simple with multiline cells with headers
        let result = multiline(Style::Simple).tabulate();
        let expected = vec![
            "       more  more spam",
            "  spam eggs  & eggs",
            "-----------  -----------",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_ascii_escaped() {
        //Output: simple with multiline headers and colors (ascii escape)
        let mut headers = Headers::with_capacity(2);
        headers
            .push(AsciiEscapedString::from("more\nspam \x1b[31meggs\x1b[0m"))
            .push(String::from("more spam\n& eggs"));
        let result = Table::new(
            Style::Simple,
            vec![vec![Cell::Int(2), Cell::from("foo\nbar")]],
            Some(headers),
        )
        .tabulate();
        let expected = vec![
            "       more  more spam",
            "  spam \x1b[31meggs\x1b[0m  & eggs",
            "-----------  -----------",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_with_empty_cells() {
        //Output: simple with multiline cells and empty cells with headers
        let result = multiline_empty_cells(Style::Simple).tabulate();
        let expected = vec![
            "  hdr  data            fold",
            "-----  --------------  ------",
            "    1",
            "    2  very long data  fold",
            "                       this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_with_empty_cells_headerless() {
        //Output: simple with multiline cells and empty cells without headers
        let result = multiline_empty_cells_headerless(Style::Simple).tabulate();
        let expected = vec![
            "-  --------------  ----",
            "0",
            "1",
            "2  very long data  fold",
            "                   this",
            "-  --------------  ----",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn github() {
        //Output: github with headers
        let result = table(Style::Github).tabulate();
        let expected = vec![
            "| strings   |   numbers |",
            "|-----------|-----------|",
            "| spam      |   41.9999 |",
            "| eggs      |  451      |",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid() {
        //Output: grid with headers
        let result = table(Style::Grid).tabulate();
        let expected = vec![
            "+-----------+-----------+",
            "| strings   |   numbers |",
            "+===========+===========+",
            "| spam      |   41.9999 |",
            "+-----------+-----------+",
            "| eggs      |  451      |",
            "+-----------+-----------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_wide_characters() {
        //Output: grid with wide characters in headers
        let headers = Headers::from(vec!["strings", "配列"]);
        let contents = vec![
            vec![
                Cell::from("Ответ на главный вопрос жизни, вселенной и всего такого"),
                Cell::Int(42),
            ],
            vec![Cell::from("pi"), Cell::Float(3.1415)],
        ];
        let result = Table::new(Style::Grid, contents, Some(headers)).tabulate();
        let expected = vec![
            "+---------------------------------------------------------+---------+",
            "| strings                                                 |    配列 |",
            "+=========================================================+=========+",
            "| Ответ на главный вопрос жизни, вселенной и всего такого | 42      |",
            "+---------------------------------------------------------+---------+",
            "| pi                                                      |  3.1415 |",
            "+---------------------------------------------------------+---------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_headerless() {
        //Output: grid without headers
        let result = headerless(Style::Grid).tabulate();
        let expected = vec![
            "+------+----------+",
            "| spam |  41.9999 |",
            "+------+----------+",
            "| eggs | 451      |",
            "+------+----------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_headerless() {
        //Output: grid with multiline cells without headers
        let result = multiline_headerless(Style::Grid).tabulate();
        let expected = vec![
            "+---------+-----------+",
            "| foo bar |   hello   |",
            "|   baz   |           |",
            "|   bau   |           |",
            "+---------+-----------+",
            "|         | multiline |",
            "|         |   world   |",
            "+---------+-----------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline() {
        //Output: grid with multiline cells with headers
        let result = multiline(Style::Grid).tabulate();
        let expected = vec![
            "+-------------+-------------+",
            "|        more | more spam   |",
            "|   spam eggs | & eggs      |",
            "+=============+=============+",
            "|           2 | foo         |",
            "|             | bar         |",
            "+-------------+-------------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_with_empty_cells() {
        //Output: grid with multiline cells and empty cells with headers
        let result = multiline_empty_cells(Style::Grid).tabulate();
        let expected = vec![
            "+-------+----------------+--------+",
            "|   hdr | data           | fold   |",
            "+=======+================+========+",
            "|     1 |                |        |",
            "+-------+----------------+--------+",
            "|     2 | very long data | fold   |",
            "|       |                | this   |",
            "+-------+----------------+--------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_with_empty_cells_headerless() {
        //Output: grid with multiline cells and empty cells without headers
        let result = multiline_empty_cells_headerless(Style::Grid).tabulate();
        let expected = vec![
            "+---+----------------+------+",
            "| 0 |                |      |",
            "+---+----------------+------+",
            "| 1 |                |      |",
            "+---+----------------+------+",
            "| 2 | very long data | fold |",
            "|   |                | this |",
            "+---+----------------+------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid() {
        //Output: fancy_grid with headers
        let result = table(Style::Fancy).tabulate();
        let expected = vec![
            "╒═══════════╤═══════════╕",
            "│ strings   │   numbers │",
            "╞═══════════╪═══════════╡",
            "│ spam      │   41.9999 │",
            "├───────────┼───────────┤",
            "│ eggs      │  451      │",
            "╘═══════════╧═══════════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_headerless() {
        //Output: fancy_grid without headers
        let result = headerless(Style::Fancy).tabulate();
        let expected = vec![
            "╒══════╤══════════╕",
            "│ spam │  41.9999 │",
            "├──────┼──────────┤",
            "│ eggs │ 451      │",
            "╘══════╧══════════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_headerless() {
        //Output: fancy_grid with multiline cells without headers
        let result = multiline_headerless(Style::Fancy).tabulate();
        let expected = vec![
            "╒═════════╤═══════════╕",
            "│ foo bar │   hello   │",
            "│   baz   │           │",
            "│   bau   │           │",
            "├─────────┼───────────┤",
            "│         │ multiline │",
            "│         │   world   │",
            "╘═════════╧═══════════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline() {
        //Output: fancy_grid with multiline cells with headers
        let result = multiline(Style::Fancy).tabulate();
        let expected = vec![
            "╒═════════════╤═════════════╕",
            "│        more │ more spam   │",
            "│   spam eggs │ & eggs      │",
            "╞═════════════╪═════════════╡",
            "│           2 │ foo         │",
            "│             │ bar         │",
            "╘═════════════╧═════════════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_with_empty_cells() {
        //Output: fancy_grid with multiline cells and empty cells with headers
        let result = multiline_empty_cells(Style::Fancy).tabulate();
        let expected = vec![
            "╒═══════╤════════════════╤════════╕",
            "│   hdr │ data           │ fold   │",
            "╞═══════╪════════════════╪════════╡",
            "│     1 │                │        │",
            "├───────┼────────────────┼────────┤",
            "│     2 │ very long data │ fold   │",
            "│       │                │ this   │",
            "╘═══════╧════════════════╧════════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_with_empty_cells_headerless() {
        //Output: fancy_grid with multiline cells and empty cells without headers
        let result = multiline_empty_cells_headerless(Style::Fancy).tabulate();
        let expected = vec![
            "╒═══╤════════════════╤══════╕",
            "│ 0 │                │      │",
            "├───┼────────────────┼──────┤",
            "│ 1 │                │      │",
            "├───┼────────────────┼──────┤",
            "│ 2 │ very long data │ fold │",
            "│   │                │ this │",
            "╘═══╧════════════════╧══════╛",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto() {
        //Output: presto with headers
        let result = table(Style::Presto).tabulate();
        let expected = vec![
            " strings   |   numbers",
            "-----------+-----------",
            " spam      |   41.9999",
            " eggs      |  451",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_headerless() {
        //Output: presto without headers
        let result = headerless(Style::Presto).tabulate();
        let expected = vec![" spam |  41.9999", " eggs | 451"].join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_headerless() {
        //Output: presto with multiline cells without headers
        let result = multiline_headerless(Style::Presto).tabulate();
        let expected = vec![
            " foo bar |   hello",
            "   baz   |",
            "   bau   |",
            "         | multiline",
            "         |   world",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline() {
        //Output: presto with multiline cells with headers
        let result = multiline(Style::Presto).tabulate();
        let expected = vec![
            "        more | more spam",
            "   spam eggs | & eggs",
            "-------------+-------------",
            "           2 | foo",
            "             | bar",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_with_empty_cells() {
        //Output: presto with multiline cells and empty cells with headers
        let result = multiline_empty_cells(Style::Presto).tabulate();
        let expected = vec![
            "   hdr | data           | fold",
            "-------+----------------+--------",
            "     1 |                |",
            "     2 | very long data | fold",
            "       |                | this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_with_empty_cells_headerless() {
        //Output: presto with multiline cells and empty cells without headers
        let result = multiline_empty_cells_headerless(Style::Presto).tabulate();
        let expected = vec![
            " 0 |                |",
            " 1 |                |",
            " 2 | very long data | fold",
            "   |                | this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid() {
        let result = table(Style::FancyGithub).tabulate();
        let expected = vec![
            "│ strings   │   numbers │",
            "├───────────┼───────────┤",
            "│ spam      │   41.9999 │",
            "│ eggs      │  451      │",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_headerless() {
        let result = headerless(Style::FancyGithub).tabulate();
        let expected = vec!["│ spam │  41.9999 │", "│ eggs │ 451      │"].join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_headerless() {
        let result = multiline_headerless(Style::FancyGithub).tabulate();
        let expected = vec![
            "│ foo bar │   hello   │",
            "│   baz   │           │",
            "│   bau   │           │",
            "│         │ multiline │",
            "│         │   world   │",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline() {
        let result = multiline(Style::FancyGithub).tabulate();
        let expected = vec![
            "│        more │ more spam   │",
            "│   spam eggs │ & eggs      │",
            "├─────────────┼─────────────┤",
            "│           2 │ foo         │",
            "│             │ bar         │",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_with_empty_cells() {
        let result = multiline_empty_cells(Style::FancyGithub).tabulate();
        let expected = vec![
            "│   hdr │ data           │ fold   │",
            "├───────┼────────────────┼────────┤",
            "│     1 │                │        │",
            "│     2 │ very long data │ fold   │",
            "│       │                │ this   │",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_with_empty_cells_headerless() {
        let result = multiline_empty_cells_headerless(Style::FancyGithub).tabulate();
        let expected = vec![
            "│ 0 │                │      │",
            "│ 1 │                │      │",
            "│ 2 │ very long data │ fold │",
            "│   │                │ this │",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid() {
        let result = table(Style::FancyPresto).tabulate();
        let expected = vec![
            "strings   │   numbers",
            "──────────┼──────────",
            "spam      │   41.9999",
            "eggs      │  451",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_headerless() {
        let result = headerless(Style::FancyPresto).tabulate();
        let expected = vec!["spam │  41.9999", "eggs │ 451"].join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_headerless() {
        let result = multiline_headerless(Style::FancyPresto).tabulate();
        let expected = vec![
            "foo bar │   hello",
            "  baz   │",
            "  bau   │",
            "        │ multiline",
            "        │   world",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline() {
        let result = multiline(Style::FancyPresto).tabulate();
        let expected = vec![
            "       more │ more spam",
            "  spam eggs │ & eggs",
            "────────────┼────────────",
            "          2 │ foo",
            "            │ bar",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_with_empty_cells() {
        let result = multiline_empty_cells(Style::FancyPresto).tabulate();
        let expected = vec![
            "  hdr │ data           │ fold",
            "──────┼────────────────┼───────",
            "    1 │                │",
            "    2 │ very long data │ fold",
            "      │                │ this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_with_empty_cells_headerless() {
        let result = multiline_empty_cells_headerless(Style::FancyPresto).tabulate();
        let expected = vec![
            "0 │                │",
            "1 │                │",
            "2 │ very long data │ fold",
            "  │                │ this",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[cfg(feature = "ansi_term_style")]
    #[test]
    fn ansi_term_colored_content<'a>() {
        use ansi_term::Colour::Red;
        use ansi_term::{ANSIString, ANSIStrings};

        let some_value = format!("{:b}", 42);
        let strings: &[ANSIString<'a>] =
            &[Red.paint("["), Red.bold().paint(some_value), Red.paint("]")];

        let result = Table::new(
            Style::Grid,
            vec![vec![
                Cell::Int(42),
                Cell::Text(Box::new(ANSIStrings(&strings))),
            ]],
            Some(Headers::from(vec!["Int", "Colored binary"])),
        )
        .tabulate();

        let expected = vec![
            "+-------+------------------+",
            "|   Int | Colored binary   |",
            "+=======+==================+",
            "|    42 | \u{1b}[31m[\u{1b}[1m101010\u{1b}[0m\u{1b}[31m]\u{1b}[0m         |",
            "+-------+------------------+",
        ]
        .join("\n");
        assert_eq!(expected, result);
    }

    #[cfg(feature = "ansi_term_style")]
    #[test]
    fn ansi_styled_borders() {
        let mut table = table(Style::Fancy);
        table.set_border_style(ansi_term::Color::Green.bold());
        let result = table.tabulate();
        let expected = vec![
            "\u{1b}[1;32m╒═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╤═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╕\u{1b}[0m",
            "\u{1b}[1;32m│ \u{1b}[0mstrings  \u{1b}[1;32m │ \u{1b}[0m  numbers\u{1b}[1;32m │\u{1b}[0m",
            "\u{1b}[1;32m╞═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╪═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╡\u{1b}[0m",
            "\u{1b}[1;32m│ \u{1b}[0mspam     \u{1b}[1;32m │ \u{1b}[0m  41.9999\u{1b}[1;32m │\u{1b}[0m",
            "\u{1b}[1;32m├─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─┼─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─\u{1b}[0m\u{1b}[1;32m─┤\u{1b}[0m",
            "\u{1b}[1;32m│ \u{1b}[0meggs     \u{1b}[1;32m │ \u{1b}[0m 451     \u{1b}[1;32m │\u{1b}[0m",
            "\u{1b}[1;32m╘═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╧═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═\u{1b}[0m\u{1b}[1;32m═╛\u{1b}[0m"
        ].join("\n");
        assert_eq!(expected, result);
    }
}
