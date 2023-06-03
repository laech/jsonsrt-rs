use std::{
  io::{self},
  iter::Peekable,
  str::Chars,
};

#[derive(Debug, PartialEq)]
enum Token {
  BeginObject(usize),
  EndObject(usize),
  BeginArray(usize),
  EndArray(usize),
  NameSeparator(usize),
  ValueSeparator(usize),
  Value(usize, String),
}

impl Token {
  fn from(offset: usize, c: char) -> Option<Token> {
    match c {
      '{' => Some(Self::BeginObject(offset)),
      '}' => Some(Self::EndObject(offset)),
      '[' => Some(Self::BeginArray(offset)),
      ']' => Some(Self::EndArray(offset)),
      ':' => Some(Self::NameSeparator(offset)),
      ',' => Some(Self::ValueSeparator(offset)),
      _ => None,
    }
  }
}

struct Lexer<'a> {
  data: Peekable<Chars<'a>>,
  buffer: Vec<char>,
  offset: usize,
}

impl Lexer<'_> {
  fn new(data: Chars) -> Lexer {
    Lexer {
      data: data.peekable(),
      buffer: Vec::new(),
      offset: 0,
    }
  }

  fn next(&mut self) -> Option<io::Result<Token>> {
    self.skip_spaces()?;
    let c = self.data.next()?;
    match Token::from(self.offset, c) {
      Some(token) => {
        self.offset += 1;
        return Some(Ok(token));
      }
      _ => {
        self.buffer.clear();
        self.buffer.push(c);
        if c == '"' {
          return self.read_string();
        } else {
          return self.read_value().map(Ok);
        }
      }
    }
  }

  fn skip_spaces(&mut self) -> Option<()> {
    loop {
      if self.data.peek()?.is_whitespace() {
        self.data.next();
        self.offset += 1;
      } else {
        return Some(());
      }
    }
  }

  fn read_string(&mut self) -> Option<io::Result<Token>> {
    let mut escape = false;
    loop {
      match self.data.next() {
        Some(c) => {
          self.buffer.push(c);
          if c == '\\' {
            escape = !escape;
          } else {
            if !escape && c == '"' {
              let offset = self.offset;
              self.offset += self.buffer.len();
              return Some(Ok(Token::Value(offset, self.buffer.iter().collect())));
            }
            escape = false
          }
        }
        None => return Some(Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))),
      }
    }
  }

  fn read_value(&mut self) -> Option<Token> {
    loop {
      match self.data.peek() {
        Some(c)
          if !c.is_whitespace()
            && *c != '{'
            && *c != '}'
            && *c != '['
            && *c != ']'
            && *c != ','
            && *c != ':' =>
        {
          self.buffer.push(self.data.next()?);
        }
        None | Some(_) => {
          let offset = self.offset;
          self.offset += self.buffer.len();
          return Some(Token::Value(offset, self.buffer.iter().collect()));
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

  #[test]
  fn lexer() {
    for (input, output) in lexer_tests() {
      assert_eq!(
        read_all_tokens(input).unwrap(),
        output,
        "\n input: `{}`\n",
        input
      )
    }
  }

  fn read_all_tokens(data: &'static str) -> io::Result<Vec<Token>> {
    let mut lexer = Lexer::new(data.chars());
    let mut tokens = Vec::new();
    loop {
      match lexer.next() {
        None => return Ok(tokens),
        Some(Ok(token)) => tokens.push(token),
        Some(Err(e)) => return Err(e),
      }
    }
  }

  fn lexer_tests() -> Vec<(&'static str, Vec<Token>)> {
    vec![
      ("{", vec![BeginObject(0)]),
      ("}", vec![EndObject(0)]),
      ("[", vec![BeginArray(0)]),
      ("]", vec![EndArray(0)]),
      (":", vec![NameSeparator(0)]),
      (",", vec![ValueSeparator(0)]),
      ("\"\"", vec![Value(0, "\"\"".to_owned())]),
      (" \"hello\"", vec![Value(1, "\"hello\"".to_owned())]),
      (" \"he\\\"llo\"", vec![Value(1, "\"he\\\"llo\"".to_owned())]),
      ("123", vec![Value(0, "123".to_owned())]),
      ("123 ", vec![Value(0, "123".to_owned())]),
      ("{}", vec![BeginObject(0), EndObject(1)]),
      ("[]", vec![BeginArray(0), EndArray(1)]),
      (
        "{\"a\": 1}",
        vec![
          BeginObject(0),
          Value(1, "\"a\"".to_owned()),
          NameSeparator(4),
          Value(6, "1".to_owned()),
          EndObject(7),
        ],
      ),
      (
        "[true, null]",
        vec![
          BeginArray(0),
          Value(1, "true".to_owned()),
          ValueSeparator(5),
          Value(7, "null".to_owned()),
          EndArray(11),
        ],
      ),
    ]
  }
}
