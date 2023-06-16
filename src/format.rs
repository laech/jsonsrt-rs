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

#[cfg(test)]
mod tests {
  use crate::parse::parse;

  #[test]
  fn format() {
    let tests = vec![
      ("null", "null"),
      (" true", "true"),
      ("false ", "false"),
      (" 1 ", "1"),
      ("\t-2", "-2"),
      ("-3e10\n", "-3e10"),
      ("{}", "{}"),
      ("[]", "[]"),
      (
        r#"{"a":"hello"}"#,
        r#"{
  "a": "hello"
}"#,
      ),
      (
        r#"{"a":"hello", "b":  [1, 2 , false]}"#,
        r#"{
  "a": "hello",
  "b": [
    1,
    2,
    false
  ]
}"#,
      ),
      (
        r#"["a", "hello", null, { "i": "x"}, -1.000 ]"#,
        r#"[
  "a",
  "hello",
  null,
  {
    "i": "x"
  },
  -1.000
]"#,
      ),
    ];

    for (input, expected) in tests {
      let actual = parse(input).map(|x| x.to_string());
      assert_eq!(
        actual.as_ref(),
        Ok(&expected.to_owned()),
        "\n input: `{}`\n",
        input.replace("\n", "\\n"),
      );
    }
  }
}
