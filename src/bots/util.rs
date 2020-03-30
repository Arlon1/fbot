pub fn tokenize_args(s: &str) -> Option<Vec<String>> {
  use nom::{
    branch::alt,
    bytes::complete::{is_not, take_till1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, value, verify},
    error::ParseError,
    multi::{fold_many0, separated_list},
    sequence::{delimited, preceded, terminated},
    IResult,
  };
  fn parse_args<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Vec<String>, E> {
    let literally = verify(is_not("\\\""), |s: &str| !s.is_empty());
    let escaped = preceded(
      char('\\'),
      alt((value("\\", char('\\')), value("\"", char('"')))),
    );
    let quoted_string = delimited(
      char('"'),
      fold_many0(alt((literally, escaped)), String::new(), |mut s, f| {
        s.push_str(f);
        s
      }),
      char('"'),
    );
    let no_ws_string = take_till1(|c: char| c.is_ascii_whitespace());
    let args = separated_list(
      multispace1,
      alt((quoted_string, map(no_ws_string, |s: &str| s.to_owned()))),
    );
    terminated(args, multispace0)(input)
  }
  match parse_args::<()>(s) {
    Ok(("", output)) => Some(output),
    _ => None,
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_tokenization() {
    fn test(input: &str, expected: Option<&[&str]>) {
      assert_eq!(
        tokenize_args(input),
        expected.map(|e| e.into_iter().map(|&s| s.to_owned()).collect())
      )
    }
    test("", Some(&[]));
    test("  ", Some(&[]));
    test("a  b c", Some(&["a", "b", "c"]));
    test("omega\"lul", Some(&["omega\"lul"]));
    test(r#"a "asdf\\\"" "b""#, Some(&["a", "asdf\\\"", "b"]));
    test("\"", Some(&["\""]));
    test("\\ ", Some(&["\\"]));
    test(" asdf", None);
  }
}
