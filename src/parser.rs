use std::collections::HashMap;
use std::fmt;

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

pub struct Parser {
}

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse_entry(&self, l: &String) -> Result<Entry, String> {
        Err("FooBar".to_string())
    }
}
