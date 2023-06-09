use nom::{
  branch::alt,
  bytes::complete::{tag, take_while, take_while1},
  combinator::{map, peek},
  error::{convert_error, VerboseError},
  multi::{fold_many0, separated_list0},
  sequence::{delimited, separated_pair},
  IResult,
};
use std::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub enum Jsonish<'a> {
  Object(Vec<(&'a str, Jsonish<'a>)>),
  Array(Vec<Jsonish<'a>>),
  Value(&'a str),
}

impl Jsonish<'_> {
  pub fn sort_by_name(&mut self) {
    match self {
      Jsonish::Value(_) => {}
      Jsonish::Object(xs) => xs.sort_by_key(|x| x.0),
      Jsonish::Array(xs) => xs.iter_mut().for_each(|x| x.sort_by_name()),
    }
  }

  pub fn sort_by_value(&mut self, name: &str) {
    let qname = format!("\"{}\"", name);
    match self {
      Jsonish::Value(_) => {}
      Jsonish::Object(xs) => xs.iter_mut().for_each(|(_, x)| x.sort_by_value(name)),
      Jsonish::Array(xs) => {
        xs.iter_mut().for_each(|x| x.sort_by_value(name));
        xs.sort_by(|a, b| match (a, b) {
          (Jsonish::Object(a), Jsonish::Object(b)) => {
            let a = a.iter().find(|(key, _)| *key == qname).map(|x| x.0);
            let b = b.iter().find(|(key, _)| *key == qname).map(|x| x.0);
            match (a, b) {
              (Some(a), Some(b)) => a.cmp(b),
              _ => Ordering::Equal,
            }
          }
          _ => Ordering::Equal,
        })
      }
    }
  }
}

impl ToString for Jsonish<'_> {
  fn to_string(&self) -> String {
    let mut buf = String::new();
    self.format(&mut buf, "  ", 0, false);
    buf
  }
}

impl Jsonish<'_> {
  fn format(&self, buf: &mut String, indent: &str, level: usize, apply_initial_indent: bool) {
    let print_indent =
      |level: usize, buf: &mut String| (0..level).for_each(|_| buf.push_str(indent));

    if apply_initial_indent {
      print_indent(level, buf);
    }

    match self {
      Jsonish::Value(x) => buf.push_str(x),
      Jsonish::Array(xs) if xs.is_empty() => buf.push_str("[]"),
      Jsonish::Array(xs) => {
        buf.push_str("[\n");
        xs.iter().enumerate().for_each(|(i, x)| {
          x.format(buf, indent, level + 1, true);
          if i < xs.len() - 1 {
            buf.push_str(",\n")
          }
        });
        buf.push_str("\n");
        print_indent(level, buf);
        buf.push_str("]");
      }
      Jsonish::Object(xs) if xs.is_empty() => buf.push_str("{}"),
      Jsonish::Object(xs) => {
        buf.push_str("{\n");
        xs.iter().enumerate().for_each(|(i, (key, val))| {
          print_indent(level + 1, buf);
          buf.push_str(key);
          buf.push_str(": ");
          val.format(buf, indent, level + 1, false);
          if i < xs.len() - 1 {
            buf.push_str(",\n")
          }
        });
        buf.push_str("\n");
        print_indent(level, buf);
        buf.push_str("}");
      }
    }
  }
}

pub type Result<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

pub fn parse(input: &str) -> std::result::Result<Jsonish, String> {
  match jsonish()(input) {
    Ok((_, node)) => Ok(node),
    Err(nom::Err::Error(e)) => Err(convert_error(input, e)),
    Err(nom::Err::Failure(e)) => Err(convert_error(input, e)),
    Err(nom::Err::Incomplete(_)) => panic!("unexpected incomplete error"),
  }
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
  use super::Jsonish;
  use super::Jsonish::*;

  #[test]
  fn parse() {
    for (input, expected) in parser_tests() {
      let actual = super::parse(input);
      assert_eq!(
        actual.as_ref(),
        Ok(&expected),
        "expected: {:?}\n  actual: {:?}\n   input: `{}`\n",
        expected,
        actual,
        input.replace("\n", "\\n"),
      );
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
