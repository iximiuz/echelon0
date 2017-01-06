use std::str;

use nom::{alphanumeric, is_digit, multispace};

use super::ast::*;

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

named!(plugin_type<PluginType>,
    alt!(
        tag!("input")  => { |_| PluginType::Input  }
      | tag!("filter") => { |_| PluginType::Filter }
      | tag!("output") => { |_| PluginType::Output }
    )
);

//   rule plugin
//     name _ "{"
//       _
//       attributes:( attribute (whitespace _ attribute)*)?
//       _
//     "}"
//     <LogStash::Config::AST::Plugin>
//   end
named!(plugin<Plugin>,
    do_parse!(
        name: name >>
        blank0     >>
        tag!("{")  >>
        blank0     >>  // TODO: attrs
        tag!("}")  >>
        (Plugin::new(name))
    )
);

// rule branch
//   if (_ else_if)* (_ else)?
//   <LogStash::Config::AST::Branch>
// end
// named!(branch<Branch>,
//     do_parse!(
//         case_if: case_if >>
//         // TODO: case_else_if: case_else_if >>
//         // TODO: case_else: case_else >>
//         (Branch{})
//     )
// );

// rule if
//     "if" _ condition _ "{" _ (branch_or_plugin _)* "}"
//     <LogStash::Config::AST::If>
// end

// named!(case_if<BranchOrPlugin>,
// );

// rule else_if
//   "else" _ "if" _ condition _ "{" _ ( branch_or_plugin _)* "}"
//   <LogStash::Config::AST::Elsif>
// end

// rule else
//   "else" _ "{" _ (branch_or_plugin _)* "}"
//   <LogStash::Config::AST::Else>
// end

named!(bool_operator<BoolOperator>,
    alt!(
        tag!("and")  => { |_| BoolOperator::And  }
      | tag!("or")   => { |_| BoolOperator::Or   }
      | tag!("xor")  => { |_| BoolOperator::Xor  }
      | tag!("nand") => { |_| BoolOperator::Nand }
    )
);

named!(compare_operator<CompareOperator>,
    alt!(
        tag!("==") => { |_| CompareOperator::Eq }
      | tag!("!=") => { |_| CompareOperator::Ne }
      | tag!("<=") => { |_| CompareOperator::Le }
      | tag!(">=") => { |_| CompareOperator::Ge }
      | tag!("<")  => { |_| CompareOperator::Lt }
      | tag!(">")  => { |_| CompareOperator::Gt }
    )
);

// bool_expr (_ bool_operator _ bool_expr)+
named!(condition<Condition>,
    do_parse!(
        head: bool_expr >>
        tail: many1!(
            tuple!(
                delimited!(blank0, bool_operator, blank0),
                bool_expr
            )
        ) >>
        (parse_condition(head, tail))
    )
);

fn parse_condition(head: BoolExpr, tail: Vec<(BoolOperator, BoolExpr)>) -> Condition {
    Condition::Leaf(Box::new(head))
}

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

named!(
/// Parses an atomic boolean expression.
///
/// It is rather an operand for a compound boolean expression (called `condition`), but
/// it's called `expression` to mimic the original idea from Logstash configs.
,
    bool_expr<BoolExpr>,
    alt!(
        rvalue_expr
      | compare_expr
      | parens_expr
    )
);


named!(
/// Parses a (r)value which will be then converted to a `bool` value.
///
/// Does it use `ruby`'s conversions rules?
,
    rvalue_expr<BoolExpr>,
    map!(rvalue, |v| BoolExpr::Rvalue(Box::new(v)))
);

named!(
/// Parses a comparison expression.
///
/// E.g. `some_var > 42` or `foo == bar`.
///
/// Logstash rule: `rvalue _ compare_operator _ rvalue`.
,
    compare_expr<BoolExpr>,
    do_parse!(
        lhs: rvalue          >>
        opt!(blank0)         >>
        op: compare_operator >>
        opt!(blank0)         >>
        rhs: rvalue          >>
        (BoolExpr::Compare(op, Box::new(lhs), Box::new(rhs)))
    )
);

named!(
/// Parses a parenthesized and maybe compound boolean expression (i.e. `condition`).
///
/// E.g. `('foo' in ['foo', 'bar'] and 5 > 6)`.
///
/// Logstash rule: `"(" _ condition _ ")"`.
,
    parens_expr<BoolExpr>,
    do_parse!(
        tag!("(")    >>
        blank0       >>
        c: condition >>
        blank0       >>
        tag!(")")    >>
        (BoolExpr::Parens(Box::new(c)))
    )
);

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

named!(
/// Parses numbers in form \d+(\.\d*)? and produces a float value.
,
    number<f64>,
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

named!(
/// Parses strings (double or single quoted).
,
    string<String>, alt!(single_quoted | double_quoted)
);

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
named!(rvalue<Rvalue>,
    alt!(
        string => { |v| Rvalue::String(v) }
      | number => { |v| Rvalue::Number(v) }
// TODO: add remaining cases
    )
);

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
    fn test_rvalue() {
        assert_eq!(IResult::Done(&b""[..], Rvalue::Number(123.0)),
                   rvalue(&b"123"[..]));

        assert_eq!(IResult::Done(&b""[..], Rvalue::String("foobar".to_string())),
                   rvalue(&b"'foobar'"[..]));
        // TODO: selector, array, method_call, regexp
    }

    #[test]
    fn test_plugin() {
        let config = &b"stdin {}"[..];
        assert_eq!(IResult::Done(&b""[..], Plugin::new("stdin".to_string())),
                   plugin(config));

        let config = &b"file {\n\n    \n}"[..];
        assert_eq!(IResult::Done(&b""[..], Plugin::new("file".to_string())),
                   plugin(config));
    }

    #[test]
    fn test_bool_expr() {
        assert_eq!(IResult::Done(&b""[..],
                   BoolExpr::Rvalue(Box::new(Rvalue::Number(1.0)))),
                   bool_expr(&b"1"[..]));
    }
}
