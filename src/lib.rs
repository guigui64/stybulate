//! # Stybulate : Tabulate with Style!
//! This library creates tables in ASCII with styled borders
//!
//! # References
//! It was inspired by the Python package <https://pypi.org/project/tabulate/>
//!
//! # Examples
//! ```
//! use stybulate::{tabulate, Style, Cell};
//! let headers = vec!["strings", "numbers"];
//! let contents = vec![
//!     vec![Cell::Text("answer"), Cell::Int(42)],
//!     vec![Cell::Text("pi"), Cell::Float(3.1415)],
//! ];
//! let expected = vec![
//!     "╒═══════════╤═══════════╕",
//!     "│ strings   │   numbers │",
//!     "╞═══════════╪═══════════╡",
//!     "│ answer    │   42      │",
//!     "├───────────┼───────────┤",
//!     "│ pi        │    3.1415 │",
//!     "╘═══════════╧═══════════╛",
//! ].join("\n");
//! let table = tabulate(Style::Fancy, contents, headers);
//! assert_eq!(expected, table);
//! ```

use std::cmp;

use unicode_width::UnicodeWidthStr;

// constants
const MIN_PADDING: usize = 2;

// --------------------------- Public ---------------------------

/// The style of the table
///
/// Examples shown will have a header line and two content lines
pub enum Style {
    /// ```text
    /// item      qty
    /// spam       42
    /// eggs      451
    /// ```
    Plain,
    /// ```text
    /// item      qty
    /// ------  -----
    /// spam       42
    /// eggs      451
    /// ```
    Simple,
    /// ```text
    /// | item   |   qty |
    /// |--------|-------|
    /// | spam   |    42 |
    /// | eggs   |   451 |
    /// ```
    Github,
    /// ```text
    /// +--------+-------+
    /// | item   |   qty |
    /// +========+=======+
    /// | spam   |    42 |
    /// +--------+-------+
    /// | eggs   |   451 |
    /// +--------+-------+
    /// ```
    Grid,
    /// ```text
    /// ╒════════╤═══════╕
    /// │ item   │   qty │
    /// ╞════════╪═══════╡
    /// │ spam   │    42 │
    /// ├────────┼───────┤
    /// │ eggs   │   451 │
    /// ╘════════╧═══════╛
    /// ```
    Fancy,
    /// ```text
    /// item   |   qty
    ///--------+-------
    /// spam   |    42
    /// eggs   |   451
    /// ```
    Presto,
    /// ```text
    /// │ item   │   qty │
    /// ├────────┼───────┤
    /// │ spam   │    42 │
    /// │ eggs   │   451 │
    /// ```
    FancyGithub,
    /// ```text
    /// item   │   qty
    /// ───────┼──────
    /// spam   │    42
    /// eggs   │   451
    /// ```
    FancyPresto,
}

/// The column alignments
///
/// Numbers are only considered as non-text when align is `Decimal`.
#[derive(PartialEq)]
pub enum Align {
    /// Left aligned text
    Left,
    /// Centered text
    Center,
    /// Right aligned text
    Right,
    /// Numbers right aligned and aligned with their fractional dot
    Decimal,
}

/// The content of each cell of the table (either a string or a number)
pub enum Cell<'a> {
    Int(i32),
    Float(f64),
    Text(&'a str),
}

// ---------------------------------- API ----------------------------------

/// Tabulate with default alignment (left for strings and decimal for numbers)
pub fn tabulate(style: Style, contents: Vec<Vec<Cell>>, headers: Vec<&str>) -> String {
    tabulate_with_align(style, contents, headers, Align::Left, Align::Decimal)
}

