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
  let args = Args::parse();
  match parse(&read_input(&args)?) {
    Err(e) => {
      eprintln!("{}", e);
      exit(1);
    }

    Ok(mut node) => {
      if args.sort_by_name {
        node.sort_by_name();
      }

      if let Some(name) = args.sort_by_value.as_ref() {
        node.sort_by_value(name);
      }

      let mut output = node.to_string();
      output.push('\n');
      write_output(&args, &output)?;

      Ok(())
    }
  }
}

fn read_input(args: &Args) -> io::Result<String> {
  if let Some(path) = args.file.as_ref() {
    fs::read_to_string(path)
  } else {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
  }
}

fn write_output(args: &Args, output: &str) -> io::Result<()> {
  if let Some(path) = args.file.as_ref() {
    fs::write(path, output)
  } else {
    print!("{}", output);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
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
    let path = temp.path().to_str().unwrap().to_owned();
    temp.write(b"{ }")?;
    temp.flush()?;

    let output = Command::new("cargo")
      .args(["run", "--quiet", "--", &path])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?
      .wait_with_output()?;

    assert_eq!("", String::from_utf8_lossy(&output.stdout).to_string());
    assert_eq!("", String::from_utf8_lossy(&output.stderr).to_string());
    assert!(output.status.success());
    assert_eq!(&fs::read_to_string(&path)?, "{}\n");
    Ok(())
  }

  #[test]
  fn can_sort_by_name() -> Result<(), Box<dyn Error>> {
    let mut temp = NamedTempFile::new()?;
    let path = temp.path().to_str().unwrap().to_owned();
    temp.write(r#"{"1":0,"0":0}"#.as_bytes())?;
    temp.flush()?;

    let output = Command::new("cargo")
      .args(["run", "--quiet", "--", "--sort-by-name", &path])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?
      .wait_with_output()?;

    assert_eq!("", String::from_utf8_lossy(&output.stdout).to_string());
    assert_eq!("", String::from_utf8_lossy(&output.stderr).to_string());
    assert!(output.status.success());
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
    let path = temp.path().to_str().unwrap().to_owned();
    temp.write(r#"[{"x":1},{"x":0}]"#.as_bytes())?;
    temp.flush()?;

    let output = Command::new("cargo")
      .args(["run", "--quiet", "--", "--sort-by-value", "x", &path])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?
      .wait_with_output()?;

    assert_eq!("", String::from_utf8_lossy(&output.stdout).to_string());
    assert_eq!("", String::from_utf8_lossy(&output.stderr).to_string());
    assert!(output.status.success());
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
