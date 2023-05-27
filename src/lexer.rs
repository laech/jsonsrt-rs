use std::{
  fmt::Debug,
  io::{self, Bytes, Read},
  iter::Peekable,
};

#[derive(PartialEq)]
enum Token {
  BeginObject(usize),
  EndObject(usize),
  BeginArray(usize),
  EndArray(usize),
  NameSeparator(usize),
  ValueSeparator(usize),
  Value(usize, Vec<u8>),
}

impl Debug for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::BeginObject(offset) => f.debug_tuple("BeginObject").field(offset).finish(),
      Self::EndObject(offset) => f.debug_tuple("EndObject").field(offset).finish(),
      Self::BeginArray(offset) => f.debug_tuple("BeginArray").field(offset).finish(),
      Self::EndArray(offset) => f.debug_tuple("EndArray").field(offset).finish(),
      Self::NameSeparator(offset) => f.debug_tuple("NameSeparator").field(offset).finish(),
      Self::ValueSeparator(offset) => f.debug_tuple("ValueSeparator").field(offset).finish(),
      Self::Value(offset, value) => f
        .debug_tuple("Value")
        .field(offset)
        .field(&String::from_utf8_lossy(&value))
        .finish(),
    }
  }
}

impl Token {
  fn from(offset: usize, b: u8) -> Option<Token> {
    match b {
      b'{' => Some(Self::BeginObject(offset)),
      b'}' => Some(Self::EndObject(offset)),
      b'[' => Some(Self::BeginArray(offset)),
      b']' => Some(Self::EndArray(offset)),
      b':' => Some(Self::NameSeparator(offset)),
      b',' => Some(Self::ValueSeparator(offset)),
      _ => None,
    }
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
      Ok(b) => match Token::from(self.offset, b) {
        Some(token) => {
          self.offset += 1;
          return Some(Ok(token));
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
            let offset = self.offset;
            self.offset += self.buffer.len();
            return Some(Ok(Token::Value(offset, self.buffer.clone())));
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
          let offset = self.offset;
          self.offset += self.buffer.len();
          return Some(Ok(Token::Value(offset, self.buffer.clone())));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Lexer;
  use super::Token;
  use super::Token::*;
  use std::io;
  use std::io::Read;

  #[test]
  fn lexer() {
    for (input, output) in lexer_tests() {
      assert_eq!(
        read_all_tokens(input).unwrap(),
        output,
        "\n input: `{}`\n",
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
      (b"{", vec![BeginObject(0)]),
      (b"}", vec![EndObject(0)]),
      (b"[", vec![BeginArray(0)]),
      (b"]", vec![EndArray(0)]),
      (b":", vec![NameSeparator(0)]),
      (b",", vec![ValueSeparator(0)]),
      (b"\"\"", vec![Value(0, b"\"\"".to_vec())]),
      (b" \"hello\"", vec![Value(1, b"\"hello\"".to_vec())]),
      (b"123", vec![Value(0, b"123".to_vec())]),
      (b"123 ", vec![Value(0, b"123".to_vec())]),
      (b"{}", vec![BeginObject(0), EndObject(1)]),
      (b"[]", vec![BeginArray(0), EndArray(1)]),
      (
        b"{\"a\": 1}",
        vec![
          BeginObject(0),
          Value(1, b"\"a\"".to_vec()),
          NameSeparator(4),
          Value(6, b"1".to_vec()),
          EndObject(7),
        ],
      ),
      (
        b"[true, null]",
        vec![
          BeginArray(0),
          Value(1, b"true".to_vec()),
          ValueSeparator(5),
          Value(7, b"null".to_vec()),
          EndArray(11),
        ],
      ),
    ]
  }
}
