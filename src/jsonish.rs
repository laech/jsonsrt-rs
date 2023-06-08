use nom::{
  branch::alt,
  bytes::complete::{tag, take_while, take_while1},
  combinator::{map, peek},
  error::VerboseError,
  multi::{fold_many0, separated_list0},
  sequence::{delimited, separated_pair},
  IResult,
};

#[derive(Debug, PartialEq)]
enum Jsonish<'a> {
  Object(Vec<(&'a str, Jsonish<'a>)>),
  Array(Vec<Jsonish<'a>>),
  Value(&'a str),
}

type Result<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

fn parse(input: &str) -> Result<Jsonish> {
  jsonish()(input)
}

fn jsonish() -> impl Fn(&str) -> Result<Jsonish> {
  |input| ws(alt((object(), array(), value())))(input)
}

fn array() -> impl Fn(&str) -> Result<Jsonish> {
  |input| {
    map(
      delimited(
        ws(tag("[")),
        separated_list0(ws(tag(",")), jsonish()),
        ws(tag("]")),
      ),
      Jsonish::Array,
    )(input)
  }
}

fn object() -> impl Fn(&str) -> Result<Jsonish> {
  |input| {
    map(
      delimited(
        ws(tag("{")),
        separated_list0(
          ws(tag(",")),
          separated_pair(string(), ws(tag(":")), jsonish()),
        ),
        ws(tag("}")),
      ),
      Jsonish::Object,
    )(input)
  }
}

fn value() -> impl Fn(&str) -> Result<Jsonish> {
  |input| {
    map(
      |input| {
        if peek(tag::<&str, &str, VerboseError<&str>>("\""))(input).is_ok() {
          string()(input)
        } else {
          stringish()(input)
        }
      },
      Jsonish::Value,
    )(input)
  }
}

fn stringish() -> impl Fn(&str) -> Result<&str> {
  |input| take_while1(|x: char| !x.is_whitespace() && !",:{}[]".contains(x))(input)
}

fn string() -> impl Fn(&str) -> Result<&str> {
  |input0| {
    let (input, count) = delimited(
      tag("\""),
      fold_many0(
        alt((
          take_while1(|x| !"\\\"".contains(x)),
          tag("\\\""),
          take_while1(|x| x != '\"'),
        )),
        || 0,
        |acc, xs: &str| acc + xs.len(),
      ),
      tag("\""),
    )(input0)?;
    Ok((input, &input0[0..count + 2]))
  }
}

fn ws<'a, O>(
  mut parse: impl FnMut(&'a str) -> Result<'a, O> + 'a,
) -> impl FnMut(&'a str) -> Result<'a, O> {
  delimited(space(), move |input| parse(input), space())
}

fn space() -> impl Fn(&str) -> Result<&str> {
  |input| take_while(|c: char| c.is_whitespace())(input)
}

#[cfg(test)]
mod tests {
  use nom::error::convert_error;

  use super::Jsonish;
  use super::Jsonish::*;

  #[test]
  fn parse() {
    for (input, expected) in parser_tests() {
      match super::parse(input) {
        Ok(actual) => {
          let expected = ("", expected);
          if actual != expected {
            assert!(
              false,
              "expected: {:?}\n  actual: {:?}\n   input: `{}`\n",
              expected,
              actual,
              input.replace("\n", "\\n"),
            );
          }
        }
        Err(e) => {
          assert!(
            false,
            "error: {}\ninput: `{}`\n",
            if let nom::Err::Error(e) = e {
              convert_error(input, e)
            } else {
              e.to_string()
            },
            input.replace("\n", "\\n")
          );
        }
      }
    }
  }

  fn parser_tests() -> Vec<(&'static str, Jsonish<'static>)> {
    vec![
      ("true", Value("true")),
      (" true", Value("true")),
      (" true ", Value("true")),
      ("true ", Value("true")),
      ("false\t", Value("false")),
      ("\nfalse\t", Value("false")),
      ("null", Value("null")),
      ("1", Value("1")),
      ("-2", Value("-2")),
      ("-3.4", Value("-3.4")),
      ("5e6", Value("5e6")),
      ("7.00", Value("7.00")),
      ("-8.900", Value("-8.900")),
      (" -10", Value("-10")),
      (" 11 ", Value("11")),
      ("12\t", Value("12")),
      ("\n\t13\n", Value("13")),
      ("\"\"", Value("\"\"")),
      (" \"\"", Value("\"\"")),
      (" \"\" ", Value("\"\"")),
      ("\"\" ", Value("\"\"")),
      (" \" \" ", Value("\" \"")),
      (" \"a b\" ", Value("\"a b\"")),
      (" \"\\\"a b\" ", Value("\"\\\"a b\"")),
      (" \"a\\\" b\" ", Value("\"a\\\" b\"")),
      (" \"a b\\\"\" ", Value("\"a b\\\"\"")),
      (" \"a\nb\" ", Value("\"a\nb\"")),
      (" \"\ta \nb false\" ", Value("\"\ta \nb false\"")),
      ("[]", Array(vec![])),
      (" []", Array(vec![])),
      (" [] ", Array(vec![])),
      (" [ ] ", Array(vec![])),
      ("[ ] ", Array(vec![])),
      ("[] ", Array(vec![])),
      ("{}", Object(vec![])),
      ("{} ", Object(vec![])),
      ("{ } ", Object(vec![])),
      (" { } ", Object(vec![])),
      (" {} ", Object(vec![])),
      (" {}", Object(vec![])),
      ("[1] ", Array(vec![Value("1")])),
      ("[ 1, false] ", Array(vec![Value("1"), Value("false")])),
      (
        "[ 0E-18 , true ] ",
        Array(vec![Value("0E-18"), Value("true")]),
      ),
      (
        "[ 2 , true , {}] ",
        Array(vec![Value("2"), Value("true"), Object(vec![])]),
      ),
      (
        "[\t{},{} , {} , {}\n, []] ",
        Array(vec![
          Object(vec![]),
          Object(vec![]),
          Object(vec![]),
          Object(vec![]),
          Array(vec![]),
        ]),
      ),
      ("{\"hi\" : true} ", Object(vec![("\"hi\"", Value("true"))])),
      (
        "{\"hello world\" : {}} ",
        Object(vec![("\"hello world\"", Object(vec![]))]),
      ),
      ("{\"bob\" : []} ", Object(vec![("\"bob\"", Array(vec![]))])),
      (
        "{\"bob\" : { \"ja\tck\": [1, -3, true, {\"a\" : false}]}} ",
        Object(vec![(
          "\"bob\"",
          Object(vec![(
            "\"ja\tck\"",
            Array(vec![
              Value("1"),
              Value("-3"),
              Value("true"),
              Object(vec![("\"a\"", Value("false"))]),
            ]),
          )]),
        )]),
      ),
      (
        "[ 10.000000 , null, { \"ja\tck\": [1, -3, true, {\"a\" : false}]} ]",
        Array(vec![
          Value("10.000000"),
          Value("null"),
          Object(vec![(
            "\"ja\tck\"",
            Array(vec![
              Value("1"),
              Value("-3"),
              Value("true"),
              Object(vec![("\"a\"", Value("false"))]),
            ]),
          )]),
        ]),
      ),
      ("\\u001b\\u007f", Value("\\u001b\\u007f")),
      (
        "\"^[^@]+@[^@.]+\\.[^@]+$\"",
        Value("\"^[^@]+@[^@.]+\\.[^@]+$\""),
      ),
    ]
  }
}
