use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, Lines};

#[derive(Debug)]
pub struct Config {
  ignore_case: bool,
  suppress_column_1: bool,
  suppress_column_2: bool,
  suppress_column_3: bool,
}

#[derive(Default)]
pub struct CreateConfig {
  ignore_case: bool,
  suppress_column_1: bool,
  suppress_column_2: bool,
  suppress_column_3: bool,
}

impl CreateConfig {
  pub fn new() -> CreateConfig {
    CreateConfig::default()
  }

  pub fn ignore_case(mut self, value: bool) -> CreateConfig {
    self.ignore_case = value;
    self
  }

  pub fn suppress_column(mut self, column: u8, value: bool) -> CreateConfig {
    match column {
      1 => {
        self.suppress_column_1 = value;
        self
      }
      2 => {
        self.suppress_column_2 = value;
        self
      }
      3 => {
        self.suppress_column_3 = value;
        self
      }
      _ => self,
    }
  }

  pub fn create(&self) -> Config {
    Config {
      ignore_case: self.ignore_case,
      suppress_column_1: self.suppress_column_1,
      suppress_column_2: self.suppress_column_2,
      suppress_column_3: self.suppress_column_3,
    }
  }
}

/// Files used as input to the `rcomm` command
///
/// These files should be sorted.
pub struct FilePair {
  file1: File,
  file2: File,
}

impl FilePair {
  /// Returns a new instance of the `FilePair` struct
  pub fn new(file1: File, file2: File) -> FilePair {
    FilePair { file1, file2 }
  }

  /// Returns a new iterator through the lines of the two files
  ///
  /// Consumes this struct
  pub fn into_lines(self, config: Config) -> FilePairLinesIterator {
    FilePairLinesIterator {
      file1_lines: FileLines::from_file(self.file1),
      file2_lines: FileLines::from_file(self.file2),
      config,
    }
  }

  // Determine if the cursor fora given file should advance
  fn should_advance_cursor(
    line1: &Option<String>,
    line2: &Option<String>,
    ignore_case: bool,
  ) -> (bool, bool, u8) {
    match (line1, line2) {
      (Some(line1_string), Some(line2_string)) => {
        let advance1 = match ignore_case {
          true => line1_string.to_lowercase() < line2_string.to_lowercase(),
          false => line1_string < line2_string,
        };

        let advance2 = match ignore_case {
          true => line2_string.to_lowercase() < line1_string.to_lowercase(),
          false => line2_string < line1_string,
        };

        if advance1 {
          (true, false, 0)
        } else if advance2 {
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

pub struct FilePairLinesIterator {
  file1_lines: FileLines,
  file2_lines: FileLines,
  config: Config,
}

impl Iterator for FilePairLinesIterator {
  type Item = ResultRow;

  fn next(&mut self) -> Option<Self::Item> {
    // Read the next line from each buffer
    if self.file1_lines.advance_cursor {
      self.file1_lines.current_line = self.file1_lines.lines.next()
    };

    if self.file2_lines.advance_cursor {
      self.file2_lines.current_line = self.file2_lines.lines.next()
    };

    let file1_next = &self.file1_lines.current_line;
    let file2_next = &self.file2_lines.current_line;

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

      let cursors =
        FilePair::should_advance_cursor(&file1_line, &file2_line, self.config.ignore_case);
      self.file1_lines.advance_cursor = cursors.0;
      self.file2_lines.advance_cursor = cursors.1;

      match cursors {
        (_, _, 0) => Some(ResultRow::new(&file1_line.unwrap(), 0, "\t")),
        (_, _, 1) => Some(ResultRow::new(&file2_line.unwrap(), 1, "\t")),
        (_, _, 2) => Some(ResultRow::new(&file1_line.unwrap(), 2, "\t")),
        _ => None,
      }
    }
  }
}

struct FileLines {
  lines: Lines<BufReader<File>>,
  current_line: Option<Result<String, io::Error>>,
  advance_cursor: bool,
}

impl FileLines {
  pub fn from_file(file: File) -> FileLines {
    FileLines {
      lines: BufReader::new(file).lines(),
      current_line: None,
      advance_cursor: true,
    }
  }
}

/// Represents a single line of output
pub struct ResultRow {
  text: String,
  column: u8,
  separator: String,
}

impl ResultRow {
  /// Returns a new instance of the struct
  pub fn new(text: &str, column: u8, separator: &str) -> ResultRow {
    ResultRow {
      text: text.into(),
      column,
      separator: separator.into(),
    }
  }
}

impl fmt::Display for ResultRow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}{}",
      self.separator.repeat(self.column as usize),
      self.text
    )
  }
}
