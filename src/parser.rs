use std::collections::HashMap;
use std::fmt;
use std::str::Chars;

extern crate regex;
use self::regex::Regex;

pub enum Value {
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(String),
}

pub type Entry<'a> = HashMap<&'a str, Value>;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value")
    }
}

pub enum ParserError {
    BadParsePattern,
}

pub struct Parser<'a> {
    fields: Vec<&'a str>,
}

impl<'a> Parser<'a> {
    pub fn new(pattern: &'a str) -> Result<Parser<'a>, ParserError> {
        Ok(Parser { fields: vec![pattern] })
    }

    pub fn parse_entry(&self, l: &String) -> Result<Entry, String> {
        Err("FooBar".to_string())
    }
}

enum Token<'a> {
    Regex(&'a str),
    FieldName(&'a str),
    TypeInt,
    TypeFloat,
    TypeDateTime(&'a str),
    Comma,
    WS, // White spaces
    Illegal,
}

type ScannedToken<'a> = Option<(Token<'a>, usize)>;

struct RuleReader<'a> {
    rule: Chars<'a>,
    pos: usize,
    cur: char,
    buffered: bool,
}

impl<'a> RuleReader<'a> {
    fn new(rule: &'a str) -> RuleReader<'a> {
        RuleReader {
            rule: rule.chars(),
            pos: 0,
            cur: ' ',
            buffered: false,
        }
    }

    fn read_char(&mut self) -> Option<(char, usize)> {
        if self.buffered {
            self.buffered = false;
            return Some((self.cur, self.pos));
        }

        self.cur = match self.rule.next() {
            Some(c) => c,
            None => return None,
        };
        self.pos += 1;
        Some((self.cur, self.pos))
    }

    fn unread(&mut self) {
        if self.buffered {
            panic!("Cannot unread twice!");
        }
        self.buffered = self.pos > 0;
    }
}

struct ParseRuleScanner<'a> {
    reader: RuleReader<'a>,
}

impl<'a> ParseRuleScanner<'a> {
    fn new(rule: &'a str) -> ParseRuleScanner<'a> {
        ParseRuleScanner { reader: RuleReader::new(rule) }
    }

    fn scan(&mut self) -> ScannedToken<'a> {
        let (ch0, pos) = match self.reader.read_char() {
            Some((c, p)) => (c, p),
            None => return None, 
        };

        match ch0 {
            '/' => self.scan_regex(),
            ' ' | '\t' => self.scan_whitespace(),
            ':' => self.scan_field_type(),
            ',' => Some((Token::Comma, pos)),
            _ => {
                if self.is_ident_first_symbol(ch0) {
                    self.scan_field_name()
                } else {
                    Some((Token::Illegal, pos))
                }
            }

        }
    }

    fn scan_field_name(&mut self) -> ScannedToken<'a> {
        None
    }

    fn scan_field_type(&mut self) -> ScannedToken<'a> {
        None
    }

    fn scan_regex(&mut self) -> ScannedToken<'a> {
        None
    }

    fn scan_whitespace(&mut self) -> ScannedToken<'a> {
        None
    }

    fn is_ident_first_symbol(&self, ch0: char) -> bool {
        false
    }
}
