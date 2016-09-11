use rule::{ParseRule, RuleParser};
use rule::Error as RuleError;

use self::chrono::{DateTime, UTC};

use std::collections::HashMap;

extern crate chrono;

/// Typed value of an entry's field.
#[derive(Debug)]
pub enum FieldValue {
    Int(i64),
    UInt(u64),
    Float(f64),
    DateTime(DateTime<UTC>),
    Str(String),
}

/// Parsed data unit (line, message, whatever).
pub type Entry<'a> = HashMap<&'a str, FieldValue>;

/// Error cases during parser creation.
pub enum Error {
    BadParseRule(RuleError),
}

impl From<RuleError> for Error {
    fn from(err: RuleError) -> Error {
        Error::BadParseRule(err)
    }
}

/// Error cases during parsing an entry.
#[derive(Debug)]
pub enum ParseError {
    Stub,
}

/// The main part of the Echelon0!
pub struct Parser<'a> {
    rule: ParseRule<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(rule: &'a str) -> Result<Parser<'a>, Error> {
        let mut rule_parser = RuleParser::new(rule);
        Ok(Parser { rule: try!(rule_parser.parse()) })
    }

    pub fn parse_entry(&self, l: &String) -> Result<Entry, ParseError> {
        Err(ParseError::Stub)
    }
}
