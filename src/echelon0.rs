use parser::{Entry, Parser};

use std::io::{BufRead, Write};

extern crate regex;
extern crate serde;
extern crate serde_json;

#[derive(Debug)]
struct OpStats {
    ok: u64,
    failed: u64,
}

impl OpStats {
    fn new() -> OpStats {
        OpStats { ok: 0, failed: 0 }
    }

    #[inline]
    fn ok(&mut self) {
        self.ok += 1;
    }

    #[inline]
    fn fail(&mut self) {
        self.failed += 1;
    }
}

#[derive(Debug)]
struct Stats {
    read: OpStats,
    parse: OpStats,
    write: OpStats,
    skipped: u64,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            read: OpStats::new(),
            parse: OpStats::new(),
            write: OpStats::new(),
            skipped: 0,
        }
    }

    #[inline]
    fn skipped(&mut self) {
        self.skipped += 1;
    }
}

pub struct Echelon0<'a, 'b> {
    input: &'a mut BufRead,
    output: &'a mut Write,
    parser: &'b Parser<'b>,
    include: Option<regex::Regex>,
    exclude: Option<regex::Regex>,
    stats: Stats,
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
            stats: Stats::new(),
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
        let mut line = String::with_capacity(4096);
        while self.read_line(&mut line) {
            match self.parser.parse_entry(&line) {
                Ok(entry) => {
                    self.stats.parse.ok();
                    self.try_write(&entry);
                }
                Err(err) => {
                    self.stats.parse.fail();
                    warn!("Parsing error\nLine: {}\nError: {:?}", line, err);
                }
            }
        }
    }

    #[inline]
    fn read_line(&mut self, mut line: &mut String) -> bool {
        loop {
            line.clear();
            if let Err(err) = self.input.read_line(&mut line) {
                self.stats.read.fail();
                warn!("Reading error: {}", err);
                continue;
            }

            self.stats.read.ok();

            if line.len() == 0 {
                return false; // found EOF
            }

            if let Some(ref re) = self.include {
                if !re.is_match(&line) {
                    self.stats.skipped();
                    continue;
                }
            }

            if let Some(ref re) = self.exclude {
                if re.is_match(&line) {
                    self.stats.skipped();
                    continue;
                }
            }

            return true;
        }
    }

    #[inline]
    fn try_write(&mut self, entry: &Entry) {
        match serde_json::to_string(&entry) {
            Ok(v) => {
                match write!(self.output, "{}\n", v) {
                    Ok(_) => {
                        self.stats.write.ok();
                    }
                    Err(err) => {
                        self.stats.write.fail();
                        warn!("Write error: {}", err);
                    }
                }
            }
            Err(err) => {
                self.stats.write.fail();
                warn!("Serialize error: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::Parser;
    use std::io::{BufRead, BufReader, Write};

    #[test]
    fn test_start() {
        let lines = vec![
            "foo 123",
            "bar 345",
            "",
            "baz 678",
            "",
            "",
            "bad formatted",
            "foobar 123456",
            "",
        ];
        let lines = lines.join("\n");
        let mut result: Vec<u8> = Vec::new();

        {
            let mut reader = BufReader::new(lines.as_bytes());
            let mut input: &mut BufRead = &mut reader;
            let mut output: &mut Write = &mut result;

            let parser = Parser::new(r"/(\w+) (\d+)/ word,digit:int").unwrap();
            let mut ech0 = Echelon0::new(input, output, &parser);
            ech0.set_exclude_filter("baz").expect("");
            ech0.start();

            assert_eq!(9, ech0.stats.read.ok);
            assert_eq!(0, ech0.stats.read.failed);
            assert_eq!(1, ech0.stats.skipped);
            assert_eq!(3, ech0.stats.parse.ok);
            assert_eq!(4, ech0.stats.parse.failed);
            assert_eq!(3, ech0.stats.write.ok);
            assert_eq!(0, ech0.stats.write.failed);
        }

        let result = String::from_utf8(result).unwrap();
        assert_eq!(4, result.split("\n").count()); // +1 because of extra new line in the end
    }
}
