use std::fs;
use std::io::{Read, Seek};
use std::default::Default;
use super::parse_state::Result;
use super::parse_state::ParseError;

pub trait AtomLike {
  fn atom_size(&self) -> u64;
  fn atom_type(&self) -> &str;
  fn atom_location(&self) -> u64;
  fn header_size(&self) -> u32;
}

pub struct ContainerAtom {
  header: AtomHeader,
  children: Vec<AtomNodes>,
}

pub enum AtomNodes {
  Container(ContainerAtom),
  Atom(AtomHeader),
}
#[derive(Debug)]
pub enum Atoms {
  Ftyp(FtypAtom),

  UnknownAtom(AtomHeader),
}
impl Atoms {
  pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<Box<Atoms>>
    where T: Read + Seek {
    match atom_header.atom_type() {
      "ftyp" => {
        Ok(Box::new(Atoms::Ftyp(FtypAtom::new(atom_header, file)?)))
      }
      _ => Ok(Box::new(Atoms::UnknownAtom(atom_header) ))
    }
  }
}

/// The Ftyp Atom is the [file type compatibility atom](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap1/qtff1.html#//apple_ref/doc/uid/TP40000939-CH203-CJBCBIFF).
/// Allows the reader to determine whether this a type of file that the reader understands. When a
/// file is compatible with more than one specificatio, the fiel type atom lists all the
/// compatible types and inidicates the preferred brand, or best use, among the compatible types.
#[derive(Debug, Default)]
struct FtypAtom {
  atom_header: AtomHeader,
  major_brand: u32,
  minor_version: u32,
  compatible_brands: Vec<u32>,
}

impl FtypAtom {
  pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<FtypAtom>
    where T: Read + Seek {
    file.seek(std::io::SeekFrom::Start(atom_header.atom_location()))?;
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(atom_header.atom_size as usize, 0);
    println!("Buf Length {}, atom_size as usize = {}", buf.len(), atom_header.atom_size as usize);
    let read = file.read(buf[..].as_mut())?;
    if read >= atom_header.atom_size() as usize {
      let mut atom = FtypAtom{atom_header, ..Default::default() };
      let start_offset = atom_header.header_size as usize;
      let mut bytes: [u8;4] = [0;4];
      bytes.clone_from_slice(&buf[(start_offset)..(start_offset + std::mem::size_of::<u32>())]);
      atom.major_brand = u32::from_be_bytes(bytes);
      bytes.clone_from_slice( &buf[start_offset+4..start_offset+8]);
      atom.minor_version = u32::from_be_bytes(bytes);
      let bytes_left = atom.atom_header.atom_size() -
        atom.atom_header.header_size() as u64 -
        2*std::mem::size_of::<u32>() as u64;
      let num_u32 = bytes_left / std::mem::size_of::<u32>() as u64;
      let start_offset = atom.header_size() + 2*std::mem::size_of::<u32>() as u32;
      for i in 0..num_u32 as u32 {
        let start_offset = (start_offset + i*4 )as usize;
        bytes.clone_from_slice(&buf[start_offset..(start_offset+std::mem::size_of::<u32>() as usize)]);
        atom.compatible_brands.push(u32::from_be_bytes(bytes));
      }
      Ok(atom)
    } else {
      Err(ParseError::AtomParseFailed(String::from(atom_header.atom_type())))
    }
  }
}
impl AtomLike for FtypAtom {
  fn atom_size(&self) -> u64 { self.atom_header.atom_size() }
  fn atom_type(&self) -> &str { self.atom_header.atom_type() }
  fn atom_location(&self) -> u64 { self.atom_header.atom_location() }
  fn header_size(&self) -> u32 { self.atom_header.header_size() }
}

#[derive(Default, Copy, Clone)]
pub struct AtomHeader {
  atom_size: u64,
  atom_type: [u8;4],
  atom_location: u64,
  header_size: u32,
}

impl AtomHeader {
  pub fn new(atom_size: u64, atom_type: [u8; 4], atom_location: u64, header_size: u32) -> AtomHeader
  {
    AtomHeader{atom_size, atom_type, atom_location, header_size}
  }
}
impl AtomLike for AtomHeader {
  fn atom_size(&self) -> u64 { self.atom_size }
  fn atom_type(&self) -> &str { std::str::from_utf8(&self.atom_type).unwrap() }
  fn atom_location(&self) -> u64 { self.atom_location }
  fn header_size(&self) -> u32 { self.header_size }
}

impl std::fmt::Debug for AtomHeader{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Header{{ atom_size: {}, atom_type: {}, atom_location: {}, header_size: {}",
    self.atom_size(), self.atom_type(), self.atom_location(), self.header_size())
  }
}
