mod hash;
mod meta;
mod output;
mod scanner;

use clap::Parser;
use meta::Meta;
use output::{json::print_json, tree::print_tree};
use std::path::PathBuf;

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
    match Meta::new(path) {
      Ok(metadata) => roots.push(metadata),
      Err(error) => {
        eprintln!("Failed to get metadata for {}: {}", path.display(), error);
      }
    }
  }

  if args.json {
    match print_json(&roots, args.include_hashes, args.recursive) {
      Ok(_) => (),
      Err(error) => {
        eprintln!("Failed to convert the metadata to JSON: {}", error);
      }
    };

    return;
  }

  print_tree(&roots, args.include_hashes, args.recursive);
}