/// Tabulate
/// # Panics
/// Panics if str_align is equal to `Decimal`.
pub fn tabulate_with_align(
    style: Style,
    contents: Vec<Vec<Cell>>,
    headers: Vec<&str>,
    str_align: Align,
    num_align: Align,
) -> String {
    if str_align == Align::Decimal {
        panic!("str_align should not be set to Decimal, only num_align can");
    }
    let fmt = style.to_format();
    // number of columns
    let col_nb = cmp::max(
        headers.len(),
        *contents.iter().map(Vec::len).max().get_or_insert(0),
    );
    // column specs = [0]: true if only made of numbers & [1]: digits offset
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
    // max width of the content of each column
    let mut col_width = vec![0; col_nb];
    for col in 0..col_nb {
        let mut max = 0;
        if let Some(h) = headers.get(col) {
            max = *h
                .split('\n')
                .map(strip)
                .map(|s| UnicodeWidthStr::width(&s as &str))
                .max()
                .get_or_insert(0)
                + MIN_PADDING;
        }
        for row in contents.iter() {
            if let Some(c) = row.get(col) {
                let width = if col_spec[col].0 && num_align == Align::Decimal && col_spec[col].1 > 0
                {
                    c.to_str_with_digits(col_spec[col].1).len()
                } else {
                    *c.to_str()
                        .split('\n')
                        .map(strip)
                        .map(|s| UnicodeWidthStr::width(&s as &str))
                        .max()
                        .get_or_insert(0)
                };
                max = cmp::max(width, max);
            }
        }
        col_width[col] = max;
    }
    // Build the lines
    let mut lines = vec![];
    let hasheader = !headers.is_empty();
    // lineabove
    if !(hasheader && fmt.hidelineaboveifheader) {
        if let Some(lineabove) = fmt.lineabove {
            lines.push(create_line(&lineabove, &col_width));
        }
    }
    if hasheader {
        // headerrow
        let headers = line_to_multi(headers.iter().map(|s| (*s).to_string()).collect());
        for headerline in headers.iter() {
            lines.push(create_data_line(
                &fmt.headerrow,
                col_nb,
                &col_width,
                &col_spec,
                &num_align,
                &str_align,
                &headerline.iter().map(|s| &s as &str).collect::<Vec<&str>>()[..],
            ));
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
        {
            // convert cells to string
            let content = line_to_multi(
                content
                    .iter()
                    .enumerate()
                    .map(|(col, cell)| cell.to_str_with_digits(col_spec[col].1))
                    .collect(),
            );
            for contentline in content.iter() {
                lines.push(create_data_line(
                    &fmt.datarow,
                    col_nb,
                    &col_width,
                    &col_spec,
                    &num_align,
                    &str_align,
                    &contentline
                        .iter()
                        .map(|s| &s as &str)
                        .collect::<Vec<&str>>()[..],
                ));
            }
        }
    }
    // linebelow
    if !(hasheader && fmt.hidelinebelowifheader) {
        if let Some(linebelow) = fmt.linebelow {
            lines.push(create_line(&linebelow, &col_width));
        }
    }
    // finally join all lines
    lines.join("\n")
}

// --------------------------- Private ---------------------------

impl Style {
    fn to_format(&self) -> TableFormat {
        let basicrow = DataRow::new("", "  ", "");
        let emptyformat = TableFormat {
            lineabove: None,
            linebelowheader: None,
            linebetweenrows: None,
            linebelow: None,
            headerrow: basicrow.clone(),
            datarow: basicrow,
            padding: 0,
            hidelineaboveifheader: false,
            hidelinebelowifheader: false,
        };
        let basicline = Line::new("", "-", "  ", "");
        let piperow = DataRow::new("| ", " | ", " |");
        let single_line = Line::new("", "─", "─┼─", "");
        let single_line_with_ends = Line::new("├─", "─", "─┼─", "─┤");
        let row_line = DataRow::new("", " │ ", "");
        let row_line_with_ends = DataRow::new("│ ", " │ ", " │");
        match self {
            Self::Plain => emptyformat,
            Self::Simple => TableFormat {
                lineabove: Some(basicline.clone()),
                linebelowheader: Some(basicline.clone()),
                linebelow: Some(basicline),
                hidelineaboveifheader: true,
                hidelinebelowifheader: true,
                ..emptyformat
            },
            Self::Github => {
                let line = Line::new("|-", "-", "-|-", "-|");
                TableFormat {
                    lineabove: Some(line.clone()),
                    linebelowheader: Some(line),
                    headerrow: piperow.clone(),
                    datarow: piperow,
                    padding: 1,
                    hidelineaboveifheader: true,
                    ..emptyformat
                }
            }
            Self::Grid => {
                let line = Line::new("+-", "-", "-+-", "-+");
                TableFormat {
                    lineabove: Some(line.clone()),
                    linebelowheader: Some(Line::new("+=", "=", "=+=", "=+")),
                    linebetweenrows: Some(line.clone()),
                    linebelow: Some(line),
                    headerrow: piperow.clone(),
                    datarow: piperow,
                    padding: 1,
                    ..emptyformat
                }
            }
            Self::Fancy => TableFormat {
                lineabove: Some(Line::new("╒═", "═", "═╤═", "═╕")),
                linebelowheader: Some(Line::new("╞═", "═", "═╪═", "═╡")),
                linebetweenrows: Some(single_line_with_ends),
                linebelow: Some(Line::new("╘═", "═", "═╧═", "═╛")),
                headerrow: row_line_with_ends.clone(),
                datarow: row_line_with_ends,
                padding: 1,
                ..emptyformat
            },
            Self::Presto => {
                let row = DataRow::new(" ", " | ", " ");
                TableFormat {
                    linebelowheader: Some(Line::new("-", "-", "-+-", "-")),
                    headerrow: row.clone(),
                    datarow: row,
                    padding: 1,
                    ..emptyformat
                }
            }
            Self::FancyGithub => TableFormat {
                linebelowheader: Some(single_line_with_ends),
                headerrow: row_line_with_ends.clone(),
                datarow: row_line_with_ends,
                padding: 1,
                ..emptyformat
            },
            Self::FancyPresto => TableFormat {
                linebelowheader: Some(single_line),
                headerrow: row_line.clone(),
                datarow: row_line,
                padding: 1,
                ..emptyformat
            },
        }
    }
}

impl Cell<'_> {
    fn is_a_number(&self) -> bool {
        match self {
            Self::Int(_) | Self::Float(_) => true,
            _ => false,
        }
    }

    fn to_str(&self) -> String {
        match self {
            Self::Text(s) => (*s).to_string(),
            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
        }
    }

    fn to_str_with_digits(&self, digits: usize) -> String {
        match self {
            Self::Text(s) => (*s).to_string(),
            Self::Int(i) => format!("{:.prec$}", *i as f64, prec = digits),
            Self::Float(f) => format!("{:.prec$}", f, prec = digits),
        }
    }

    fn digits_len(&self) -> usize {
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

fn create_line(line: &Line, col_width: &[usize]) -> String {
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

fn create_data_line(
    row: &DataRow,
    col_nb: usize,
    col_width: &[usize],
    col_spec: &[(bool, usize)],
    num_align: &Align,
    str_align: &Align,
    content: &[&str],
) -> String {
    (row.begin.clone()
        + &(0..col_nb)
            .map(|col| {
                (
                    content.get(col).unwrap_or(&""),
                    col_width[col],
                    col_spec[col],
                )
            })
            .map(|(word, width, col_spec)| {
                let align = if col_spec.0 {
                    // numbers only
                    &num_align
                } else {
                    // strings only
                    &str_align
                };
                let stripped_word = strip(word);
                let width =
                    width - (stripped_word.len() - UnicodeWidthStr::width(&stripped_word as &str));
                let formatted = match *align {
                    Align::Right => format!("{:>width$}", stripped_word, width = width),
                    Align::Left => format!("{:<width$}", stripped_word, width = width),
                    Align::Center => format!("{:^width$}", stripped_word, width = width),
                    Align::Decimal => {
                        let mut out = format!("{:>width$}", stripped_word, width = width);
                        if let Some(dot) = out.rfind('.') {
                            if out[(dot + 1)..].bytes().all(|c| c == b'0') {
                                out.replace_range(dot.., &" ".repeat(out.len() - dot));
                            }
                        }
                        out
                    }
                };
                if &stripped_word != word {
                    formatted.replace(&stripped_word, word)
                } else {
                    formatted
                }
            })
            .collect::<Vec<String>>()
            .join(&row.sep)
        + &row.end)
        .trim_end()
        .to_string()
}

fn line_to_multi(mut line: Vec<String>) -> Vec<Vec<String>> {
    let mut multi = vec![];

    if line.iter().any(|s| s.contains('\n')) {
        loop {
            let mut rest = vec![];
            for c in line.iter_mut() {
                if let Some(idx) = c.find('\n') {
                    let next = c.split_off(idx);
                    rest.push(String::from(&next[1..]));
                } else {
                    rest.push(String::from(""));
                }
            }
            multi.push(line);
            if rest.iter().all(|s| s == &String::from("")) {
                break;
            } else {
                line = rest;
            }
        }
    } else {
        multi.push(line);
    }

    multi
}

fn strip(s: &str) -> String {
    String::from(std::str::from_utf8(&strip_ansi_escapes::strip(s).unwrap()).unwrap())
}

#[derive(Clone)]
struct Line {
    begin: String,
    hline: String,
    sep: String,
    end: String,
}
impl Line {
    fn new(begin: &str, hline: &str, sep: &str, end: &str) -> Self {
        Self {
            begin: String::from(begin),
            hline: String::from(hline),
            sep: String::from(sep),
            end: String::from(end),
        }
    }
}

#[derive(Clone)]
struct DataRow {
    begin: String,
    sep: String,
    end: String,
}
impl DataRow {
    fn new(begin: &str, sep: &str, end: &str) -> Self {
        Self {
            begin: String::from(begin),
            sep: String::from(sep),
            end: String::from(end),
        }
    }
}

// A table is structured like so:
//     --- lineabove ---------
//         headerrow
//     --- linebelowheader ---
//         datarow
//     --- linebetweenrows ---
//     ... (more datarows) ...
//     --- linebewteenrows ---
//         last datarow
//     --- linebelow ---------
#[derive(Clone)]
struct TableFormat {
    lineabove: Option<Line>,
    linebelowheader: Option<Line>,
    linebetweenrows: Option<Line>,
    linebelow: Option<Line>,
    headerrow: DataRow,
    datarow: DataRow,
    padding: u32,
    hidelineaboveifheader: bool,
    hidelinebelowifheader: bool,
}

// --------------------------- Tests ---------------------------

#[cfg(test)]
mod tests {

    use super::*;

    struct TestedInput<'a> {
        contents: Vec<Vec<Cell<'a>>>,
        headers: Vec<&'a str>,
    }

    impl<'a> TestedInput<'a> {
        fn default() -> Self {
            Self {
                contents: vec![
                    vec![Cell::Text("spam"), Cell::Float(41.9999)],
                    vec![Cell::Text("eggs"), Cell::Int(451)],
                ],
                headers: vec!["strings", "numbers"],
            }
        }

        fn with_contents(contents: Vec<Vec<Cell<'a>>>) -> Self {
            Self {
                contents,
                headers: vec![],
            }
        }

        fn new(contents: Vec<Vec<Cell<'a>>>, headers: Vec<&'a str>) -> Self {
            Self { contents, headers }
        }
    }

    #[test]
    fn plain() {
        //Output: plain with headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "strings      numbers",
            "spam         41.9999",
            "eggs        451",
        ]
        .join("\n");
        let result = tabulate(Style::Plain, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_headerless() {
        //Output: plain without headers
        let tested_input = TestedInput::default();
        let expected = vec!["spam   41.9999", "eggs  451"].join("\n");
        let result = tabulate(Style::Plain, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_headerless() {
        //Output: plain with multiline cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
        let expected = vec![
            "foo bar    hello",
            "  baz",
            "  bau",
            "         multiline",
            "           world",
        ]
        .join("\n");
        let result = tabulate_with_align(
            Style::Plain,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline() {
        //Output: plain with multiline cells with headers
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
        let expected = vec![
            "       more  more spam",
            "  spam eggs  & eggs",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        let result = tabulate(Style::Plain, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_with_empty_cells() {
        //Output: plain with multiline cells and empty cells with headers
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
        let expected = vec![
            "  hdr  data            fold",
            "    1",
            "    2  very long data  fold",
            "                       this",
        ]
        .join("\n");
        let result = tabulate(Style::Plain, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn plain_multiline_with_empty_cells_headerless() {
        //Output: plain with multiline cells and empty cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
        let expected = vec![
            "0",
            "1",
            "2  very long data  fold",
            "                   this",
        ]
        .join("\n");
        let result = tabulate(Style::Plain, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple() {
        //Output: simple with headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "strings      numbers",
            "---------  ---------",
            "spam         41.9999",
            "eggs        451",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_2() {
        //Output: simple with multiline cells
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Text("foo"), Cell::Text("bar")],
                vec![Cell::Text("spam"), Cell::Text("multiline\nworld")],
            ],
            vec!["key", "value"],
        );
        let expected = vec![
            " key     value",
            "-----  ---------",
            " foo      bar",
            "spam   multiline",
            "         world",
        ]
        .join("\n");
        let result = tabulate_with_align(
            Style::Simple,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_headerless() {
        //Output: simple without headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "----  --------",
            "spam   41.9999",
            "eggs  451",
            "----  --------",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_headerless() {
        //Output: simple with multiline cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
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
        let result = tabulate_with_align(
            Style::Simple,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline() {
        //Output: simple with multiline cells with headers
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
        let expected = vec![
            "       more  more spam",
            "  spam eggs  & eggs",
            "-----------  -----------",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_ansi() {
        //Output: simple with multiline headers and colors (ansi escape)
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam \x1b[31meggs\x1b[0m", "more spam\n& eggs"],
        );
        let expected = vec![
            "       more  more spam",
            "  spam \x1b[31meggs\x1b[0m  & eggs",
            "-----------  -----------",
            "          2  foo",
            "             bar",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_with_empty_cells() {
        //Output: simple with multiline cells and empty cells with headers
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
        let expected = vec![
            "  hdr  data            fold",
            "-----  --------------  ------",
            "    1",
            "    2  very long data  fold",
            "                       this",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn simple_multiline_with_empty_cells_headerless() {
        //Output: simple with multiline cells and empty cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
        let expected = vec![
            "-  --------------  ----",
            "0",
            "1",
            "2  very long data  fold",
            "                   this",
            "-  --------------  ----",
        ]
        .join("\n");
        let result = tabulate(Style::Simple, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn github() {
        //Output: github with headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "| strings   |   numbers |",
            "|-----------|-----------|",
            "| spam      |   41.9999 |",
            "| eggs      |  451      |",
        ]
        .join("\n");
        let result = tabulate(Style::Github, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid() {
        //Output: grid with headers
        let tested_input = TestedInput::default();
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
        let result = tabulate(Style::Grid, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_wide_characters() {
        //Output: grid with wide characters in headers
        let unistr = "配列";
        let headers = vec!["strings", unistr];
        let contents = vec![
            vec![
                Cell::Text("Ответ на главный вопрос жизни, вселенной и всего такого"),
                Cell::Int(42),
            ],
            vec![Cell::Text("pi"), Cell::Float(3.1415)],
        ];
        let tested_input = TestedInput { headers, contents };
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
        let result = tabulate(Style::Grid, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_headerless() {
        //Output: grid without headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "+------+----------+",
            "| spam |  41.9999 |",
            "+------+----------+",
            "| eggs | 451      |",
            "+------+----------+",
        ]
        .join("\n");
        let result = tabulate(Style::Grid, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_headerless() {
        //Output: grid with multiline cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
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
        let result = tabulate_with_align(
            Style::Grid,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline() {
        //Output: grid with multiline cells with headers
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
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
        let result = tabulate(Style::Grid, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_with_empty_cells() {
        //Output: grid with multiline cells and empty cells with headers
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
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
        let result = tabulate(Style::Grid, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn grid_multiline_with_empty_cells_headerless() {
        //Output: grid with multiline cells and empty cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
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
        let result = tabulate(Style::Grid, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid() {
        //Output: fancy_grid with headers
        let tested_input = TestedInput::default();
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
        let result = tabulate(Style::Fancy, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_headerless() {
        //Output: fancy_grid without headers
        let tested_input = TestedInput::default();
        let expected = vec![
            "╒══════╤══════════╕",
            "│ spam │  41.9999 │",
            "├──────┼──────────┤",
            "│ eggs │ 451      │",
            "╘══════╧══════════╛",
        ]
        .join("\n");
        let result = tabulate(Style::Fancy, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_headerless() {
        //Output: fancy_grid with multiline cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
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
        let result = tabulate_with_align(
            Style::Fancy,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline() {
        //Output: fancy_grid with multiline cells with headers
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
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
        let result = tabulate(Style::Fancy, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_with_empty_cells() {
        //Output: fancy_grid with multiline cells and empty cells with headers
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
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
        let result = tabulate(Style::Fancy, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancy_grid_multiline_with_empty_cells_headerless() {
        //Output: fancy_grid with multiline cells and empty cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
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
        let result = tabulate(Style::Fancy, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn presto() {
        //Output: presto with headers
        let tested_input = TestedInput::default();
        let expected = vec![
            " strings   |   numbers",
            "-----------+-----------",
            " spam      |   41.9999",
            " eggs      |  451",
        ]
        .join("\n");
        let result = tabulate(Style::Presto, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_headerless() {
        //Output: presto without headers
        let tested_input = TestedInput::default();
        let expected = vec![" spam |  41.9999", " eggs | 451"].join("\n");
        let result = tabulate(Style::Presto, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_headerless() {
        //Output: presto with multiline cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
        let expected = vec![
            " foo bar |   hello",
            "   baz   |",
            "   bau   |",
            "         | multiline",
            "         |   world",
        ]
        .join("\n");
        let result = tabulate_with_align(
            Style::Presto,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline() {
        //Output: presto with multiline cells with headers
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
        let expected = vec![
            "        more | more spam",
            "   spam eggs | & eggs",
            "-------------+-------------",
            "           2 | foo",
            "             | bar",
        ]
        .join("\n");
        let result = tabulate(Style::Presto, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_with_empty_cells() {
        //Output: presto with multiline cells and empty cells with headers
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
        let expected = vec![
            "   hdr | data           | fold",
            "-------+----------------+--------",
            "     1 |                |",
            "     2 | very long data | fold",
            "       |                | this",
        ]
        .join("\n");
        let result = tabulate(Style::Presto, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn presto_multiline_with_empty_cells_headerless() {
        //Output: presto with multiline cells and empty cells without headers
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
        let expected = vec![
            " 0 |                |",
            " 1 |                |",
            " 2 | very long data | fold",
            "   |                | this",
        ]
        .join("\n");
        let result = tabulate(Style::Presto, tested_input.contents, tested_input.headers);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid() {
        let tested_input = TestedInput::default();
        let expected = vec![
            "│ strings   │   numbers │",
            "├───────────┼───────────┤",
            "│ spam      │   41.9999 │",
            "│ eggs      │  451      │",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyGithub,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_headerless() {
        let tested_input = TestedInput::default();
        let expected = vec!["│ spam │  41.9999 │", "│ eggs │ 451      │"].join("\n");
        let result = tabulate(Style::FancyGithub, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_headerless() {
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
        let expected = vec![
            "│ foo bar │   hello   │",
            "│   baz   │           │",
            "│   bau   │           │",
            "│         │ multiline │",
            "│         │   world   │",
        ]
        .join("\n");
        let result = tabulate_with_align(
            Style::FancyGithub,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline() {
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
        let expected = vec![
            "│        more │ more spam   │",
            "│   spam eggs │ & eggs      │",
            "├─────────────┼─────────────┤",
            "│           2 │ foo         │",
            "│             │ bar         │",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyGithub,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_with_empty_cells() {
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
        let expected = vec![
            "│   hdr │ data           │ fold   │",
            "├───────┼────────────────┼────────┤",
            "│     1 │                │        │",
            "│     2 │ very long data │ fold   │",
            "│       │                │ this   │",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyGithub,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancygithub_grid_multiline_with_empty_cells_headerless() {
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
        let expected = vec![
            "│ 0 │                │      │",
            "│ 1 │                │      │",
            "│ 2 │ very long data │ fold │",
            "│   │                │ this │",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyGithub,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid() {
        let tested_input = TestedInput::default();
        let expected = vec![
            "strings   │   numbers",
            "──────────┼──────────",
            "spam      │   41.9999",
            "eggs      │  451",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyPresto,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_headerless() {
        let tested_input = TestedInput::default();
        let expected = vec!["spam │  41.9999", "eggs │ 451"].join("\n");
        let result = tabulate(Style::FancyPresto, tested_input.contents, vec![]);
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_headerless() {
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Text("foo bar\nbaz\nbau"), Cell::Text("hello")],
            vec![Cell::Text(""), Cell::Text("multiline\nworld")],
        ]);
        let expected = vec![
            "foo bar │   hello",
            "  baz   │",
            "  bau   │",
            "        │ multiline",
            "        │   world",
        ]
        .join("\n");
        let result = tabulate_with_align(
            Style::FancyPresto,
            tested_input.contents,
            tested_input.headers,
            Align::Center,
            Align::Right,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline() {
        let tested_input = TestedInput::new(
            vec![vec![Cell::Int(2), Cell::Text("foo\nbar")]],
            vec!["more\nspam eggs", "more spam\n& eggs"],
        );
        let expected = vec![
            "       more │ more spam",
            "  spam eggs │ & eggs",
            "────────────┼────────────",
            "          2 │ foo",
            "            │ bar",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyPresto,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_with_empty_cells() {
        let tested_input = TestedInput::new(
            vec![
                vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
                vec![
                    Cell::Int(2),
                    Cell::Text("very long data"),
                    Cell::Text("fold\nthis"),
                ],
            ],
            vec!["hdr", "data", "fold"],
        );
        let expected = vec![
            "  hdr │ data           │ fold",
            "──────┼────────────────┼───────",
            "    1 │                │",
            "    2 │ very long data │ fold",
            "      │                │ this",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyPresto,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }

    #[test]
    fn fancypresto_grid_multiline_with_empty_cells_headerless() {
        let tested_input = TestedInput::with_contents(vec![
            vec![Cell::Int(0), Cell::Text(""), Cell::Text("")],
            vec![Cell::Int(1), Cell::Text(""), Cell::Text("")],
            vec![
                Cell::Int(2),
                Cell::Text("very long data"),
                Cell::Text("fold\nthis"),
            ],
        ]);
        let expected = vec![
            "0 │                │",
            "1 │                │",
            "2 │ very long data │ fold",
            "  │                │ this",
        ]
        .join("\n");
        let result = tabulate(
            Style::FancyPresto,
            tested_input.contents,
            tested_input.headers,
        );
        assert_eq!(expected, result);
    }
}
