//Kartik Aiyer
use std::error;

mod parse_state;

use crate::parse_state::*;

pub struct Config {
  filename: String,
}

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
  Ok(())
}

