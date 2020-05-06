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
    /// Get Style from &str
    pub fn from(s: &str) -> Option<Self> {
        match s {
            "plain" => Some(Self::Plain),
            "simple" => Some(Self::Simple),
            "github" => Some(Self::Github),
            "grid" => Some(Self::Grid),
            "fancy" => Some(Self::Fancy),
            "presto" => Some(Self::Presto),
            "fancygithub" => Some(Self::FancyGithub),
            "fancypresto" => Some(Self::FancyPresto),
            _ => None,
        }
    }

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

    #[cfg(feature = "ansi_term_style")]
    /// Apply style to line
    pub fn apply_style(&mut self, style: ansi_term::Style) {
        self.begin = paint(&self.begin, style);
        self.hline = paint(&self.hline, style);
        self.sep = paint(&self.sep, style);
        self.end = paint(&self.end, style);
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

    #[cfg(feature = "ansi_term_style")]
    /// Apply style to datarow
    pub fn apply_style(&mut self, style: ansi_term::Style) {
        self.begin = paint(&self.begin, style);
        self.sep = paint(&self.sep, style);
        self.end = paint(&self.end, style);
    }
}

#[cfg(feature = "ansi_term_style")]
fn paint(s: &str, style: ansi_term::Style) -> String {
    style.paint(s).to_string()
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

#[cfg(feature = "ansi_term_style")]
impl TableFormat {
    /// Apply the style to all the Strings in the TableFormat
    pub fn apply_style(&mut self, style: ansi_term::Style) {
        if let Some(la) = self.lineabove.as_mut() {
            la.apply_style(style);
        }
        if let Some(lbh) = self.linebelowheader.as_mut() {
            lbh.apply_style(style);
        }
        if let Some(lbr) = self.linebetweenrows.as_mut() {
            lbr.apply_style(style);
        }
        if let Some(lb) = self.linebelow.as_mut() {
            lb.apply_style(style);
        }
        self.headerrow.apply_style(style);
        self.datarow.apply_style(style);
    }
}
