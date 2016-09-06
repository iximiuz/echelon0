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
        if self.pos == 0 {
            panic!("Cannot unread without reading before!");
        }
        self.buffered = true;
    }
}


#[derive(Eq, PartialEq, Debug)]
enum Token<'a> {
    Regex(&'a str),
    FieldName(&'a str),
    TypeInt,
    TypeFloat,
    TypeDateTime(&'a str),
    Comma,
    WS, // white spaces: ' ', '\t'
    EOF,
}

type ScannedToken<'a> = (Token<'a>, usize);

#[derive(Eq, PartialEq, Debug)]
enum RuleScanError {
    IllegalSymbol {
        pos: usize,
        token: &'static str,
    },
    UnexpectedEndOfRule,
}

type ScanResult<'a> = Result<Token<'a>, RuleScanError>;

struct RuleScanner<'a> {
    rule: &'a str,
    reader: RuleStrReader<'a>,
}

impl<'a> RuleScanner<'a> {
    fn new(rule: &'a str) -> RuleScanner<'a> {
        RuleScanner {
            rule: rule,
            reader: RuleStrReader::new(rule),
        }
    }

    fn scan(&mut self) -> Result<ScannedToken<'a>, RuleScanError> {
        let (ch0, pos) = match self.reader.read_char() {
            Some((c, p)) => (c, p),
            None => return Ok((Token::EOF, self.reader.pos)), 
        };

        let token = match ch0 {
            '/' => try!(self.scan_regex()),
            ' ' | '\t' => try!(self.scan_whitespace()),
            ':' => try!(self.scan_field_type()),
            ',' => Token::Comma,
            _ => {
                if !self.is_ident_symbol(ch0, true) {
                    return Err(RuleScanError::IllegalSymbol {
                        pos: pos,
                        token: "field name",
                    });
                }
                self.reader.unread();
                try!(self.scan_field_name())
            }
        };

        Ok((token, pos))
    }

    fn scan_field_name(&mut self) -> ScanResult<'a> {
        Ok(Token::EOF)
    }

    fn scan_field_type(&mut self) -> ScanResult<'a> {
        Ok(Token::EOF)
    }

    fn scan_regex(&mut self) -> ScanResult<'a> {
        let start_pos = self.reader.pos;
        let mut prev = '_';  // any symbols except '\'
        loop {
            if let Some((ch, _)) = self.reader.read_char() {
                if ch == '/' && prev != '\\' {
                    break;
                }
                prev = ch;
            } else {
                return Err(RuleScanError::UnexpectedEndOfRule);
            }
        }

        let end_pos = self.reader.pos - 1;
        Ok(Token::Regex(&self.rule[start_pos..end_pos]))
    }

    fn scan_whitespace(&mut self) -> ScanResult<'a> {
        loop {
            match self.reader.read_char() {
                Some((' ', _)) | Some(('\t', _)) => continue,
                None => break,
                _ => {
                    self.reader.unread();
                    break;
                }
            }
        }

        Ok(Token::WS)
    }

    fn is_ident_symbol(&self, ch0: char, is_first: bool) -> bool {
        let is_alpha = ('a' <= ch0 && ch0 <= 'z') || ('A' <= ch0 && ch0 <= 'Z');
        let can_be_first = is_alpha || (ch0 == '_');
        can_be_first || (!is_first && '0' <= ch0 && ch0 <= '9')
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

#[cfg(test)]
mod tests {
    use super::{RuleScanner, RuleScanError, Token};

    #[test]
    fn scan_regex() {
        let mut s = RuleScanner::new("/");
        let scanned = s.scan();
        assert_eq!(Err(RuleScanError::UnexpectedEndOfRule), scanned);

        let mut s = RuleScanner::new("//");
        let scanned = s.scan();
        assert_eq!(Ok((Token::Regex(""), 1)), scanned);

        let mut s = RuleScanner::new("/hello/");
        let scanned = s.scan();
        assert_eq!(Ok((Token::Regex("hello"), 1)), scanned);

        let mut s = RuleScanner::new(r"/\d+ \/foo /");
        let scanned = s.scan();
        assert_eq!(Ok((Token::Regex(r"\d+ \/foo "), 1)), scanned);
    }

    #[test]
    fn scan_whitespaces() {
        for rule in &[" ", "\t", "   ", "\t\t\t", " \t ", "\t \t"] {
            let mut s = RuleScanner::new(rule);
            let scanned = s.scan();
            assert_eq!(Ok((Token::WS, 1)), scanned);

            let scanned = s.scan();
            assert_eq!(Ok((Token::EOF, rule.len())), scanned);
        }
    }
}
