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
          if let (Some(a), Some(b)) = (find_value(a, &name), find_value(b, &name)) {
            return unquote(a).cmp(unquote(b));
          }
          return Ordering::Equal;
        })
      }
    }
  }
}

fn find_value<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
  if let Object(xs) = node {
    let qname = format!("\"{}\"", key);
    xs.iter().find_map(|(k, v)| match v {
      Value(x) if *k == qname => Some(*x),
      _ => None,
    })
  } else {
    None
  }
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

  #[test]
  fn sort_by_value() {
    let tests = [
      ("", Value("1"), Value("1")),
      ("", Object(vec![]), Object(vec![])),
      ("", Array(vec![]), Array(vec![])),
      (
        "name",
        Array(vec![
          Object(vec![("\"name\"", Value("1"))]),
          Object(vec![("\"name\"", Value("2"))]),
        ]),
        Array(vec![
          Object(vec![("\"name\"", Value("1"))]),
          Object(vec![("\"name\"", Value("2"))]),
        ]),
      ),
      (
        "name",
        Array(vec![
          Object(vec![("\"name\"", Value("2"))]),
          Object(vec![("\"name\"", Value("1"))]),
        ]),
        Array(vec![
          Object(vec![("\"name\"", Value("1"))]),
          Object(vec![("\"name\"", Value("2"))]),
        ]),
      ),
      (
        "name",
        Object(vec![(
          "\"name\"",
          Array(vec![
            Object(vec![("\"name\"", Value("2"))]),
            Object(vec![("\"name\"", Value("1"))]),
          ]),
        )]),
        Object(vec![(
          "\"name\"",
          Array(vec![
            Object(vec![("\"name\"", Value("1"))]),
            Object(vec![("\"name\"", Value("2"))]),
          ]),
        )]),
      ),
      (
        "a",
        Array(vec![
          Object(vec![("\"a\"", Value("1"))]),
          Object(vec![("\"a\"", Value("2"))]),
          Object(vec![("\"a\"", Value("0"))]),
        ]),
        Array(vec![
          Object(vec![("\"a\"", Value("0"))]),
          Object(vec![("\"a\"", Value("1"))]),
          Object(vec![("\"a\"", Value("2"))]),
        ]),
      ),
      (
        "a",
        Array(vec![
          Object(vec![("\"a\"", Value("\"cmd+h c\""))]),
          Object(vec![("\"a\"", Value("\"cmd+h\""))]),
        ]),
        Array(vec![
          Object(vec![("\"a\"", Value("\"cmd+h\""))]),
          Object(vec![("\"a\"", Value("\"cmd+h c\""))]),
        ]),
      ),
      (
        "a",
        Array(vec![
          Object(vec![("\"a\"", Value("1"))]),
          Object(vec![("\"a\"", Value("0"))]),
          Object(vec![(
            "\"b\"",
            Array(vec![
              Object(vec![("\"a\"", Value("2"))]),
              Object(vec![("\"a\"", Value("1"))]),
            ]),
          )]),
        ]),
        Array(vec![
          Object(vec![("\"a\"", Value("0"))]),
          Object(vec![("\"a\"", Value("1"))]),
          Object(vec![(
            "\"b\"",
            Array(vec![
              Object(vec![("\"a\"", Value("1"))]),
              Object(vec![("\"a\"", Value("2"))]),
            ]),
          )]),
        ]),
      ),
    ];

    for (key, mut actual, expected) in tests {
      actual.sort_by_value(key);
      assert_eq!(actual, expected);
    }
  }
}
