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

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value")
    }
}


pub type Entry<'a> = HashMap<&'a str, Value>;


pub enum ParserError {
    BadParseRule,
}


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

    pub fn parse_entry(&self, l: &String) -> Result<Entry, String> {
        Err("FooBar".to_string())
    }
}


struct ParseRule<'a> {
    fields: Vec<&'a str>,
}


struct RuleStrReader<'a> {
    rule: Chars<'a>,
    pos: usize,
    cur: char,
    buffered: bool,
}

impl<'a> RuleStrReader<'a> {
    fn new(rule: &'a str) -> RuleStrReader<'a> {
        RuleStrReader {
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


enum Token<'a> {
    Regex(&'a str),
    FieldName(&'a str),
    TypeInt,
    TypeFloat,
    TypeDateTime(&'a str),
    Comma,
    WS, // White spaces
    EOF,
    Illegal,
}

type ScannedToken<'a> = (Token<'a>, usize);


struct RuleScanner<'a> {
    reader: RuleStrReader<'a>,
}

impl<'a> RuleScanner<'a> {
    fn new(rule: &'a str) -> RuleScanner<'a> {
        RuleScanner { reader: RuleStrReader::new(rule) }
    }

    fn scan(&mut self) -> Result<ScannedToken<'a>, String> {
        let (ch0, pos) = match self.reader.read_char() {
            Some((c, p)) => (c, p),
            None => return Ok((Token::EOF, self.reader.pos)), 
        };

        let scanned = match ch0 {
            '/' => self.scan_regex(),
            ' ' | '\t' => self.scan_whitespace(),
            ':' => self.scan_field_type(),
            ',' => Ok(Token::Comma),
            _ => {
                if self.is_ident_first_symbol(ch0) {
                    self.reader.unread();
                    self.scan_field_name()
                } else {
                    Ok(Token::Illegal)
                }
            }
        };

        match scanned {
            Ok(token) => Ok((token, pos)),
            Err(err) => Err(err),
        }
    }

    fn scan_field_name(&mut self) -> Result<Token<'a>, String> {
        Ok(Token::Illegal)
    }

    fn scan_field_type(&mut self) -> Result<Token<'a>, String> {
        Ok(Token::Illegal)
    }

    fn scan_regex(&mut self) -> Result<Token<'a>, String> {
        Ok(Token::Illegal)
    }

    fn scan_whitespace(&mut self) -> Result<Token<'a>, String> {
        Ok(Token::Illegal)
    }

    fn is_ident_first_symbol(&self, ch0: char) -> bool {
        false
    }
}


struct RuleParser<'a> {
    scanner: RuleScanner<'a>,
}

impl<'a> RuleParser<'a> {
    fn new(rule: &'a str) -> RuleParser<'a> {
        RuleParser { scanner: RuleScanner::new(rule) }
    }

    fn parse(&mut self) -> Result<ParseRule<'a>, String> {
        Err("Not implemented!".to_owned())
    }
}
