use std::error;
use std::fmt;
use std::str::Chars;

extern crate regex;

pub struct ParseRule<'a> {
    pub re: regex::Regex,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum FieldType<'a> {
    Int,
    UInt,
    Float,
    DateTime(&'a str),
    Str,
}

pub struct Field<'a> {
    pub typ: FieldType<'a>,
    pub name: &'a str,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken {
        token: String,
        pos: usize,
    },
    ScanFailed(ScanError),
    BadRegex(regex::Error),
    CapturesFieldsMismatch {
        captures_count: usize,
        fields_count: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedToken { ref token, ref pos } => {
                write!(f, "Found unexpected token '{}' at pos {}", token, pos)
            }
            Error::ScanFailed(ref err) => write!(f, "Cannot split parse rule on tokens: {}", err),
            Error::BadRegex(ref err) => write!(f, "Bad regex pattern provided: {}", err),
            Error::CapturesFieldsMismatch { ref captures_count, ref fields_count } => {
                write!(f,
                       "Capture group count [{}] is not equal to fields count [{}]",
                       captures_count,
                       fields_count)
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnexpectedToken { .. } => "unexpected token found",
            Error::ScanFailed(ref err) => err.description(),
            Error::BadRegex(ref err) => err.description(),
            Error::CapturesFieldsMismatch { .. } => "capture groups don't match fields",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ScanFailed(ref err) => Some(err),
            Error::BadRegex(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<ScanError> for Error {
    fn from(err: ScanError) -> Error {
        Error::ScanFailed(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        Error::BadRegex(err)
    }
}

impl<'a> From<(Token<'a>, usize)> for Error {
    fn from((token, pos): (Token<'a>, usize)) -> Error {
        Error::UnexpectedToken {
            token: format!("{:?}", token),
            pos: pos,
        }
    }
}

/// Parses a given rule string to a corresponding `ParseRule` object.
pub struct RuleParser<'a> {
    scanner: RuleScanner<'a>,
}


impl<'a> RuleParser<'a> {
    pub fn new(rule: &'a str) -> RuleParser {
        RuleParser { scanner: RuleScanner::new(rule) }
    }

    pub fn parse(&mut self) -> Result<ParseRule<'a>, Error> {
        match try!(self.scan()) {
            (Token::Regex(re), _) => {
                let re = try!(regex::Regex::new(re));
                let fields = try!(self.parse_fields());
                if re.captures_len() != fields.len() + 1 {
                    Err(Error::CapturesFieldsMismatch {
                        captures_count: re.captures_len(),
                        fields_count: fields.len(),
                    })
                } else {
                    Ok(ParseRule {
                        re: re,
                        fields: fields,
                    })
                }
            }
            (token, pos) => err!((token, pos)),
        }
    }

    fn parse_fields(&mut self) -> Result<Vec<Field<'a>>, Error> {
        let (token, pos) = try!(self.scan());
        if token != Token::WS {
            err!((token, pos))
        }

        let mut fields = Vec::new();
        let mut expect_sep = false;
        loop {
            if expect_sep {
                let (token, pos) = try!(self.scan());
                if token != Token::Comma {
                    err!((token, pos))
                }
            }

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
            expect_sep = token != Token::Comma;
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
pub enum ScanError {
    IllegalSymbol {
        pos: usize,
        symbol: char,
        token: &'static str,
    },
    UnexpectedEndOfRule,
}

impl From<(char, usize, &'static str)> for ScanError {
    fn from((symbol, pos, token): (char, usize, &'static str)) -> ScanError {
        ScanError::IllegalSymbol {
            pos: pos,
            symbol: symbol,
            token: token,
        }
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScanError::IllegalSymbol { pos, symbol, token } => {
                write!(f,
                       "Illegal symbol '{}' found at pos {} while reading '{}' token",
                       symbol,
                       pos,
                       token)
            }
            ScanError::UnexpectedEndOfRule => write!(f, "Parse rule ended unexpectedly"),
        }
    }
}

impl error::Error for ScanError {
    fn description(&self) -> &str {
        match *self {
            ScanError::IllegalSymbol { pos: _, symbol: _, token: _ } => {
                "illegal symbol found in rule"
            }
            ScanError::UnexpectedEndOfRule => "unexpected end of rule occurred",
        }
    }
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
                    err!((ch0, self.reader.pos, "FieldName"))
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
                Ok(Token::TypeInt)
            }
            Some('u') => {
                self.reader.unread();
                try!(self.scan_word("uint"));
                Ok(Token::TypeUInt)
            }
            Some('f') => {
                self.reader.unread();
                try!(self.scan_word("float"));
                Ok(Token::TypeFloat)
            }
            Some('d') => {
                try!(self.scan_symbol('t', "datetime"));
                self.scan_dt_pattern()
            }
            Some(ch) => err!((ch, self.reader.pos, "FieldType")),
            None => err!(ScanError::UnexpectedEndOfRule),
        }
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
                    err!((ch, self.reader.pos, token))
                }
            }
            None => err!(ScanError::UnexpectedEndOfRule),
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
            None => err!(ScanError::UnexpectedEndOfRule),
        };

        loop {
            if let Some(ch) = self.reader.read_char() {
                if ch == symbol && prev != '\\' {
                    break;
                }
                prev = ch;
            } else {
                err!(ScanError::UnexpectedEndOfRule)
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
    use super::{FieldType, RuleParser, RuleScanner, Error, ScanError, Token};

    #[test]
    fn parse() {
        let mut parser = RuleParser::new(r"/(\d+)\s(\w)/ time:uint,url");
        let rule = parser.parse().unwrap();
        assert_eq!(r"(\d+)\s(\w)", rule.re.as_str());
        assert_eq!(2, rule.fields.len());
        assert_eq!("time", rule.fields[0].name);
        assert_eq!(FieldType::UInt, rule.fields[0].typ);
        assert_eq!("url", rule.fields[1].name);
        assert_eq!(FieldType::Str, rule.fields[1].typ);
    }

    #[test]
    fn parse_no_fields() {
        let mut parser = RuleParser::new(r"/some_re/ ");
        match parser.parse() {
            Err(Error::UnexpectedToken { token, pos }) => {
                assert_eq!("EOF", token);
                assert_eq!(10, pos);
            }
            _ => unreachable!(), 
        }
    }

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
