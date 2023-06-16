use crate::node::Node::{self, Array, Object, Value};

impl ToString for Node<'_> {
  fn to_string(&self) -> String {
    let mut buf = String::new();
    self.format(&mut buf, "  ", 0, false);
    buf
  }
}

impl Node<'_> {
  fn format(&self, buf: &mut String, indent: &str, level: usize, apply_initial_indent: bool) {
    let print_indent =
      |level: usize, buf: &mut String| (0..level).for_each(|_| buf.push_str(indent));

    if apply_initial_indent {
      print_indent(level, buf);
    }

    match self {
      Value(x) => buf.push_str(x),

      Array(xs) if xs.is_empty() => buf.push_str("[]"),
      Array(xs) => {
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

      Object(xs) if xs.is_empty() => buf.push_str("{}"),
      Object(xs) => {
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
