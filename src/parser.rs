use rule::{FieldType, ParseRule, RuleParser};
use rule::Error as RuleError;

use self::chrono::{DateTime, TimeZone, UTC};
use self::serde::ser::{Serialize, Serializer};

use std::collections::HashMap;
use std::num;

extern crate chrono;
extern crate regex;
extern crate serde;
extern crate serde_json;

/// Typed value of an entry's field.
#[derive(Debug, PartialEq)]
pub enum FieldValue<'t> {
    Int(i64),
    UInt(u64),
    Float(f64),
    DateTime(DateTime<UTC>),
    Str(&'t str),
}

impl<'t> Serialize for FieldValue<'t> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match *self {
            FieldValue::Int(v) => serializer.serialize_i64(v),
            FieldValue::UInt(v) => serializer.serialize_u64(v),
            FieldValue::Float(v) => serializer.serialize_f64(v),
            FieldValue::Str(v) => serializer.serialize_str(v),
            FieldValue::DateTime(ref v) => serializer.serialize_str(&format!("{}", v)),
        }
    }
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
        let rule = try!(rule_parser.parse());
        Ok(Parser { rule: rule })
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
    use super::chrono::{TimeZone, UTC};

    #[test]
    fn parse_entry() {
        let rule = r"/(\d+)\s(\w+)/ num:uint,res";
        let line = String::from("123 some_word");
        let parser = Parser::new(&rule).unwrap();
        let entry = parser.parse_entry(&line).unwrap();

        assert_eq!(FieldValue::UInt(123), entry[&"num"]);
    }

    #[test]
    fn parse_nginx_combined() {
        let rule = concat!(r#"/([\d\.]+) - (.+) \[(.+)\] "(.+) ([^?]+)\??(.*) HTTP.+" (\d{3}) (\d+) "(.+)" "(.+)"/"#,
                           " remote_addr,remote_user,time_local:dt[%d/%b/%Y:%H:%M:%S %z]", 
                           ",method,path,query,status:uint,body_bytes_sent:uint,referrer,user_agent");

        let line = concat!(r#"82.208.100.105 - - [01/Aug/2016:22:59:50 +0000] "#,
                           r#""GET /platforms/sa/apps/115?consumer=portal-ru&user_id=11124493 "#,
                           r#"HTTP/1.1" 200 9127 "https://espritgames.ru/fairytail/go/" "#,
                           r#""Mozilla/5.0 (Windows NT 6.3; WOW64) AppleWebKit/537.36 "#,
                           r#"(KHTML, like Gecko) Chrome/52.0.2743.82 Safari/537.36""#);
        let line = String::from(line);
        let parser = Parser::new(&rule).unwrap();
        let entry = parser.parse_entry(&line).unwrap();

        assert_eq!(FieldValue::Str("82.208.100.105"), entry[&"remote_addr"]);
        assert_eq!(FieldValue::Str("-"), entry[&"remote_user"]);
        assert_eq!(FieldValue::DateTime(UTC.ymd(2016, 8, 1).and_hms(22, 59, 50)),
                   entry[&"time_local"]);
        assert_eq!(FieldValue::Str("GET"), entry[&"method"]);
        assert_eq!(FieldValue::Str("/platforms/sa/apps/115"), entry[&"path"]);
        assert_eq!(FieldValue::Str("consumer=portal-ru&user_id=11124493"),
                   entry[&"query"]);
        assert_eq!(FieldValue::UInt(200), entry[&"status"]);
        assert_eq!(FieldValue::UInt(9127), entry[&"body_bytes_sent"]);
        assert_eq!(FieldValue::Str("https://espritgames.ru/fairytail/go/"),
                   entry[&"referrer"]);
        assert_eq!(FieldValue::Str("Mozilla/5.0 (Windows NT 6.3; WOW64) AppleWebKit/537.36 \
                                    (KHTML, like Gecko) Chrome/52.0.2743.82 Safari/537.36"),
                   entry[&"user_agent"]);
    }
}
