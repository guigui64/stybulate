/// The style of the table
///
/// Examples shown will have a header line and two content lines
#[derive(Clone)]
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

impl Style {
    /// Returns the corresponding format
    pub fn to_format(&self) -> TableFormat {
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

#[derive(Clone)]
pub struct Line {
    pub begin: String,
    pub hline: String,
    pub sep: String,
    pub end: String,
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
pub struct DataRow {
    pub begin: String,
    pub sep: String,
    pub end: String,
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
pub struct TableFormat {
    pub lineabove: Option<Line>,
    pub linebelowheader: Option<Line>,
    pub linebetweenrows: Option<Line>,
    pub linebelow: Option<Line>,
    pub headerrow: DataRow,
    pub datarow: DataRow,
    pub padding: u32,
    pub hidelineaboveifheader: bool,
    pub hidelinebelowifheader: bool,
}
