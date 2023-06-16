use crate::node::Node::{self, Array, Object, Value};
use std::cmp::Ordering;

impl Node<'_> {
  pub fn sort_by_name(&mut self) {
    match self {
      Value(_) => {}
      Object(xs) => {
        xs.iter_mut().for_each(|(_, x)| x.sort_by_name());
        xs.sort_by_key(|x| unquote(x.0));
      }
      Array(xs) => xs.iter_mut().for_each(Self::sort_by_name),
    }
  }

  pub fn sort_by_value(&mut self, name: &str) {
    match self {
      Value(_) => {}
      Object(xs) => xs.iter_mut().for_each(|(_, x)| x.sort_by_value(name)),
      Array(xs) => {
        xs.iter_mut().for_each(|x| x.sort_by_value(name));
        xs.sort_by(|a, b| {
          if let (Object(a), Object(b)) = (a, b) {
            if let (Some(a), Some(b)) = (find_key(a, &name), find_key(b, &name)) {
              return unquote(a).cmp(unquote(b));
            }
          }
          return Ordering::Equal;
        })
      }
    }
  }
}

fn find_key<'a>(members: &Vec<(&'a str, Node<'a>)>, key: &str) -> Option<&'a str> {
  let qname = format!("\"{}\"", key);
  members.iter().find(|(k, _)| *k == qname).map(|x| x.0)
}

fn unquote(s: &str) -> &str {
  if s.len() > 1 && s.starts_with("\"") && s.ends_with("\"") {
    &s[1..s.len() - 1]
  } else {
    s
  }
}

#[cfg(test)]
mod tests {
  use super::Node::*;

  #[test]
  fn sort_by_name() {
    let tests = vec![
      (Value("1"), Value("1")),
      (Object(vec![]), Object(vec![])),
      (
        Object(vec![("1", Value("a"))]),
        Object(vec![("1", Value("a"))]),
      ),
      (
        Object(vec![("1", Value("a")), ("2", Value("b"))]),
        Object(vec![("1", Value("a")), ("2", Value("b"))]),
      ),
      (
        Object(vec![("2", Value("b")), ("1", Value("a"))]),
        Object(vec![("1", Value("a")), ("2", Value("b"))]),
      ),
      (
        Object(vec![("\"a \"", Value("x")), ("\"a\"", Value("x"))]),
        Object(vec![("\"a\"", Value("x")), ("\"a \"", Value("x"))]),
      ),
      (
        Object(vec![
          ("2", Value("b")),
          ("1", Value("a")),
          ("3", Object(vec![("1", Value("one")), ("0", Value("zero"))])),
        ]),
        Object(vec![
          ("1", Value("a")),
          ("2", Value("b")),
          ("3", Object(vec![("0", Value("zero")), ("1", Value("one"))])),
        ]),
      ),
      (
        Object(vec![
          ("2", Value("b")),
          ("1", Value("a")),
          (
            "3",
            Array(vec![Object(vec![
              ("1", Value("one")),
              ("0", Value("zero")),
            ])]),
          ),
        ]),
        Object(vec![
          ("1", Value("a")),
          ("2", Value("b")),
          (
            "3",
            Array(vec![Object(vec![
              ("0", Value("zero")),
              ("1", Value("one")),
            ])]),
          ),
        ]),
      ),
      (Array(vec![]), Array(vec![])),
      (
        Array(vec![Object(vec![
          ("1", Value("one")),
          ("0", Value("zero")),
        ])]),
        Array(vec![Object(vec![
          ("0", Value("zero")),
          ("1", Value("one")),
        ])]),
      ),
      (
        Array(vec![Object(vec![
          ("1", Value("one")),
          (
            "0",
            Array(vec![Object(vec![("y", Value("yy")), ("x", Value("xx"))])]),
          ),
        ])]),
        Array(vec![Object(vec![
          (
            "0",
            Array(vec![Object(vec![("x", Value("xx")), ("y", Value("yy"))])]),
          ),
          ("1", Value("one")),
        ])]),
      ),
    ];

    for (mut actual, expected) in tests {
      actual.sort_by_name();
      assert_eq!(actual, expected);
    }
  }
}
