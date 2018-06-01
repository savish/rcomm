#[macro_use]
extern crate structopt;
extern crate rcomm;

use std::fs::File;
use std::io::{self, Write};

use rcomm::{Config, CreateConfig, FilePair};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "rcomm_so",
    about = "A Rust version of the GNU core util `comm`",
    raw(global_settings = "&[AppSettings::ColoredHelp]")
)]
struct Opt {
    /// The first file.
    #[structopt(name = "file1")]
    file1: String,
    /// The second file.
    #[structopt(name = "file2")]
    file2: String,
    /// Suppress printing of column 1.
    #[structopt(short = "1")]
    col1: bool,
    /// Suppress printing of column 2.
    #[structopt(short = "2")]
    col2: bool,
    /// Suppress printing of column 3.
    #[structopt(short = "3")]
    col3: bool,
    /// Case insensitive comparison of lines.
    #[structopt(short = "i")]
    ignore_case: bool,
}

fn main() -> io::Result<()> {
    let matches = Opt::from_args();

    let file_pair = FilePair::new(File::open(matches.file1)?, File::open(matches.file2)?);

    let config: Config = CreateConfig::new()
        .ignore_case(matches.ignore_case)
        .suppress_column(1, matches.col1)
        .suppress_column(2, matches.col2)
        .suppress_column(3, matches.col3)
        .create();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for result_row in file_pair.into_lines(config) {
        writeln!(handle, "{}", result_row)?;
    }

    Ok(())
}
