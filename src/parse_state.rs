use std::fs;
use std::fmt;
use std::error;

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

const MIN_FILE_READ: u64 = 8;

struct ParseState {
  filename: String,
  file: fs::File,
}

impl ParseState {
  pub fn new(filename: &str) -> Result<ParseState, ParseError> {
    let file = fs::File::open(filename)?;
    let meta = file.metadata()?;
    if meta.len() > MIN_FILE_READ {
      Ok(ParseState { filename: String::from(filename), file })
    } else {
      Err(ParseError::NotValidMediaFileSize(String::from("Bad File Size")))
    }
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
}
