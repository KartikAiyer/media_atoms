use std::fs;
use std::fmt;
use std::error;
use super::atoms;
use std::io::{Read, SeekFrom, Seek};
use std::convert::TryInto;
use byteorder::{ByteOrder, BigEndian};
use std::io::SeekFrom::Current;

#[derive(Debug)]
enum ParseError {
  IoError(std::io::Error),
  NotValidMediaFileSize(String),
}
impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ParseError::IoError(ref err) => write!(f, "{}", err),
      ParseError::NotValidMediaFileSize(ref reason) => write!(f, "{}", reason),
    }
  }
}

impl error::Error for ParseError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ParseError::IoError(ref err) => Some(err),
      ParseError::NotValidMediaFileSize(ref reason) => None,
    }
  }
}

impl From<std::io::Error> for ParseError {
  fn from(err: std::io::Error) -> ParseError {
    ParseError::IoError(err)
  }
}

type Result<T> = std::result::Result<T, ParseError>;

const MIN_FILE_READ: u64 = 8;

struct ParseState {
  filename: String,
  file: fs::File,
}

#[derive(Debug, Default)]
struct Atom {
  atom_size: u64,
  atom_type: String,
  atom_location: u64,
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

  fn parse_header(&mut self) -> Result<Atom> {
    let mut buf: [u8;8] = [0;8];
    let readout = self.file.read(buf.as_mut())?;

    let mut atom_size = [0;4];
    atom_size[..4].clone_from_slice(&buf[0..4]);
    let atom_size = u32::from_be_bytes(atom_size);
    println!("Size32 = {}", atom_size);

    let mut atom_type = [0;4];
    atom_type[..4].clone_from_slice(&buf[4..8]);
    let atom_type = String::from_utf8_lossy(&atom_type).to_string();
    println!("Type = {}", atom_type);
    let mut res = Atom{atom_size: atom_size.into(), atom_type, ..std::default::Default::default()};
    if 1 == res.atom_size {
      let readout = self.file.read(buf.as_mut()).unwrap();
      println!("Extended Buffer Read = {:?}",buf);
      res.atom_size = u64::from_be_bytes(buf);
    }
    res.atom_location = self.file.seek(Current(0))?;

    Ok(res)
  }

  fn parse(&mut self) -> Result<Vec<Atom>> {
    let mut res = vec!{};
    self.file.seek(SeekFrom::Start(0));
    let mut file_size = self.file_size();
    loop {
      if 0 == file_size {
        break;
      }
      let mut atom = self.parse_header()?;
      file_size -= atom.atom_size;
      self.file.seek(SeekFrom::Current((atom.atom_size - 8).try_into().unwrap()));//THis is broken
      res.push(atom);
    }
    Ok(res)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::borrow::Borrow;

  #[test]
  fn fails_on_a_non_existant_file() {
    let res = ParseState::new("resources/test/Nonsense.mp4");
    assert!(res.is_err());
    match res.err().unwrap() {
      ParseError::IoError(ref err) => (),
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
      ParseError::NotValidMediaFileSize(ref reason) => (),
      ref err => panic!("exprected NotValidMediaFileSize, got {:?}", err),
    }
  }

  #[test]
  fn should_parse_a_header() {
    let res = ParseState::new("resources/tests/sample.mp4");
    assert!(res.is_ok());
    assert!(res.unwrap().parse_header().is_ok());
  }

  #[test]
  fn should_parse_all_atoms_out() {
    let parser= ParseState::new("resources/tests/sample.mp4");
    assert!(parser.is_ok());
    let res = parser.unwrap().parse();
    assert!(res.is_ok());
    let res = res.unwrap();
    println!("Atoms = {:#?}", res);
  }
}
