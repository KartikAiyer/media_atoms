// Kartik Aiyer
use std::env;
use std::process;
use media_atoms::*;

fn main() {
  let args: Vec<String> = env::args().collect();

  if( args.len() != 2) {
    eprintln!("Usage: {} <path to file>", args[0]);
    process::exit(1);
  }
  let filename = &args[1];
  println!("Will parse {}", filename);

  let config = Config::new(filename);
  let nodes = run(config);
  println!("{}", nodes)
}
