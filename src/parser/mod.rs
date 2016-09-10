use self::rule::{ParseRule, RuleParser};

use self::chrono::{DateTime, UTC};

use std::collections::HashMap;
use std::fmt;

extern crate chrono;
mod rule;

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
pub enum ParserError {
    BadParseRule,
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
    pub fn new(rule: &'a str) -> Result<Parser<'a>, ParserError> {
        let mut rule_parser = RuleParser::new(rule);
        let rule = match rule_parser.parse() {
            Ok(r) => r,
            Err(_) => return Err(ParserError::BadParseRule),
        };
        Ok(Parser { rule: rule })
    }

    pub fn parse_entry(&self, l: &String) -> Result<Entry, ParseError> {
        Err(ParseError::Stub)
    }
}
