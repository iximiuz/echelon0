use std::str;

use nom::{alphanumeric, is_digit, multispace};

use super::ast;

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

// named!(plugin_section,
//        plagin_type);


named!(plugin_type<ast::PluginType>,
    alt!(
        tag!("input")  => { |_| ast::PluginType::Input }
      | tag!("filter") => { |_| ast::PluginType::Filter }
      | tag!("output") => { |_| ast::PluginType::Output }
    )
);

named!(plugin<ast::Plugin>,
    do_parse!(
        name: name >>
        blank0     >>
        tag!("{")  >>
        blank0     >>  // TODO: attrs
        tag!("}")  >>
        (ast::Plugin::new(name))
    )
);

// rule branch
//   if (_ else_if)* (_ else)?
//   <LogStash::Config::AST::Branch>
// end
// named!(branch<ast::Branch>,
//     do_parse!(
//         case_if: case_if >>
//         // TODO: case_else_if: case_else_if >>
//         // TODO: case_else: case_else >>
//         (ast::Branch{})
//     )
// );

// rule if
//     "if" _ condition _ "{" _ (branch_or_plugin _)* "}"
//     <LogStash::Config::AST::If>
// end

// named!(case_if<ast::BranchOrPlugin>,
// );

// rule condition
//   expression (_ boolean_operator _ expression)*
//   <LogStash::Config::AST::Condition>
// end
// named!(condition<ast::Condition>,
// );

// rule expression
//   (
//       ("(" _ condition _ ")")
//     / negative_expression
//     / in_expression
//     / not_in_expression
//     / compare_expression
//     / regexp_expression
//     / rvalue
//   ) <LogStash::Config::AST::Expression>
// end

//  rule boolean_operator
//    ("and" / "or" / "xor" / "nand")
//    <LogStash::Config::AST::BooleanOperator>
//  end

named!(comments,
    map!(
        many1!(
            preceded!(
                opt!(multispace),
                delimited!(tag!("#"), take_until!("\n"), tag!("\n"))
            )
        ),
        |_| b""
    )
);

named!(blank0, map!(many0!(alt!(multispace | comments)), |_| b""));

named!(number<f64>,
    do_parse!(
        minus:      opt!(tag!("-"))        >>
        integer:    take_while1!(is_digit) >>
        fractional: opt!(complete!(preceded!(tag!("."), take_while!(is_digit)))) >>
        (parse_f64(minus, integer, fractional))
    )
);

fn parse_f64(minus: Option<&[u8]>, integer: &[u8], fractional: Option<&[u8]>) -> f64 {
    // Since this function is only for internal usage with the `number` parser
    // we assume that input data is always valid, so we can unwrap() fearlessly.
    let mut res = String::new();
    if let Some(_) = minus {
        res.push('-');
    }

    res.push_str(str::from_utf8(integer).unwrap());
    if let Some(f) = fractional {
        res.push('.');
        res.push_str(str::from_utf8(f).unwrap());
    }

    res.parse().unwrap()
}

named!(double_quoted<String>,
    delimited!(
        tag!("\""),
        fold_many0!(
            map_res!(
                alt!(tag!(r#"\""#) | take_until_either!(r#"\""#)),
                str::from_utf8
            ),
            String::new(),
            |mut acc: String, item| {
                if item == r#"\""# {
                    acc.push('"');
                } else {
                    acc.push_str(item);
                }
                acc
            }
        ),
        tag!("\"")
    )
);

named!(single_quoted<String>,
    delimited!(
        tag!("'"),
        fold_many0!(
            map_res!(
                alt!(tag!(r"\'") | take_until_either!(r"\'")),
                str::from_utf8
            ),
            String::new(),
            |mut acc: String, item| {
                if item == r"\'" {
                    acc.push('\'');
                } else {
                    acc.push_str(item);
                }
                acc
            }
        ),
        tag!("'")
    )
);

named!(string<String>, alt!(single_quoted | double_quoted));

named!(name<String>,
    alt!(
        fold_many1!(
            map_res!(alt!(alphanumeric | tag!("-") | tag!("_")), str::from_utf8),
            String::new(), |mut acc: String, item| { acc.push_str(item); acc }
        )
        | string
    )
);

// rule rvalue
//   string / number / selector / array / method_call / regexp
// end
// named!(rvalue<ast::Rvalue,
//     alt!(
//     )
// );

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{IResult, ErrorKind};

    #[test]
    fn test_parse_blank0() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), blank0(config));
    }

    #[test]
    fn test_parse_comments() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), comments(config));
    }

    #[test]
    fn test_parse_number() {
        let valid = vec!["0", "123", "-1", "0.", "1.5", "1.123", "-0.42"];
        for x in &valid {
            assert_eq!(IResult::Done(&b""[..], x.parse().unwrap()), number(x.as_bytes()));
        }

        assert_eq!(IResult::Done(&b"abc"[..], -0.123), number(&b"-0.123abc"[..]));
        assert_eq!(IResult::Error(ErrorKind::TakeWhile1), number(&b"+1"[..]));
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

    #[test]
    fn test_plugin() {
        let config = &b"stdin {}"[..];
        assert_eq!(IResult::Done(&b""[..], ast::Plugin::new("stdin".to_string())),
                   plugin(config));

        let config = &b"file {\n\n    \n}"[..];
        assert_eq!(IResult::Done(&b""[..], ast::Plugin::new("file".to_string())),
                   plugin(config));
    }
}
