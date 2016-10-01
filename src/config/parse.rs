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


named!(plugin_type, alt!(tag!("input") | tag!("filter") | tag!("output")));

named!(comments,
       map!(many1!(preceded!(opt!(multispace),
                             delimited!(tag!("#"), take_until!("\n"), tag!("\n")))),
            |_| b""));

named!(blank, map!(many0!(alt!(multispace | comments)), |_| b""));

named!(double_quoted,
	   delimited!(tag!("\""), re_bytes_find!(r#"(\\"|[^"])*"#), tag!("\"")));

named!(single_quoted,
	   delimited!(tag!("'"), re_bytes_find!(r"(\\'|[^'])*"), tag!("'")));

named!(string, alt!(single_quoted | double_quoted));


#[cfg(test)]
mod tests {
    use super::{comments, string};
    use nom::IResult;

    #[test]
    fn test_parse_comments() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), comments(config));
    }

    #[test]
    fn test_parse_strings() {
		let double_quoted = br#""foo bar baz""#;
        assert_eq!(IResult::Done(&b""[..], &b"foo bar baz"[..]), string(double_quoted));

        let double_quoted_escaped = br#""foo \"bar\" baz""#;
        assert_eq!(IResult::Done(&b""[..], &b"foo \"bar\" baz"[..]), string(double_quoted_escaped));
    }
}
