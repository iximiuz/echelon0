use std::str;

use nom::multispace;


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

// named!(double_quoted,
//        delimited!(tag!("\""),
//                   map!(many0!(alt!(tag!(r#"\""#) | not!(char!('"')))), |_: Vec<&[u8]>| b""),
//                   tag!("\"")));

named!(single_quoted<String>,
       delimited!(tag!("'"),
                  fold_many0!(map_res!(alt!(tag!(r"\'") | take_until_either!(r"\'")), str::from_utf8),
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


// named!(string, alt!(single_quoted | double_quoted));

// named!(name, alt!(re_bytes_match!("^[-A-Za-z0-9_]+$") | tag!("~")));


#[cfg(test)]
mod tests {
    use super::{blank, comments, /* name, string, double_quoted, */ single_quoted};
    use nom::IResult;

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

    // #[test]
    // fn test_parse_double_quoted_string() {
    //     let quoted = br#""foo bar baz""#;
    //     assert_eq!(IResult::Done(&b""[..], &b"foo bar baz"[..]),
    //                double_quoted(quoted));

    //     let quoted_escaped = br#""foo \"bar\" baz""#;
    //     assert_eq!(IResult::Done(&b""[..], &br#"foo \"bar\" baz"#[..]),
    //                double_quoted(quoted_escaped));
    // }

    // #[test]
    // fn test_parse_string() {
    //     let quoted = br#""foo bar baz""#;
    //     assert_eq!(IResult::Done(&b""[..], &b"foo bar baz"[..]), string(quoted));

    //     let quoted_escaped = br#""foo \"bar\" baz""#;
    //     assert_eq!(IResult::Done(&b""[..], &br#"foo \"bar\" baz"#[..]),
    //                string(quoted_escaped));

    //     let quoted = b"'foo bar baz'";
    //     assert_eq!(IResult::Done(&b""[..], &b"foo bar baz"[..]), string(quoted));

    //     let quoted_escaped = br"'foo \'bar\' baz'";
    //     assert_eq!(IResult::Done(&b""[..], &br"foo \'bar\' baz"[..]),
    //                string(quoted_escaped));
    // }

    // #[test]
    // fn test_name() {
    //     let simple_name = b"example123";
    //     assert_eq!(IResult::Done(&b""[..], &b"example123"[..]),
    //                name(simple_name));

    //     let quoted_name = b"\"example123\"";
    //     assert_eq!(IResult::Done(&b""[..], &b"example123"[..]),
    //                name(quoted_name));

    //     let quoted_name = b"'example123'";
    //     assert_eq!(IResult::Done(&b""[..], &b"example123"[..]),
    //                name(quoted_name));
    // }
}
