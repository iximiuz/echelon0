use nom::multispace;

named!(comments,
       map!(many1!(preceded!(opt!(multispace),
                             delimited!(tag!("#"), take_until!("\n"), tag!("\n")))),
            |_| b""));

named!(blank, map!(many0!(alt!(multispace | comments)), |_| b""));

#[cfg(test)]
mod tests {
    use super::comments;
    use nom::IResult;

    #[test]
    fn test_parse_comments() {
        let config = include_bytes!("../../tests/assets/config/comments.conf");
        assert_eq!(IResult::Done(&b"input {}"[..], &b""[..]), comments(config));
    }
}
