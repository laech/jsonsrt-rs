use std::{
  io::{self, Bytes, Read},
  iter::Peekable,
};

#[derive(Debug, PartialEq)]
enum TokenValue {
  BeginObject,
  EndObject,
  BeginArray,
  EndArray,
  NameSeparator,
  ValueSeparator,
  Value(Vec<u8>),
}

impl TryFrom<u8> for TokenValue {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      b'{' => Ok(Self::BeginObject),
      b'}' => Ok(Self::EndObject),
      b'[' => Ok(Self::BeginArray),
      b']' => Ok(Self::EndArray),
      b':' => Ok(Self::NameSeparator),
      b',' => Ok(Self::ValueSeparator),
      _ => Err(()),
    }
  }
}

impl ToString for TokenValue {
  fn to_string(&self) -> String {
    match self {
      Self::BeginObject => "BeginObject".to_owned(),
      Self::EndObject => "EndObject".to_owned(),
      Self::BeginArray => "BeginArray".to_owned(),
      Self::EndArray => "EndArray".to_owned(),
      Self::NameSeparator => "NameSeparator".to_owned(),
      Self::ValueSeparator => "ValueSeparator".to_owned(),
      Self::Value(x) => format!("Value({})", String::from_utf8_lossy(&x).to_string()),
    }
  }
}

#[derive(Debug, PartialEq)]
struct Token {
  value: TokenValue,
  offset: usize,
}

impl ToString for Token {
  fn to_string(&self) -> String {
    format!(
      "Token {{ {}, offset: {} }}",
      self.value.to_string(),
      self.offset
    )
  }
}

struct Lexer<R: Read> {
  data: Peekable<Bytes<R>>,
  buffer: Vec<u8>,
  offset: usize,
}

impl<R: Read> Lexer<R> {
  fn new(data: Bytes<R>) -> Lexer<R> {
    Lexer {
      data: data.peekable(),
      buffer: Vec::new(),
      offset: 0,
    }
  }

  fn next(&mut self) -> Option<io::Result<Token>> {
    if let Err(e) = self.skip_spaces()? {
      return Some(Err(e));
    }
    match self.data.next()? {
      Err(e) => return Some(Err(e)),
      Ok(b) => match TokenValue::try_from(b) {
        Ok(value) => {
          let offset = self.offset;
          self.offset += 1;
          return Some(Ok(Token { value, offset }));
        }
        _ => {
          self.buffer.clear();
          self.buffer.push(b);
          if b == b'"' {
            return self.read_string();
          } else {
            return self.read_value();
          }
        }
      },
    }
  }

  fn skip_spaces(&mut self) -> Option<io::Result<()>> {
    loop {
      match self.data.peek()? {
        Ok(b) if (*b as char).is_whitespace() => {
          self.data.next();
          self.offset += 1;
        }
        Ok(_) => return Some(Ok(())),
        Err(_) => return Some(Err(self.data.next()?.unwrap_err())),
      }
    }
  }

  fn read_string(&mut self) -> Option<io::Result<Token>> {
    let mut escape = false;
    loop {
      match self.data.next() {
        Some(Ok(b)) => {
          self.buffer.push(b);
          if b == b'\\' {
            escape = !escape;
          } else if !escape && b == b'"' {
            let value = TokenValue::Value(self.buffer.clone());
            let offset = self.offset;
            self.offset += self.buffer.len();
            return Some(Ok(Token { value, offset }));
          }
        }
        Some(Err(e)) => return Some(Err(e)),
        None => return Some(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))),
      }
    }
  }

  fn read_value(&mut self) -> Option<io::Result<Token>> {
    loop {
      match self.data.peek() {
        Some(Err(_)) => return Some(Err(self.data.next()?.unwrap_err())),
        Some(Ok(b))
          if !(*b as char).is_whitespace()
            && *b != b'{'
            && *b != b'}'
            && *b != b'['
            && *b != b']'
            && *b != b','
            && *b != b':' =>
        {
          self.buffer.push(self.data.next()?.unwrap());
        }
        None | Some(Ok(_)) => {
          let value = TokenValue::Value(self.buffer.clone());
          let offset = self.offset;
          self.offset += self.buffer.len();
          return Some(Ok(Token { value, offset }));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Lexer;
  use super::Token;
  use super::TokenValue::*;
  use std::io;
  use std::io::Read;

  #[test]
  fn lexer() {
    for (input, output) in lexer_tests() {
      let tokens = read_all_tokens(input).unwrap();

      // t.Fatalf("\nexpected: %s\n     got: %s\n   input: %s",
      // 	test.output, tokens, string(test.input))
      assert_eq!(
        tokens,
        output,
        "\nexpected: {}\n     got: {}\n   input: {}\n",
        output
          .iter()
          .map(|t| t.to_string())
          .collect::<Vec<_>>()
          .join(", "),
        tokens
          .iter()
          .map(|t| t.to_string())
          .collect::<Vec<_>>()
          .join(", "),
        String::from_utf8_lossy(input)
      )
    }
  }

  fn read_all_tokens(data: &'static [u8]) -> io::Result<Vec<Token>> {
    let mut lexer = Lexer::new(data.bytes());
    let mut tokens = Vec::new();
    loop {
      match lexer.next() {
        None => return Ok(tokens),
        Some(Ok(token)) => tokens.push(token),
        Some(Err(e)) => return Err(e),
      }
    }
  }

  fn lexer_tests() -> Vec<(&'static [u8], Vec<Token>)> {
    vec![
      (b"{", vec![begin_object(0)]),
      (b"}", vec![end_object(0)]),
      (b"[", vec![begin_array(0)]),
      (b"]", vec![end_array(0)]),
      (b":", vec![name_separator(0)]),
      (b",", vec![value_separator(0)]),
      (b"\"\"", vec![value(0, b"\"\"")]),
      (b" \"hello\"", vec![value(1, b"\"hello\"")]),
      (b"123", vec![value(0, b"123")]),
      (b"123 ", vec![value(0, b"123")]),
      (b"{}", vec![begin_object(0), end_object(1)]),
      (b"[]", vec![begin_array(0), end_array(1)]),
      (
        b"{\"a\": 1}",
        vec![
          begin_object(0),
          value(1, b"\"a\""),
          name_separator(4),
          value(6, b"1"),
          end_object(7),
        ],
      ),
      (
        b"[true, null]",
        vec![
          begin_array(0),
          value(1, b"true"),
          value_separator(5),
          value(7, b"null"),
          end_array(11),
        ],
      ),
    ]
  }

  fn begin_object(offset: usize) -> Token {
    Token {
      value: BeginObject,
      offset,
    }
  }

  fn end_object(offset: usize) -> Token {
    Token {
      value: EndObject,
      offset,
    }
  }

  fn begin_array(offset: usize) -> Token {
    Token {
      value: BeginArray,
      offset,
    }
  }

  fn end_array(offset: usize) -> Token {
    Token {
      value: EndArray,
      offset,
    }
  }

  fn name_separator(offset: usize) -> Token {
    Token {
      value: NameSeparator,
      offset,
    }
  }

  fn value_separator(offset: usize) -> Token {
    Token {
      value: ValueSeparator,
      offset,
    }
  }

  fn value(offset: usize, value: &'static [u8]) -> Token {
    Token {
      value: Value(value.into()),
      offset,
    }
  }
}
