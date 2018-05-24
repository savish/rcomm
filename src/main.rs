#[macro_use]
extern crate clap;
use clap::App;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

#[derive(Debug)]
struct Row<'a> {
    text: &'a str,
    column: u8,
}

fn display_row(row: &Row) -> String {
    format!("{}{}", "\t".repeat(row.column as usize), row.text)
}

fn generate_row<'a>(str1: &'a str, str2: &'a str, ignore_case: bool) -> Row<'a> {
    let _str1 = if ignore_case {
        str1.to_lowercase()
    } else {
        str1.to_string()
    };

    let _str2 = if ignore_case {
        str2.to_lowercase()
    } else {
        str2.to_string()
    };

    if _str1 < _str2 {
        Row {
            text: str1,
            column: 0,
        }
    } else if _str1 > _str2 {
        Row {
            text: str2,
            column: 1,
        }
    } else {
        Row {
            text: str1,
            column: 2,
        }
    }
}

fn main() -> io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file1 = File::open(matches.value_of("file1").unwrap())?;
    let mut file1_reader = BufReader::new(file1).lines();

    let file2 = File::open(matches.value_of("file2").unwrap())?;
    let mut file2_reader = BufReader::new(file2).lines();

    let mut line1 = file1_reader.next().unwrap_or(Ok("".to_string())).unwrap();
    let mut line2 = file2_reader.next().unwrap_or(Ok("".to_string())).unwrap();

    while !line1.is_empty() && !line2.is_empty() {
        let line1_ref = &(line1.clone());
        let line2_ref = &(line2.clone());
        let row = generate_row(line1_ref, line2_ref, matches.is_present("ignore_case"));
        println!("{}", display_row(&row));

        line1 = match row.column {
            0 | 2 => file1_reader.next().unwrap_or(Ok("".to_string())).unwrap(),
            _ => line1.clone(),
        };

        line2 = match row.column {
            1 | 2 => file2_reader.next().unwrap_or(Ok("".to_string())).unwrap(),
            _ => line2.clone(),
        };
    }

    if !line1.is_empty() {
        println!(
            "{}",
            display_row(&Row {
                text: &line1,
                column: 0
            })
        );

        while let Some(val) = file1_reader.next() {
            println!(
                "{}",
                display_row(&Row {
                    text: &(val.unwrap()),
                    column: 0
                })
            );
        }
    }

    if !line2.is_empty() {
        println!(
            "{}",
            display_row(&Row {
                text: &line2,
                column: 1
            })
        );

        while let Some(val) = file2_reader.next() {
            println!(
                "{}",
                display_row(&Row {
                    text: &(val.unwrap()),
                    column: 1
                })
            );
        }
    }

    Ok(())
}
