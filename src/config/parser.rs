use std::str;

use nom::{alphanumeric, is_digit, multispace};

use super::ast::*;

//   rule config
//     _ plugin_section _ (_ plugin_section)* _ <LogStash::Config::AST::Config>
//   end
//
//   rule plugin_section
//     plugin_type _ "{" _ (branch_or_plugin _)* "}"
//     <LogStash::Config::AST::PluginSection>
//   end
//
named!(block<Block>,
    do_parse!(
        tag!("{") >>
        bps: many0!(delimited!(blank0, branch_or_plugin, blank0)) >>
        tag!("}") >>
        (bps)
    )
);

named!(plugin_type<PluginType>,
    alt!(
        tag!("input")  => { |_| PluginType::Input  }
      | tag!("filter") => { |_| PluginType::Filter }
      | tag!("output") => { |_| PluginType::Output }
    )
);

named!(branch_or_plugin<BranchOrPlugin>,
    alt!(
        branch => { |b| BranchOrPlugin::Branch(b) }
      | plugin => { |p| BranchOrPlugin::Plugin(p) }
    )
);

named!(plugin<Plugin>,
    do_parse!(
        name: name >>
        blank0     >>
        tag!("{")  >>
        blank0     >>
// TODO: attributes:( attribute (whitespace _ attribute)*)?
        blank0     >>
        tag!("}")  >>
        (Plugin::new(name))
    )
);

// rule branch
//   if (_ else_if)* (_ else)?
//   <LogStash::Config::AST::Branch>
// end
named!(branch<Branch>,
    do_parse!(
        case_if: case_if >>
        // TODO: case_else_if: case_else_if >>
        // TODO: case_else: case_else >>
        (Branch::new())
    )
);

// rule if
//     "if" _ condition _ "{" _ (branch_or_plugin _)* "}"
//     <LogStash::Config::AST::If>
// end
named!(case_if<Case>,
    do_parse!(
        tag!("if")   >>
        blank0       >>
        c: condition >>
        blank0       >>
        b: block     >>
        (Case::new(c, b))
    )
);

// rule else_if
//   "else" _ "if" _ condition _ "{" _ (branch_or_plugin _)* "}"
//   <LogStash::Config::AST::Elsif>
// end

// rule else
//   "else" _ "{" _ (branch_or_plugin _)* "}"
//   <LogStash::Config::AST::Else>
// end

