//! # QT Atoms
//! `qt_atoms` is a quick time media file parser based on
//! [QTFF format](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFPreface/qtffPreface.html#//apple_ref/doc/uid/TP40000939-CH202-TPXREF101)
//! specified by apple
//!


mod parse_state;
mod atoms;

pub use atoms::*;
pub use parse_state::{ParseError, Result, ParseResults};

use parse_state::Parser;

pub struct Config {
  filename: String,
}

impl Config {
  pub fn new(filename: &str) -> Config{
    Config{ filename: filename.to_string() }
  }
}
pub fn run(config: Config) -> ParseResults{
  let mut parser = Parser::new(&config.filename).unwrap();
  parser.parse()
}