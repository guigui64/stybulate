use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use structopt::StructOpt;

use stybulate::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "stybulate", about = "tabulate with style")]
struct Opt {
    /// The path to the file to read, stdin if not present
    #[structopt(parse(from_os_str))]
    path: Option<PathBuf>,

    /// Print table to outputfile, stdout if not present
    #[structopt(parse(from_os_str), short, long)]
    output: Option<PathBuf>,

    /// Use the first row of data as a table header
    #[structopt(short = "1", long)]
    header: bool,

    /// Set output table format.
    /// Supported formats: plain, simple, github, grid, fancy, presto, fancygithub, fancypresto.
    /// Defaults to simple.
    #[structopt(short, long, default_value = "simple")]
    fmt: String,
}

fn main() -> Result<()> {
    // Parse arguments
    let opt = Opt::from_args();

    // Style
    let fmt = Style::from(&opt.fmt).ok_or(anyhow!("Unsupported format \"{}\"", opt.fmt))?;

    // Output
    let mut writer: Box<dyn Write> = match opt.output {
        None => Box::new(BufWriter::new(io::stdout())),
        Some(opath) => Box::new(BufWriter::new(
            File::create(opath).context("Could not write to specified output file")?,
        )),
    };

    // Input
    let reader: Box<dyn BufRead> = match opt.path {
        None => Box::new(BufReader::new(io::stdin())),
        Some(ipath) => Box::new(BufReader::new(
            File::open(ipath).context("Could not read input file")?,
        )),
    };

    // Parse
    let mut first = true;
    let mut headers = None;
    let mut contents: Vec<Vec<Cell>> = Vec::new();
    for line in reader.lines() {
        let l = line.unwrap();
        let l = l.trim();
        // header
        if opt.header && first {
            first = false;
            headers = Some(Headers::from(l.split_whitespace().collect()));
            continue;
        }
        // data
        contents.push(
            l.split_whitespace()
                .map(|data| {
                    if let Ok(i) = data.parse::<i32>() {
                        Cell::Int(i)
                    } else if let Ok(f) = data.parse::<f64>() {
                        Cell::Float(f)
                    } else {
                        Cell::from(data)
                    }
                })
                .collect(),
        );
    }

    // Tabulate
    writeln!(writer, "{}", Table::new(fmt, contents, headers).tabulate())?;

    Ok(())
}