named!(bool_operator<BoolOperator>,
    alt!(
        tag!("and")  => { |_| BoolOperator::And }
      | tag!("or")   => { |_| BoolOperator::Or  }
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

named!(
/// Parses compound conditions.
///
/// Internally, to deal with boolean operators precedences the algorithm
/// reassembles the last node instead of doing one-step look ahead.
///
/// Trying to build a tree from the next cases:
///
/// ```bash
/// a && b
/// a || b
/// a && b || c
/// a || b && c
/// a && b && c
/// a || b && c || d
/// a && b || c && d
/// ```
///
/// Logstash rule: `expression (_ boolean_operator _ expression)*`.
,
    condition<Condition>,
    do_parse!(
        head: bool_expr >>
        tail: many0!(
            tuple!(
                delimited!(blank0, bool_operator, blank0),
                bool_expr
            )
        ) >>
        (parse_condition(Condition::from(head), tail))
    )
);

fn parse_condition(head: Condition, tail: Vec<(BoolOperator, BoolExpr)>) -> Condition {
    let mut cond = head;

    for part in tail {
        let next_op = part.0;
        let next_expr = Box::new(Condition::from(part.1));

        cond = match cond {
            Condition::Leaf(_) => Condition::Branch(next_op, Box::new(cond), next_expr),
            Condition::Branch(op, lhs, rhs) => {
                if op.precedence() >= next_op.precedence() {
                    // Wrap
                    Condition::Branch(next_op,
                                      Box::new(Condition::Branch(op, lhs, rhs)), // reassemble cond
                                      next_expr)
                } else {
                    // Unwrap and rewrap
                    Condition::Branch(op,
                                      lhs,
                                      Box::new(Condition::Branch(next_op, rhs, next_expr)))
                }
            }
        };
    }

    cond
}

named!(
/// Parses an atomic boolean expression.
///
/// It is rather an operand for a compound boolean expression (called `condition`), but
/// it's called `expression` to mimic the original idea from Logstash configs.
,
    bool_expr<BoolExpr>,
    alt!(
        complete!(parens_expr)
      | complete!(negative_expr)
// TODO: in_expr
// TODO: not_in_expr
      | complete!(compare_expr)
// TODO: re_expr
      | complete!(rvalue_expr)
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

named!(
/// Parses either `!<parenthesized expr>` or `!<selector expr>`.
,
    negative_expr<BoolExpr>,
    preceded!(
        preceded!(tag!("!"), blank0),
        alt!(
            parens_expr => { |expr: BoolExpr| expr.not() }
          | selector    => { |sel| BoolExpr::from(Rvalue::from(sel)).not() }
        )
    )
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
/// Parses a (r)value which will be then converted to a `bool` value.
///
/// Does it use `ruby`'s conversions rules?
,
    rvalue_expr<BoolExpr>,
    map!(rvalue, |v| BoolExpr::Rvalue(Box::new(v)))
);

named!(
/// Consumes multiline comments and replaces them with an empty value.
,
    comments,
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

named!(selector<Selector>,
    map!(
        many1!(
            map_res!(
                delimited!(tag!("["), take_until_either!("],"), tag!("]")),
                str::from_utf8
            )
        ),
        { |elems: Vec<&str>| Selector::new(elems.iter().map(|e| e.to_string()).collect()) }
    )
);

// rule rvalue
//   string / number / selector / array / method_call / regexp
// end
named!(rvalue<Rvalue>,
    alt!(
        number   => { |v| Rvalue::from(v) }
      | string   => { |v| Rvalue::from(v) }
      | selector => { |v| Rvalue::from(v) }
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
        assert_eq!(IResult::Done(&b""[..], Rvalue::from(123.0)),
                   rvalue(&b"123"[..]));

        assert_eq!(IResult::Done(&b""[..], Rvalue::from("foobar")),
                   rvalue(&b"'foobar'"[..]));

        let sel = Selector::new(vec!["foo".to_string()]);
        assert_eq!(IResult::Done(&b""[..], Rvalue::from(sel)),
                   rvalue(&b"[foo]"[..]));

        let sel = Selector::new(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(IResult::Done(&b""[..], Rvalue::from(sel)),
                   rvalue(&b"[foo][bar]"[..]));

        // TODO: array, method_call, regexp
    }

    #[test]
    fn test_condition_leaf() {
        let expr = bool_expr(&b"1 > 2"[..]).unwrap().1;
        let cond = Condition::from(expr);
        assert_eq!(IResult::Done(&b""[..], cond),
                   condition(&b"1 > 2"[..]));
    }

    #[test]
    fn test_condition_branch() {
        let expr1 = bool_expr(&b"1 > 2"[..]).unwrap().1;
        let expr2 = bool_expr(&b"'foo' != 'bar'"[..]).unwrap().1;
        let op = bool_operator(&b"and"[..]).unwrap().1;
        let cond = Condition::Branch(op,
                                     Box::new(Condition::from(expr1)),
                                     Box::new(Condition::from(expr2)));
        assert_eq!(IResult::Done(&b""[..], cond),
                   condition(&b"1 > 2 and 'foo' != 'bar'"[..]));
    }

    #[test]
    fn test_condition_compound_1() {
        let expr1 = bool_expr(&b"1 > 2"[..]).unwrap().1;
        let expr2 = bool_expr(&b"'foo' != 'bar'"[..]).unwrap().1;
        let expr3 = bool_expr(&b"42 == [sel]"[..]).unwrap().1;
        let op_and = bool_operator(&b"and"[..]).unwrap().1;
        let op_or = bool_operator(&b"or"[..]).unwrap().1;

        let cond = Condition::Branch(op_or,
                                     Box::new(Condition::Branch(op_and,
                                                                Box::new(Condition::from(expr1)),
                                                                Box::new(Condition::from(expr2)))),
                                     Box::new(Condition::from(expr3)));
        assert_eq!(IResult::Done(&b""[..], cond),
                   condition(&b"1 > 2 and 'foo' != 'bar' or 42 == [sel]"[..]));
    }

    #[test]
    fn test_condition_compound_2() {
        let expr1 = bool_expr(&b"1 > 2"[..]).unwrap().1;
        let expr2 = bool_expr(&b"'foo' != 'bar'"[..]).unwrap().1;
        let expr3 = bool_expr(&b"42 == [sel]"[..]).unwrap().1;
        let op_and = bool_operator(&b"and"[..]).unwrap().1;
        let op_or = bool_operator(&b"or"[..]).unwrap().1;
        let cond = Condition::Branch(op_or,
                                     Box::new(Condition::from(expr1)),
                                     Box::new(Condition::Branch(op_and,
                                                                Box::new(Condition::from(expr2)),
                                                                Box::new(Condition::from(expr3)))));
        assert_eq!(IResult::Done(&b""[..], cond),
                   condition(&b"1 > 2 or 'foo' != 'bar' and 42 == [sel]"[..]));
    }

    // TODO: #[test]
    // fn test_condition_compound_3() {
    //     // a && b || c && d
    //     // a || b && c || d
    // }

    #[test]
    fn test_bool_expr_rvalue() {
        assert_eq!(IResult::Done(&b""[..], BoolExpr::from(Rvalue::from(1.0))),
                   bool_expr(&b"1"[..]));
    }

    #[test]
    fn test_bool_expr_compare() {
        use self::CompareOperator::*;

        // TODO: test other rvalues
        for sides in &[("1", "0"),
                       ("'foo'", "'bar'"),
                       ("\"foo\"", "\"bar\""),
                       ("[foo][bar]", "[baz]")] {
            for pattern in &["{lhs} {op} {rhs}",
                             "{lhs}{op}{rhs}",
                             "{lhs} {op}{rhs}",
                             "{lhs}{op} {rhs}",
                             "{lhs}   {op}   {rhs}",
                             "{lhs}\n{op}\n  \n  {rhs}"] {
                for op in &[Eq, Ne, Gt, Lt, Ge, Le] {
                    let lhs = rvalue(sides.0.as_bytes()).unwrap().1;
                    let rhs = rvalue(sides.1.as_bytes()).unwrap().1;
                    let expr = BoolExpr::Compare(*op, Box::new(lhs), Box::new(rhs));
                    let config = pattern.replace("{lhs}", sides.0)
                        .replace("{op}", op.to_string())
                        .replace("{rhs}", sides.1);
                    assert_eq!(IResult::Done(&b""[..], expr),
                               bool_expr(config.as_bytes()));
                }
            }
        }
    }

    #[test]
    fn test_bool_expr_parens() {
        // TODO: add test cases.
        let expr =
            BoolExpr::Parens(
                Box::new(
                    Condition::from(
                        BoolExpr::Compare(
                            CompareOperator::Gt,
                            Box::new(Rvalue::from(1.0)),
                            Box::new(Rvalue::from(2.0))
                        )
                    )
                )
            );
        assert_eq!(IResult::Done(&b""[..], expr),
                   bool_expr(&b"(1 > 2)"[..]));
    }

    #[test]
    fn test_bool_expr_negative() {
        // not (some parens expr or condition)
        let expr =
            BoolExpr::Parens(
                Box::new(
                    Condition::from(
                        BoolExpr::Compare(
                            CompareOperator::Gt,
                            Box::new(Rvalue::from(1.0)),
                            Box::new(Rvalue::from(2.0))
                        )
                    )
                )
            ).not();
        assert_eq!(IResult::Done(&b""[..], expr),
                   bool_expr(&b"!(1 > 2)"[..]));

        // not selector
        let expr = bool_expr(&b"[foo][bar]"[..]).unwrap().1.not();
        assert_eq!(IResult::Done(&b""[..], expr),
                   bool_expr(&b"![foo][bar]"[..]));
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
}
