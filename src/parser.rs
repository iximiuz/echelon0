use rule::{FieldType, ParseRule, RuleParser};
use rule::Error as RuleError;

use self::chrono::{DateTime, UTC};

use std::collections::HashMap;
use std::num;

extern crate chrono;

/// Typed value of an entry's field.
#[derive(Debug)]
pub enum FieldValue<'a> {
    Int(i64),
    UInt(u64),
    Float(f64),
    DateTime(DateTime<UTC>),
    Str(&'a str),
}

/// Parsed data unit (line, message, whatever).
pub type Entry<'a, 't> = HashMap<&'a str, FieldValue<'t>>;

/// Error cases during parser creation.
#[derive(Debug)]
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
    NotMatch,
    ParseIntError,
    ParseFloatError,
}

impl From<num::ParseIntError> for ParseError {
    fn from(_: num::ParseIntError) -> ParseError {
        ParseError::ParseIntError
    }
}

impl From<num::ParseFloatError> for ParseError {
    fn from(_: num::ParseFloatError) -> ParseError {
        ParseError::ParseFloatError
    }
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

    pub fn parse_entry<'t>(&'t self, l: &'t String) -> Result<Entry, ParseError> {
        // TODO: check that all fields have unique names.

        let matches = self.rule.re.find_iter(l).collect::<Vec<(usize, usize)>>();
        if self.rule.fields.len() + 1 != matches.len() {
            return Err(ParseError::NotMatch);
        }

        let mut entry = Entry::new();
        for (i, field) in self.rule.fields.iter().enumerate() {
            let (lo, hi) = matches[i + 1];
            let val = &l[lo..hi];
            let val = match field.typ {
                FieldType::Int => FieldValue::Int(try!(val.parse())),
                FieldType::UInt => FieldValue::UInt(try!(val.parse())),
                FieldType::Float => FieldValue::Float(try!(val.parse())),
                FieldType::DateTime(_) => panic!("Not implemented"),
                FieldType::Str => FieldValue::Str(val), 
            };
            entry.insert(field.name, val);
        }
        Ok(entry)
    }

    // #[inline]
    // fn parse_int(&self, l: &str, field_name: &str) -> Result<FiledValue::Int, ParseError> {
    //     let val = l.parse();
    //     if !val.is_ok() {
    //         err!((
    //     }
    // }
}
