use clap::Parser;
use std::{
  fs,
  io::{self, Read},
  process::exit,
};

mod jsonish;

/// Sort JSON contents
#[derive(Debug, Parser)]
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

  let mut input: String;
  if let Some(path) = args.file.as_ref() {
    input = fs::read_to_string(path)?;
  } else {
    input = String::new();
    io::stdin().read_to_string(&mut input)?;
  }

  match jsonish::parse(&input) {
    Ok(mut node) => {
      if args.sort_by_name {
        node.sort_by_name();
      }
      if let Some(name) = args.sort_by_value {
        node.sort_by_value(&name);
      }

      if let Some(path) = args.file.as_ref() {
        fs::write(path, node.to_string() + "\n")?;
      } else {
        println!("{}", node.to_string())
      }
    }
    Err(e) => {
      eprintln!("{}", e);
      exit(1);
    }
  }

  Ok(())
}
