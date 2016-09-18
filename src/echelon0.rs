use parser::Parser;

use std::io::{BufRead, Write};

extern crate regex;
extern crate serde;
extern crate serde_json;

pub struct Echelon0<'a, 'b> {
    input: &'a mut BufRead,
    output: &'a mut Write,
    parser: &'b Parser<'b>,
    include: Option<regex::Regex>,
    exclude: Option<regex::Regex>,
}

#[derive(Debug)]
pub enum Error {
    FiltersConflict,
    BadFilterRegex(regex::Error),
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::BadFilterRegex(err)
    }
}

impl<'a, 'b> Echelon0<'a, 'b> {
    pub fn new(input: &'a mut BufRead,
               output: &'a mut Write,
               parser: &'b Parser<'b>)
               -> Echelon0<'a, 'b> {
        Echelon0 {
            input: input,
            output: output,
            parser: parser,
            include: None,
            exclude: None,
        }
    }

    pub fn set_include_filter(&mut self, f: &str) -> Result<(), Error> {
        if self.exclude.is_some() {
            return Err(Error::FiltersConflict);
        }
        self.include = Some(try!(regex::Regex::new(f)));
        Ok(())
    }

    pub fn set_exclude_filter(&mut self, f: &str) -> Result<(), Error> {
        if self.include.is_some() {
            return Err(Error::FiltersConflict);
        }
        self.exclude = Some(try!(regex::Regex::new(f)));
        Ok(())
    }

    pub fn start(&mut self) {
        let mut line = String::new();
        loop {
            // TODO: Move line reading to the separate function.
            // TODO: add stats fields (read/read_failed/skipped/parsed/parse_failed)
            line.clear();
            match self.input.read_line(&mut line) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                }
                Err(err) => {
                    println!("{:?}", err);  // TODO: log
                    continue;
                }
            }

            if let Some(ref re) = self.include {
                if !re.is_match(&line) {
                    continue;
                }
            }

            if let Some(ref re) = self.exclude {
                if re.is_match(&line) {
                    continue;
                }
            }

            match self.parser.parse_entry(&line) {
                Ok(entry) => {
                    write!(self.output, "{}\n", serde_json::to_string(&entry).unwrap());
                }
                Err(err) => {
                    println!("{:?}", err); // TODO: log
                }
            }
        }
    }
}
