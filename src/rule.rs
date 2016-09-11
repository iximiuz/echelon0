use std::str::Chars;

extern crate regex;

pub struct ParseRule<'a> {
    re: regex::Regex,
    fields: Vec<Field<'a>>,
}

enum FieldType<'a> {
    Int,
    UInt,
    Float,
    DateTime(&'a str),
    Str,
}

struct Field<'a> {
    typ: FieldType<'a>,
    name: &'a str,
}

/// Parses a given rule string to a corresponding `ParseRule` object.
pub struct RuleParser<'a> {
    scanner: RuleScanner<'a>,
}

enum ParseError<'a> {
    UnexpectedToken(Token<'a>, usize),
    ScanFailed(ScanError),
    BadRegex(regex::Error),
}

impl<'a> From<ScanError> for ParseError<'a> {
    fn from(err: ScanError) -> ParseError<'a> {
        ParseError::ScanFailed(err)
    }
}

impl<'a> From<regex::Error> for ParseError<'a> {
    fn from(err: regex::Error) -> ParseError<'a> {
        ParseError::BadRegex(err)
    }
}

impl<'a> From<(Token<'a>, usize)> for ParseError<'a> {
    fn from((token, pos): (Token<'a>, usize)) -> ParseError<'a> {
        ParseError::UnexpectedToken(token, pos)
    }
}

impl<'a> RuleParser<'a> {
    pub fn new(rule: &'a str) -> RuleParser<'a> {
        RuleParser { scanner: RuleScanner::new(rule) }
    }

    pub fn parse(&mut self) -> Result<ParseRule<'a>, ParseError> {
        match try!(self.scan()) {
            (Token::Regex(re), _) => {
                Ok(ParseRule {
                    re: try!(regex::Regex::new(re)),
                    fields: try!(self.parse_fields()),
                })
            }
            (token, pos) => err!((token, pos)),
        }
    }

    fn parse_fields(&mut self) -> Result<Vec<Field<'a>>, ParseError> {
        let (token, pos) = try!(self.scan());
        if token != Token::WS {
            err!((token, pos))
        }

        let mut fields = Vec::new();
        loop {
            let name = match try!(self.scan()) {
                (Token::FieldName(n), _) => n,
                (token, pos) => err!((token, pos)),
            };

            // Allowed continuations: field type, sep ',' or EOF.
            let (token, pos) = try!(self.scan());
            let typ = match token {
                Token::TypeInt => FieldType::Int,
                Token::TypeUInt => FieldType::UInt,
                Token::TypeFloat => FieldType::Float,
                Token::TypeDateTime(p) => FieldType::DateTime(p),
                Token::Comma | Token::EOF => FieldType::Str,
                _ => err!((token, pos)),
            };

            fields.push(Field {
                name: name,
                typ: typ,
            });

            if token == Token::EOF {
                break;
            }
        }
        Ok(fields)
    }

    #[inline]
    fn scan(&mut self) -> Result<(Token<'a>, usize), ScanError> {
        self.scanner.scan()
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Token<'a> {
    Regex(&'a str),
    FieldName(&'a str),
    TypeInt,
    TypeUInt,
    TypeFloat,
    TypeDateTime(&'a str),
    Comma,
    WS, // white spaces: ' ', '\t'
    EOF,
}

#[derive(Eq, PartialEq, Debug)]
enum ScanError {
    IllegalSymbol {
        pos: usize,
        symbol: char,
        token: &'static str,
    },
    UnexpectedEndOfRule,
}

type ScanResult<'a> = Result<Token<'a>, ScanError>;

/// Reads tokens from rule string one by one.
struct RuleScanner<'a> {
    rule: &'a str,
    reader: RuleReader<'a>,
}

impl<'a> RuleScanner<'a> {
    fn new(rule: &'a str) -> RuleScanner<'a> {
        RuleScanner {
            rule: rule,
            reader: RuleReader::new(rule),
        }
    }

    fn scan(&mut self) -> Result<(Token<'a>, usize), ScanError> {
        let ch0 = match self.reader.read_char() {
            Some(c) => c,
            None => return Ok((Token::EOF, self.reader.pos)), 
        };
        let pos = self.reader.pos;
        let token = match ch0 {
            '/' => try!(self.scan_regex()),
            ' ' | '\t' => try!(self.scan_whitespace()),
            ':' => try!(self.scan_field_type()),
            ',' => Token::Comma,
            _ => {
                if !self.is_ident_symbol(ch0, true) {
                    return Err(ScanError::IllegalSymbol {
                        pos: self.reader.pos,
                        symbol: ch0,
                        token: "field name",
                    });
                }
                try!(self.scan_field_name())
            }
        };

        Ok((token, pos))
    }

    fn scan_field_name(&mut self) -> ScanResult<'a> {
        let start_pos = self.reader.pos - 1; // first symbol is already read
        loop {
            match self.reader.read_char() {
                Some(ch) => {
                    if !self.is_ident_symbol(ch, false) {
                        self.reader.unread();
                        break;
                    }
                }
                None => break,
            }
        }

        let end_pos = self.reader.pos;
        Ok(Token::FieldName(&self.rule[start_pos..end_pos]))
    }

    fn scan_field_type(&mut self) -> ScanResult<'a> {
        match self.reader.read_char() {
            Some('i') => {
                self.reader.unread();
                try!(self.scan_word("int"));
                return Ok(Token::TypeInt);
            }
            Some('u') => {
                self.reader.unread();
                try!(self.scan_word("uint"));
                return Ok(Token::TypeUInt);
            }
            Some('f') => {
                self.reader.unread();
                try!(self.scan_word("float"));
                return Ok(Token::TypeFloat);
            }
            Some('d') => {
                try!(self.scan_symbol('t', "datetime"));
                return self.scan_dt_pattern();
            }
            Some(ch) => {
                return Err(ScanError::IllegalSymbol {
                    pos: self.reader.pos,
                    symbol: ch,
                    token: "field type",
                })
            }
            None => return Err(ScanError::UnexpectedEndOfRule),
        };
    }

    fn scan_dt_pattern(&mut self) -> ScanResult<'a> {
        Ok(Token::TypeDateTime(try!(self.scan_between('[', ']', "datetime pattern"))))
    }

    fn scan_whitespace(&mut self) -> ScanResult<'a> {
        loop {
            match self.reader.read_char() {
                Some(' ') | Some('\t') => continue,
                None => break,
                _ => {
                    self.reader.unread();
                    break;
                }
            }
        }

        Ok(Token::WS)
    }

    fn scan_regex(&mut self) -> ScanResult<'a> {
        Ok(Token::Regex(try!(self.scan_until('/'))))
    }

    fn scan_symbol(&mut self, symbol: char, token: &'static str) -> Result<(), ScanError> {
        match self.reader.read_char() {
            Some(ch) => {
                if ch != symbol {
                    return Err(ScanError::IllegalSymbol {
                        pos: self.reader.pos,
                        symbol: ch,
                        token: token,
                    });
                }
            }
            None => return Err(ScanError::UnexpectedEndOfRule),
        }
        Ok(())
    }

    fn scan_word(&mut self, word: &'static str) -> Result<(), ScanError> {
        for wch in word.chars() {
            try!(self.scan_symbol(wch, word));
        }
        Ok(())
    }

    fn scan_until(&mut self, symbol: char) -> Result<&'a str, ScanError> {
        if symbol == '\\' {
            panic!(r"consume_until() is not implemented for reading until '\'");
        }

        let start_pos = self.reader.pos;
        let mut prev = match self.reader.read_char() {
            Some(ch) => {
                if ch == symbol {
                    return Ok("");
                }
                ch
            }
            None => return Err(ScanError::UnexpectedEndOfRule),
        };

        loop {
            if let Some(ch) = self.reader.read_char() {
                if ch == symbol && prev != '\\' {
                    break;
                }
                prev = ch;
            } else {
                return Err(ScanError::UnexpectedEndOfRule);
            }
        }

        let end_pos = self.reader.pos - 1;
        Ok(&self.rule[start_pos..end_pos])
    }

    fn scan_between(&mut self,
                    l: char,
                    r: char,
                    token: &'static str)
                    -> Result<&'a str, ScanError> {
        try!(self.scan_symbol(l, token));
        self.scan_until(r)
    }

    fn is_ident_symbol(&self, ch0: char, is_first: bool) -> bool {
        let is_alpha = ('a' <= ch0 && ch0 <= 'z') || ('A' <= ch0 && ch0 <= 'Z');
        let can_be_first = is_alpha || (ch0 == '_');
        can_be_first || (!is_first && '0' <= ch0 && ch0 <= '9')
    }
}

