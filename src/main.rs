mod hash;
mod meta;
mod output;
mod scanner;

use clap::Parser;
use output::{json::print_json, tree::print_tree};
use std::path::PathBuf;

use crate::scanner::build_tree;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(long)]
  json: bool,

  #[arg(long, short)]
  recursive: bool,

  #[arg(required = true, num_args = 1..)]
  paths: Vec<PathBuf>,

  #[arg(long = "include-hashes", short)]
  include_hashes: bool,
}

fn main() {
  let mut args = Args::parse();

  args.paths.sort();

  // Collect valid roots first.
  let mut roots = Vec::new();

  for path in &args.paths {
    roots.push(
      match build_tree(path, args.include_hashes, args.recursive) {
        Ok(val) => val,
        Err(error) => {
          eprintln!(
            "Failed to get the entry tree for {}: {}",
            path.display(),
            error
          );
          continue;
        }
      },
    );
  }

  if args.json {
    match print_json(&roots) {
      Ok(_) => (),
      Err(error) => {
        eprintln!("Failed to convert the metadata to JSON: {}", error);
      }
    };

    return;
  }

  print_tree(&roots, args.include_hashes, args.recursive);
}
