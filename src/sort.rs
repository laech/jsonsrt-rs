use crate::node::Node::{self, Array, Object, Value};
use std::cmp::Ordering;

impl Node<'_> {
  pub fn sort_by_name(&mut self) {
    match self {
      Value(_) => {}
      Object(xs) => xs.sort_by_key(|x| x.0),
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
