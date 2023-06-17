use clap::Parser;
use parse::parse;
use std::{
  fs,
  io::{self, Read},
  process::exit,
};

mod format;
mod node;
mod parse;
mod sort;

/// Sort JSON contents
#[derive(Debug, Parser, PartialEq)]
#[command(version)]
struct Args {
  /// Sort objects by key names
  #[arg(long)]
  sort_by_name: bool,

  /// Sort object arrays by comparing the values of KEY
  #[arg(long, value_name = "KEY")]
  sort_by_value: Option<String>,

  /// File to process, otherwise uses stdin/stdout
  file: Option<String>,
}

fn main() -> io::Result<()> {
  run(Args::parse())
}

fn run(args: Args) -> io::Result<()> {
  let mut input: String;
  if let Some(path) = args.file.as_ref() {
    input = fs::read_to_string(path)?;
  } else {
    input = String::new();
    io::stdin().read_to_string(&mut input)?;
  }

  match parse(&input) {
    Ok(mut node) => {
      if args.sort_by_name {
        node.sort_by_name();
      }
      if let Some(name) = args.sort_by_value {
        node.sort_by_value(&name);
      }

      let output = node.to_string() + "\n";
      if let Some(path) = args.file.as_ref() {
        fs::write(path, output)?;
      } else {
        print!("{}", output)
      }
    }
    Err(e) => {
      eprintln!("{}", e);
      exit(1);
    }
  }

  Ok(())
}

#[cfg(test)]
mod arg_tests {
  use crate::Args;
  use clap::Parser;

  #[test]
  fn can_parse_file_arg() {
    let args = Args::try_parse_from(["jsonsrt", "xyz"]).unwrap();
    assert_eq!(
      args,
      Args {
        sort_by_name: false,
        sort_by_value: None,
        file: Some("xyz".to_owned())
      }
    );
  }

  #[test]
  fn can_parse_sort_by_name_arg() {
    let args = Args::try_parse_from(["jsonsrt", "--sort-by-name"]).unwrap();
    assert_eq!(
      args,
      Args {
        sort_by_name: true,
        sort_by_value: None,
        file: None
      }
    )
  }

  #[test]
  fn can_parse_sort_by_value_arg() {
    let args = Args::try_parse_from(["jsonsrt", "--sort-by-value", "key"]).unwrap();
    assert_eq!(
      args,
      Args {
        sort_by_name: false,
        sort_by_value: Some("key".to_owned()),
        file: None
      }
    )
  }
}

#[cfg(test)]
mod main_tests {
  use crate::{run, Args};
  use clap::Parser;
  use std::{
    error::Error,
    fs,
    io::{self, Write},
    process::{Command, Stdio},
  };
  use tempfile::NamedTempFile;

  #[test]
  fn can_use_stdin_stdout() -> io::Result<()> {
    let mut proc = Command::new("cargo")
      .args(["run"])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()?;
    proc.stdin.as_mut().unwrap().write(b"{ }")?;
    let output = proc.wait_with_output()?;
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{}\n");
    Ok(())
  }

  #[test]
  fn can_use_file() -> Result<(), Box<dyn Error>> {
    let mut temp = NamedTempFile::new()?;
    temp.write(b"{ }")?;
    temp.flush()?;

    let path = temp.path().to_str().unwrap().to_owned();
    run(Args::try_parse_from(["jsonsrt", &path])?)?;
    assert_eq!(fs::read_to_string(&path)?, "{}\n".to_owned());
    Ok(())
  }

  #[test]
  fn can_sort_by_name() -> Result<(), Box<dyn Error>> {
    let mut temp = NamedTempFile::new()?;
    temp.write(r#"{"1":0,"0":0}"#.as_bytes())?;
    temp.flush()?;

    let path = temp.path().to_str().unwrap().to_owned();
    run(Args::try_parse_from(["jsonsrt", "--sort-by-name", &path])?)?;
    assert_eq!(
      fs::read_to_string(&path)?,
      r#"{
  "0": 0,
  "1": 0
}
"#
      .to_owned()
    );
    Ok(())
  }

  #[test]
  fn can_sort_by_value() -> Result<(), Box<dyn Error>> {
    let mut temp = NamedTempFile::new()?;
    temp.write(r#"[{"x":1},{"x":0}]"#.as_bytes())?;
    temp.flush()?;

    let path = temp.path().to_str().unwrap().to_owned();
    run(Args::try_parse_from([
      "jsonsrt",
      "--sort-by-value",
      "x",
      &path,
    ])?)?;

    assert_eq!(
      fs::read_to_string(&path)?,
      r#"[
  {
    "x": 0
  },
  {
    "x": 1
  }
]
"#
      .to_owned()
    );
    Ok(())
  }
}
