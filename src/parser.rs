use rule::{FieldType, ParseRule, RuleParser};
use rule::Error as RuleError;

use self::chrono::{DateTime, TimeZone, UTC};

use std::collections::HashMap;
use std::num;

extern crate chrono;

/// Typed value of an entry's field.
#[derive(Debug, PartialEq)]
pub enum FieldValue<'t> {
    Int(i64),
    UInt(u64),
    Float(f64),
    DateTime(DateTime<UTC>),
    Str(&'t str),
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
    LineNotMatch,
    EmptyCaptureGroup(usize),
    ParseIntError,
    ParseFloatError,
    ParseDateTimeError(chrono::ParseError),
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

impl From<chrono::ParseError> for ParseError {
    fn from(err: chrono::ParseError) -> ParseError {
        ParseError::ParseDateTimeError(err)
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

    pub fn parse_entry<'t>(&self, l: &'t String) -> Result<Entry<'a, 't>, ParseError> {
        let captures = match self.rule.re.captures(l) {
            Some(c) => c,
            None => return Err(ParseError::LineNotMatch),
        };

        let mut entry = Entry::new();
        for (i, field) in self.rule.fields.iter().enumerate() {
            let val = match captures.at(i + 1) {
                Some(v) => v,
                None => return Err(ParseError::EmptyCaptureGroup(i + 1)),
            };
            let val = match field.typ {
                FieldType::Int => FieldValue::Int(try!(val.parse())),  // TODO: improve error
                FieldType::UInt => FieldValue::UInt(try!(val.parse())),
                FieldType::Float => FieldValue::Float(try!(val.parse())),
                FieldType::DateTime(format) => try!(self.parse_dt(val, format)),
                FieldType::Str => FieldValue::Str(val), 
            };
            entry.insert(field.name, val);
        }
        Ok(entry)
    }

    #[inline]
    fn parse_dt<'t>(&self, val: &'t str, format: &'a str) -> Result<FieldValue<'t>, ParseError> {
        Ok(FieldValue::DateTime(try!(UTC.datetime_from_str(val, format)))) // TODO: improve error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_entry() {
        let rule = r"/(\d+)\s(\w+)/ num:uint,res";
        let line = String::from("123 some_word");
        let parser = Parser::new(&rule).unwrap();
        let entry = parser.parse_entry(&line).unwrap();

        assert_eq!(FieldValue::UInt(123), entry[&"num"]);
    }
}