/// Reads chars from rule string one by one and allows unreading of the last one.
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

    fn read_char(&mut self) -> Option<char> {
        if self.buffered {
            self.buffered = false;
            self.pos += 1;
            return Some(self.cur);
        }

        self.cur = match self.rule.next() {
            Some(c) => c,
            None => return None,
        };
        self.pos += 1;
        Some(self.cur)
    }

    fn unread(&mut self) {
        if self.buffered {
            panic!("Cannot unread twice!");
        }
        if self.pos == 0 {
            panic!("Cannot unread without reading before!");
        }
        self.pos -= 1;
        self.buffered = true;
    }
}

#[cfg(test)]
mod tests {
    use super::{RuleScanner, ScanError, Token};

    #[test]
    fn scan_regex() {
        let mut s = RuleScanner::new("/");
        let scanned = s.scan();
        assert_eq!(Err(ScanError::UnexpectedEndOfRule), scanned);

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

        let mut s = RuleScanner::new(" field_name");
        let scanned = s.scan();
        assert_eq!(Ok((Token::WS, 1)), scanned);

        let scanned = s.scan();
        assert_eq!(Ok((Token::FieldName("field_name"), 2)), scanned);
    }

    #[test]
    fn scan_field_name() {
        for rule in &["field1", "field2,", "field_three ", "field4:", "_f5", "_6", "___"] {
            let mut s = RuleScanner::new(rule);
            let scanned = s.scan();
            let expected = rule.trim_matches(&[' ', ',', ':'] as &[_]);
            assert_eq!(Ok((Token::FieldName(expected), 1)), scanned);
        }
    }

    #[test]
    fn scan_dt() {
        let mut s = RuleScanner::new("time:dt[%H:%m:%s]");
        assert_eq!(Ok((Token::FieldName("time"), 1)), s.scan());
        assert_eq!(Ok((Token::TypeDateTime("%H:%m:%s"), 5)), s.scan());
    }
}
