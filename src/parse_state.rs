use std::fs;
use std::fmt;
use std::error;
use super::atoms::{AtomLike, AtomHeader, AtomNodes};
use std::io::{Read, SeekFrom, Seek};
use std::io::SeekFrom::Current;
use std::borrow::BorrowMut;

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

struct ParseState {
  filename: String,
  file: fs::File,
}

impl ParseState {
  pub fn new(filename: &str) -> Result<ParseState> {
    let file = fs::File::open(filename)?;
    let meta = file.metadata()?;
    if meta.len() > MIN_FILE_READ {
      Ok(ParseState { filename: String::from(filename), file })
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


  pub fn parse(&mut self) -> Result<Vec<AtomNodes>> {
    let mut res = vec! {};
    self.file.seek(SeekFrom::Start(0));
    let mut file_size = self.file_size();
    loop {
      if 0 == file_size {
        break;
      }
      let header = AtomHeader::new(self.file.borrow_mut())?;
      let atom = AtomNodes::new(header, self.file.borrow_mut())?;
      file_size -= atom.atom_size();
      assert_eq!(atom.atom_size(), header.atom_size());
      self.file.seek(SeekFrom::Start((atom.atom_location() + atom.atom_size() ) as u64))?;
      res.push(atom);
    }
    Ok(res)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fails_on_a_non_existant_file() {
    let res = ParseState::new("resources/test/Nonsense.mp4");
    assert!(res.is_err());
    match res.err().unwrap() {
      ParseError::IoError(_) => (),
      ref err => panic!("expected IoError, got {:?}", err),
    }
  }

  #[test]
  fn should_be_able_to_open_a_file() {
    assert!(ParseState::new("resources/tests/sample.mp4").is_ok());
  }

  #[test]
  fn should_reject_a_file_if_not_of_valid_size() {
    let res = ParseState::new("resources/tests/empty_file.mp4");
    assert!(res.is_err());
    match res.err().unwrap() {
      ParseError::NotValidMediaFileSize(_) => (),
      ref err => panic!("expected NotValidMediaFileSize, got {:?}", err),
    }
  }

  #[test]
  fn should_parse_all_atoms_out() {
    let parser = ParseState::new("resources/tests/sample.mp4");
    assert!(parser.is_ok());
    let res = parser.unwrap().parse();
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res.len() >= 1);
    for i in res {
      println!("{}", i);
    }
  }
}
