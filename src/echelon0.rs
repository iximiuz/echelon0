use parser::Parser;

use std::io::BufRead;

extern crate regex;

pub struct Echelon0<'a, 'b> {
    reader: &'a mut BufRead,
    parser: &'b Parser<'b>,
    include: Option<regex::Regex>,
    exclude: Option<regex::Regex>,
}

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
    pub fn new(reader: &'a mut BufRead, parser: &'b Parser<'b>) -> Echelon0<'a, 'b> {
        Echelon0 {
            reader: reader,
            parser: parser,
            include: None,
            exclude: None,
        }
    }

    pub fn set_include_filter(&mut self, f: &str) -> Result<(), Error> {
        // TODO: return error if there is an exclude filter already.
        self.include = Some(try!(regex::Regex::new(f)));
        Ok(())
    }

    pub fn set_exclude_filter(&mut self, f: &str) -> Result<(), Error> {
        // TODO: return error if there is an include filter already.
        self.exclude = Some(try!(regex::Regex::new(f)));
        Ok(())
    }

    pub fn start(&mut self) {
        let mut line = String::new();
        loop {
            // TODO: Move line reading to the separate function.
            // TODO: add stats fields (read/read_failed/skipped/parsed/parse_failed)
            line.clear();
            match self.reader.read_line(&mut line) {
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
                    println!("{:?}", entry); // TODO: use output
                }
                Err(err) => {
                    println!("{:?}", err); // TODO: log
                }
            }
        }
    }
}
