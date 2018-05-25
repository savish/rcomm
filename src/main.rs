#[macro_use]
extern crate clap;
use clap::App;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, Lines, Write};

// File input to the comm utility
struct CommFile {
    lines: Lines<BufReader<File>>,
    current_line: Option<Result<String, io::Error>>,
    advance_cursor: bool,
}

impl CommFile {
    pub fn new(lines: Lines<BufReader<File>>) -> CommFile {
        CommFile {
            lines,
            current_line: None,
            advance_cursor: true,
        }
    }
}

/// Double-file iterator
struct CommFiles {
    file1: CommFile,
    file2: CommFile,
}

impl CommFiles {
    pub fn new(file1: Lines<BufReader<File>>, file2: Lines<BufReader<File>>) -> CommFiles {
        CommFiles {
            file1: CommFile::new(file1),
            file2: CommFile::new(file2),
        }
    }

    pub fn should_advance_cursor(
        line1: &Option<String>,
        line2: &Option<String>,
    ) -> (bool, bool, u8) {
        match (line1, line2) {
            (Some(line1_string), Some(line2_string)) => {
                if line1_string < line2_string {
                    (true, false, 0)
                } else if line1_string > line2_string {
                    (false, true, 1)
                } else {
                    (true, true, 2)
                }
            }
            (Some(_), None) => (true, false, 0),
            (None, Some(_)) => (false, true, 1),
            (None, None) => (false, false, 0),
        }
    }
}

impl Iterator for CommFiles {
    type Item = (String, u8);

    fn next(&mut self) -> Option<Self::Item> {
        // Read the next line from each buffer
        if self.file1.advance_cursor {
            self.file1.current_line = self.file1.lines.next()
        };

        if self.file2.advance_cursor {
            self.file2.current_line = self.file2.lines.next()
        };

        let file1_next = &self.file1.current_line;
        let file2_next = &self.file2.current_line;

        if file1_next.is_none() && file2_next.is_none() {
            // End the iterator when both files have been read to completion
            None
        } else {
            // Gets the next line from the first file, if it exists
            let file1_line = match file1_next {
                Some(result) => {
                    let line_content = match result {
                        Ok(line) => line,
                        _ => "",
                    };
                    Some(line_content.to_string())
                }
                _ => None,
            };

            // Gets the next line from the second file, if it exists
            let file2_line = match file2_next {
                Some(result) => {
                    let line_content = match result {
                        Ok(line) => line,
                        _ => "",
                    };
                    Some(line_content.to_string())
                }
                _ => None,
            };

            let cursors = CommFiles::should_advance_cursor(&file1_line, &file2_line);
            self.file1.advance_cursor = cursors.0;
            self.file2.advance_cursor = cursors.1;

            match cursors {
                (_, _, 0) => Some((file1_line.unwrap(), 0)),
                (_, _, 1) => Some((file2_line.unwrap(), 1)),
                (_, _, 2) => Some((file1_line.unwrap(), 2)),
                _ => None,
            }
        }
    }
}

fn display_line(line: (String, u8)) -> String {
    format!("{}{}", "\t".repeat(line.1 as usize), line.0)
}

fn main() -> io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file1 = File::open(matches.value_of("file1").unwrap())?;
    let file2 = File::open(matches.value_of("file2").unwrap())?;

    let files_lines = CommFiles::new(BufReader::new(file1).lines(), BufReader::new(file2).lines());
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for lines in files_lines {
        writeln!(handle, "{}", display_line(lines))?;
    }

    Ok(())
}
