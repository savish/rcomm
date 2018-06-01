#[macro_use]
extern crate clap;
extern crate rcomm;

use std::fs::File;
use std::io::{self, Write};

use clap::App;
use rcomm::{Config, CreateConfig, FilePair};

fn main() -> io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file_pair = FilePair::new(
        File::open(matches.value_of("file1").unwrap())?,
        File::open(matches.value_of("file2").unwrap())?,
    );


    let config: Config = CreateConfig::new()
        .ignore_case(matches.is_present("ignore_case"))
        .suppress_column(1, matches.is_present("col1"))
        .suppress_column(2, matches.is_present("col2"))
        .suppress_column(3, matches.is_present("col3"))
        .create();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for result_row in file_pair.into_lines(config) {
        writeln!(handle, "{}", result_row)?;
    }

    Ok(())
}
