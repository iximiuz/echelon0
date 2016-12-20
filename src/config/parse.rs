use std::str;

use nom::{alphanumeric, multispace};


//   rule config
//     _ plugin_section _ (_ plugin_section)* _ <LogStash::Config::AST::Config>
//   end
//
//   rule plugin_section
//     plugin_type _ "{"
//       _ (branch_or_plugin _)*
//     "}"
//     <LogStash::Config::AST::PluginSection>
//   end
//
//   rule branch_or_plugin
//     branch / plugin
//   end
//
//   rule plugin
//     name _ "{"
//       _
//       attributes:( attribute (whitespace _ attribute)*)?
//       _
//     "}"
//     <LogStash::Config::AST::Plugin>
//   end
//
//   rule name
//     (
//       ([A-Za-z0-9_-]+ <LogStash::Config::AST::Name>)
//       / string
//     )
//   end
//
//   rule double_quoted_string
//     ( '"' ( '\"' / !'"' . )* '"' <LogStash::Config::AST::String>)
//   end
//
//   rule single_quoted_string
//     ( "'" ( "\\'" / !"'" . )* "'" <LogStash::Config::AST::String>)
//   end
//
//   rule string
//     double_quoted_string / single_quoted_string
//   end


// named!(plugin_type,
//        alt!(tag!("input") | tag!("filter") | tag!("output")));

named!(comments,
       map!(many1!(preceded!(opt!(multispace),
                             delimited!(tag!("#"), take_until!("\n"), tag!("\n")))),
            |_| b""));

named!(blank, map!(many0!(alt!(multispace | comments)), |_| b""));

named!(double_quoted<String>,
       delimited!(tag!("\""),
                  fold_many0!(map_res!(alt!(tag!(r#"\""#) | take_until_either!(r#"\""#)),
                                       str::from_utf8),
                              String::new(),
                              |mut acc: String, item| {
                                if item == r#"\""# {
                                    acc.push('"');
                                } else {
                                    acc.push_str(item);
                                }
                                acc
                            } ),
                  tag!("\"")));

named!(single_quoted<String>,
       delimited!(tag!("'"),
                  fold_many0!(map_res!(alt!(tag!(r"\'") | take_until_either!(r"\'")),
                                       str::from_utf8),
                              String::new(),
                              |mut acc: String, item| {
                                if item == r"\'" {
                                    acc.push('\'');
                                } else {
                                    acc.push_str(item);
                                }
                                acc
                            } ),
                  tag!("'")));


named!(string<String>, alt!(single_quoted | double_quoted));

named!(name<String>,
       alt!(fold_many1!(map_res!(alt!(alphanumeric | tag!("-") | tag!("_")),
                                 str::from_utf8),
                        String::new(), |mut acc: String, item| { acc.push_str(item); acc } )
            | string));


#[cfg(test)]
mod tests {
    use super::{blank, comments, double_quoted, single_quoted, name};
    use nom::{IResult};

    #[test]
    fn test_parse_blank() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), blank(config));
    }

    #[test]
    fn test_parse_comments() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), comments(config));
    }

    #[test]
    fn test_parse_single_quoted_string() {
        let quoted = "     'foo bar baz'     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "foo bar baz".to_string()),
                   single_quoted(quoted));

        let quoted_escaped = r"     'foo \'bar\' baz'     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], r"foo 'bar' baz".to_string()),
                   single_quoted(quoted_escaped));
    }

    #[test]
    fn test_parse_double_quoted_string() {
        let quoted = r#"     "foo bar baz"     "#.trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "foo bar baz".to_string()),
                   double_quoted(quoted));

        let quoted_escaped = r#"     "foo \"bar\" baz"     "#.trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], r#"foo "bar" baz"#.to_string()),
                   double_quoted(quoted_escaped));
    }

    #[test]
    fn test_name() {
        let simple_name = "     example123     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "example123".to_string()),
                   name(simple_name));

        let dashed_name = "     ex_amp_le-123     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "ex_amp_le-123".to_string()),
                   name(dashed_name));

        let not_a_name = "     foo&bar     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b"&bar"[..], "foo".to_string()),
                   name(not_a_name));

        let double_quoted_name = r#"     "foo&bar"     "#.trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "foo&bar".to_string()),
                   name(double_quoted_name));

        let single_quoted_name = "     'foo&bar'     ".trim().as_bytes();
        assert_eq!(IResult::Done(&b""[..], "foo&bar".to_string()),
                   name(single_quoted_name));
    }
}
