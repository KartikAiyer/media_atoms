//! # QT Atoms
//! `qt_atoms` is a quick time media file parser based on
//! [QTFF format](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFPreface/qtffPreface.html#//apple_ref/doc/uid/TP40000939-CH202-TPXREF101)
//! specified by apple
//!


use std::error;

mod parse_state;
mod atoms;

use crate::parse_state::*;

pub struct Config {
  filename: String,
}
