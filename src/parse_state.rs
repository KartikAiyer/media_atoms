use std::fs;
use std::fmt;
use std::error;
use super::atoms::{AtomLike, AtomHeader, AtomNodes, containers::ContainerAtoms};
use std::io::{Read, SeekFrom, Seek};
use std::borrow::{BorrowMut, Borrow};
use crate::Container;

#[derive(Debug)]
pub enum ParseError {
  IoError(std::io::Error),
  NotValidMediaFileSize(String),
  AtomParseFailed(String),
  NotAContainer,
  FailedToReadOutAtom(String, u64, usize),
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ParseError::IoError(ref err) => write!(f, "{}", err),
      ParseError::NotValidMediaFileSize(ref reason) => write!(f, "{}", reason),
      ParseError::AtomParseFailed(ref atom_type) => write!(f, "{}", atom_type),
      ParseError::NotAContainer => Ok(()),
      ParseError::FailedToReadOutAtom(atom_type, atom_size, read_size) =>
        write!(f, "type: {}, size: {}, read out: {}", atom_type, atom_size, read_size),
    }
  }
}

impl error::Error for ParseError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ParseError::IoError(ref err) => Some(err),
      _ => None,
    }
  }
}

impl From<std::io::Error> for ParseError {
  fn from(err: std::io::Error) -> ParseError {
    ParseError::IoError(err)
  }
}

pub type Result<T> = std::result::Result<T, ParseError>;

const MIN_FILE_READ: u64 = 8;

#[derive(Default, Debug)]
pub struct ParseResults<'a> {
  nodes: AtomNodes,
  current_depth: Vec<(&'a ContainerAtoms,std::option::IterMut<'a, AtomNodes>)>,
}
impl ParseResults {
  pub fn new(root: AtomNodes) -> ParseResults {
    let mut res = ParseResults{..Default::default()};
    match &root {
      AtomNodes::Container(atom) => {
        let iter = atom.children().iter_mut();
        res.current_depth.push((atom, iter))
      },
      _ => panic!("Unexpected"),
    }

    res
  }
}

impl Iterator for ParseResults<'a> {
  type Item = AtomNodes;

  fn next(&mut self) -> Option<Self::Item> {

    match &self.nodes {
      AtomNodes::Container(atom) => {
        None
      },
      AtomNodes::Atom(atom) => None,
    }
  }
}
pub struct Parser {
  filename: String,
  file: fs::File,
}

impl Parser {
  pub fn new(filename: &str) -> Result<Parser> {
    let file = fs::File::open(filename)?;
    let meta = file.metadata()?;
    if meta.len() > MIN_FILE_READ {
      Ok(Parser { filename: String::from(filename), file })
    } else {
      Err(ParseError::NotValidMediaFileSize(String::from("Bad File Size")))
    }
  }
  fn file_size(&self) -> u64 {
    let meta = self.file.metadata();
    if meta.is_ok() {
      meta.unwrap().len()
    } else {
      0
    }
  }

  pub fn parse(&mut self) -> Result<AtomNodes> {
    self.file.seek(SeekFrom::Start(0));
    let header: AtomHeader = self.into();
    AtomNodes::new(header, &mut self.file)
  }
}

impl AtomLike for Parser {
  fn atom_size(&self) -> u64 {
    self.file_size()
  }

  fn atom_type(&self) -> &str {
    "root"
  }

  fn atom_location(&self) -> u64 {
    0
  }

  fn header_size(&self) -> u32 {
    0
  }
}
impl From<&mut Parser> for AtomHeader {
  fn from(item: &mut Parser) -> Self {
    let atom_like: &dyn AtomLike = item;
    atom_like.into()
  }
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fails_on_a_non_existant_file() {
    let res = Parser::new("resources/test/Nonsense.mp4");
    assert!(res.is_err());
    match res.err().unwrap() {
      ParseError::IoError(_) => (),
      ref err => panic!("expected IoError, got {:?}", err),
    }
  }

  #[test]
  fn should_be_able_to_open_a_file() {
    assert!(Parser::new("resources/tests/sample.mp4").is_ok());
  }

  #[test]
  fn should_reject_a_file_if_not_of_valid_size() {
    let res = Parser::new("resources/tests/empty_file.mp4");
    assert!(res.is_err());
    match res.err().unwrap() {
      ParseError::NotValidMediaFileSize(_) => (),
      ref err => panic!("expected NotValidMediaFileSize, got {:?}", err),
    }
  }

  #[test]
  fn should_parse_atoms_as_nodes() {
    let parser = Parser::new("resources/tests/sample.mp4");
    assert!(parser.is_ok());
    let res = parser.unwrap().parse().unwrap();
    assert_eq!(res.atom_type(), "root");
  }
}
