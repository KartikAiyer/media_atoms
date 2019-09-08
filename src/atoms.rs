use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::default::Default;
use super::parse_state::{Result, ParseError};
use containers::*;
use leaves::*;

pub trait AtomLike {
  fn atom_size(&self) -> u64;
  fn atom_type(&self) -> &str;
  fn atom_location(&self) -> u64;
  fn header_size(&self) -> u32;
}

pub trait Container {
  fn children(&self) -> &Vec<AtomNodes>;
}

#[derive(Default, Copy, Clone)]
pub struct AtomHeader {
  atom_size: u64,
  atom_type: [u8;4],
  atom_location: u64,
  header_size: u32,
}

impl AtomHeader {
  pub fn new<T>(file: &mut T) -> Result<AtomHeader>
  where T: Read + Seek {
    let mut buf: [u8; 8] = [0; 8];
    let mut readout = file.read(buf.as_mut())?;

    let mut atom_size = [0; 4];
    atom_size[..4].clone_from_slice(&buf[0..4]);
    let mut atom_size: u64 = u32::from_be_bytes(atom_size) as u64;

    let mut atom_type = [0; 4];
    atom_type[..4].clone_from_slice(&buf[4..8]);

    if 1 == atom_size {
      readout += file.read(buf.as_mut()).unwrap();
      atom_size = u64::from_be_bytes(buf);
    }
    let atom_location = file.seek(SeekFrom::Current(0))? - readout as u64;
    let header_size = readout as u32;
    Ok(AtomHeader{atom_size, atom_type, atom_location, header_size})
  }

  pub fn read_atom<T>(&self, file: &mut T) -> Result<Vec<u8>> where T: Read + Seek {
    let mut buf = Vec::new();
    buf.resize(self.atom_size() as usize, 0);
    file.seek(SeekFrom::Start(self.atom_location()))?;
    let read = file.read(buf.as_mut_slice())?;
    if read == self.atom_size() as usize {
      Ok(buf)
    } else {
      Err(ParseError::FailedToReadOutAtom(self.atom_type().to_string(), self.atom_size(), read))
    }
  }
}

impl AtomLike for AtomHeader {
  fn atom_size(&self) -> u64 { self.atom_size }
  fn atom_type(&self) -> &str { std::str::from_utf8(&self.atom_type).unwrap() }
  fn atom_location(&self) -> u64 { self.atom_location }
  fn header_size(&self) -> u32 { self.header_size }
}

impl std::fmt::Debug for AtomHeader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Header{{ atom_size: {}, atom_type: {}, atom_location: {}, header_size: {}",
           self.atom_size(), self.atom_type(), self.atom_location(), self.header_size())
  }
}
impl std::fmt::Display for AtomHeader {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "atom_size: {}, atom_type: {}, atom_location: {}, header_size: {}",
           self.atom_size(), self.atom_type(), self.atom_location(), self.header_size())?;
    Ok(())
  }
}

#[test]
fn should_parse_a_header() {
  let mut file = std::fs::File::open("resources/tests/free.mp4").unwrap();
  assert!(AtomHeader::new(&mut file).is_ok());
}

#[derive(Debug, Clone)]
pub enum AtomNodes {
  Container(ContainerAtoms),
  Atom(Atoms)
}

impl AtomNodes {
  pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<AtomNodes>
  where T: Read + Seek {
    if let Ok(k) = ContainerAtoms::new(atom_header, file) {
      Ok(AtomNodes::Container(k))
    } else {
      Ok(AtomNodes::Atom(Atoms::new(atom_header, file)?))
    }
  }
  pub fn is_container(&self) -> bool {
    if let AtomNodes::Container(_) = self {
      true
    } else {
      false
    }
  }
}

impl AtomLike for AtomNodes {
  fn atom_size(&self) -> u64 {
    match self {
      AtomNodes::Container(atom) => atom.atom_size(),
      AtomNodes::Atom(atom) => atom.atom_size(),
    }
  }

  fn atom_type(&self) -> &str {
    match self {
      AtomNodes::Container(atom) => atom.atom_type(),
      AtomNodes::Atom(atom) => atom.atom_type(),
    }
  }

  fn atom_location(&self) -> u64 {
    match self{
      AtomNodes::Container(atom) => atom.atom_location(),
      AtomNodes::Atom(atom) => atom.atom_location(),
    }
  }

  fn header_size(&self) -> u32 {
    match self {
      AtomNodes::Container(atom) => atom.header_size(),
      AtomNodes::Atom(atom) => atom.header_size(),
    }
  }
}

impl std::fmt::Display for AtomNodes {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      AtomNodes::Container(atom) => write!(f, "{}", atom)?,
      AtomNodes::Atom(atom) => write!(f, "{}", atom)?,
    }
    Ok(())
  }
}

mod containers {
  use super::*;

  impl AtomHeader {
    fn parse_children<T>(&self, file: &mut T) -> Result<Vec<AtomHeader>> where T:Read + Seek {
      let mut children = Vec::new();
      loop {
        let child_header = AtomHeader::new(file)?;
        children.push(child_header);
        file.seek(SeekFrom::Start(child_header.atom_location() + child_header.atom_size()))?;
        if child_header.atom_location() + child_header.atom_size() >= self.atom_size() {
          break;
        }
      }
      Ok(children)
    }
  }
  impl AtomNodes {
    fn parse_children<T>(container_header: AtomHeader, file: &mut T) -> Result<Vec<AtomNodes>>
      where T: Read + Seek {
      Ok(container_header.parse_children(file)?.iter().map(|x| {
        AtomNodes::new(*x, file)
      }).filter(|x| { x.is_ok() })
        .map(|x| { x.unwrap() })
        .collect())
    }
  }
  #[derive(Debug, Clone)]
  pub enum ContainerAtoms {
    Moov(MoovAtom),
  }

  impl ContainerAtoms {
    pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<ContainerAtoms>
      where T: Read + Seek {
      match atom_header.atom_type() {
        "moov" => Ok(ContainerAtoms::Moov(MoovAtom::new(atom_header, file)?)),
        _ => Err(ParseError::NotAContainer)
      }
    }
  }

  impl AtomLike for ContainerAtoms {
    fn atom_size(&self) -> u64 {
      match self {
        ContainerAtoms::Moov(atom) => atom.atom_size()
      }
    }

    fn atom_type(&self) -> &str {
      match self {
        ContainerAtoms::Moov(atom) => atom.atom_type()
      }
    }

    fn atom_location(&self) -> u64 {
      match self {
        ContainerAtoms::Moov(atom) => atom.atom_location()
      }
    }

    fn header_size(&self) -> u32 {
      match self {
        ContainerAtoms::Moov(atom) => atom.header_size()
      }
    }
  }

  impl std::fmt::Display for ContainerAtoms {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        ContainerAtoms::Moov(atom) => {
          writeln!(f, "{}", atom)?;
          let mut num_children = atom.children().len();
          for node in atom.children() {
            if num_children == 1 {
              write!(f, "┗ ")?;
            } else {
              write!(f, "├ ")?;
            }
            writeln!(f, "{}", node)?;
            num_children -= 1;
          }
          Ok(())
        }
      }
    }
  }

  #[test]
  fn test_container() {
    let mut file = fs::File::open("resources/tests/moov.mp4").unwrap();
    let header = AtomHeader::new(&mut file).unwrap();
    let container = ContainerAtoms::new(header, &mut file).unwrap();
    match &container {
      ContainerAtoms::Moov(atom) => {
        let children = atom.children();
        assert!(children.len() > 1);
      }
      _ => panic!("Unexpected Atom"),
    }
    println!("{}", container);
    panic!("Stuff");
  }

  #[derive(Debug, Default, Clone)]
  struct MoovAtom {
    atom_header: AtomHeader,
    children: Vec<AtomNodes>,
  }

  impl MoovAtom {
    pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<MoovAtom>
      where T: Read + Seek {
      let mut location = atom_header.atom_location() + atom_header.header_size() as u64;
      file.seek(SeekFrom::Start(location))?;
      let children = AtomNodes::parse_children(atom_header, file)?;
      Ok(MoovAtom {atom_header, children})
    }
  }

  impl AtomLike for MoovAtom {
    fn atom_size(&self) -> u64 {
      self.atom_header.atom_size()
    }

    fn atom_type(&self) -> &str {
      self.atom_header.atom_type()
    }

    fn atom_location(&self) -> u64 {
      self.atom_header.atom_location()
    }

    fn header_size(&self) -> u32 {
      self.atom_header.header_size()
    }
  }

  impl Container for MoovAtom {
    fn children(&self) -> &Vec<AtomNodes> {
      self.children.as_ref()
    }
  }

  impl std::fmt::Display for MoovAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Moov: {}", self.atom_header)?;
      Ok(())
    }
  }

  #[test]
  fn can_parse_a_moov_atom() {
    let mut file = fs::File::open("resources/tests/moov.mp4").unwrap();
    let header = AtomHeader::new(&mut file).unwrap();
    let atom = MoovAtom::new(header, &mut file).unwrap();
    for child in &atom.children {
      println!("{}", child);
    }
    assert!(atom.children.len() > 1);
  }
}

mod leaves {
  use super::*;
  #[derive(Debug, Clone)]
  pub enum Atoms {
    Ftyp(FtypAtom),
    Free(FreeAtom),
    Wide(WideAtom),
    Mdat(MdiaAtom),
    UnknownAtom(AtomHeader),
  }

  impl Atoms {
    pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<Atoms>
      where T: Read + Seek {
      match atom_header.atom_type() {
        "ftyp" => Ok(Atoms::Ftyp(FtypAtom::new(atom_header, file)?)),
        "free" => Ok(Atoms::Free(FreeAtom::new(atom_header)?)),
        "wide" => Ok(Atoms::Wide(WideAtom::new(atom_header)?)),
        "mdat" => Ok(Atoms::Mdat(MdiaAtom::new(atom_header)?)),
        _ => Ok(Atoms::UnknownAtom(atom_header))
      }
    }
  }

  impl AtomLike for Atoms {
    fn atom_size(&self) -> u64 {
      match self {
        Atoms::Ftyp(atom) => atom.atom_size(),
        Atoms::Free(atom) => atom.atom_size(),
        Atoms::Wide(atom) => atom.atom_size(),
        Atoms::Mdat(atom) => atom.atom_size(),
        Atoms::UnknownAtom(atom) => atom.atom_size(),
      }
    }

    fn atom_type(&self) -> &str {
      match self {
        Atoms::Ftyp(atom) => atom.atom_type(),
        Atoms::Free(atom) => atom.atom_type(),
        Atoms::Wide(atom) => atom.atom_type(),
        Atoms::Mdat(atom) => atom.atom_type(),
        Atoms::UnknownAtom(atom) => atom.atom_type(),
      }
    }

    fn atom_location(&self) -> u64 {
      match self {
        Atoms::Ftyp(atom) => atom.atom_location(),
        Atoms::Free(atom) => atom.atom_location(),
        Atoms::Wide(atom) => atom.atom_location(),
        Atoms::Mdat(atom) => atom.atom_location(),
        Atoms::UnknownAtom(atom) => atom.atom_location(),
      }
    }

    fn header_size(&self) -> u32 {
      match self {
        Atoms::Ftyp(atom) => atom.header_size(),
        Atoms::Free(atom) => atom.header_size(),
        Atoms::Wide(atom) => atom.header_size(),
        Atoms::Mdat(atom) => atom.header_size(),
        Atoms::UnknownAtom(atom) => atom.header_size(),
      }
    }
  }

  impl std::fmt::Display for Atoms {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        Atoms::Ftyp(atom) => {
          write!(f, "{}", atom);
          Ok(())
        },
        Atoms::Free(atom) => {
          write!(f, "{}", atom);
          Ok(())
        },
        Atoms::Wide(atom) => {
          write!(f, "{}", atom);
          Ok(())
        },
        Atoms::Mdat(atom) => {
          write!(f, "{}", atom);
          Ok(())
        },
        Atoms::UnknownAtom(atom) => {
          write!(f, "Unknown: {}", atom);
          Ok(())
        },
      }
    }
  }

  /// The Ftyp Atom is the [file type compatibility atom](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap1/qtff1.html#//apple_ref/doc/uid/TP40000939-CH203-CJBCBIFF).
/// Allows the reader to determine whether this a type of file that the reader understands. When a
/// file is compatible with more than one specificatio, the fiel type atom lists all the
/// compatible types and inidicates the preferred brand, or best use, among the compatible types.
  #[derive(Debug, Default, Clone)]
  struct FtypAtom {
    atom_header: AtomHeader,
    major_brand: u32,
    minor_version: u32,
    compatible_brands: Vec<u32>,
  }

  impl FtypAtom {
    pub fn new<T>(atom_header: AtomHeader, file: &mut T) -> Result<FtypAtom>
      where T: Read + Seek {
      let mut buf = atom_header.read_atom(file)?;

      if buf.len() >= atom_header.atom_size() as usize {
        let mut atom = FtypAtom { atom_header, ..Default::default() };
        let start_offset = atom_header.header_size as usize;
        let mut bytes: [u8; 4] = [0; 4];
        bytes.clone_from_slice(&buf[(start_offset)..(start_offset + std::mem::size_of::<u32>())]);
        atom.major_brand = u32::from_be_bytes(bytes);
        bytes.clone_from_slice(&buf[start_offset + 4..start_offset + 8]);
        atom.minor_version = u32::from_be_bytes(bytes);
        let bytes_left = atom.atom_header.atom_size() -
          atom.atom_header.header_size() as u64 -
          2 * std::mem::size_of::<u32>() as u64;
        let num_u32 = bytes_left / std::mem::size_of::<u32>() as u64;
        let start_offset = atom.header_size() + 2 * std::mem::size_of::<u32>() as u32;
        for i in 0..num_u32 as u32 {
          let start_offset = (start_offset + i * 4) as usize;
          bytes.clone_from_slice(&buf[start_offset..(start_offset + std::mem::size_of::<u32>() as usize)]);
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

  impl std::fmt::Display for FtypAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Ftyp - Major: {}, Minor: {}, Compatible = [",
             String::from_utf8_lossy(&self.major_brand.to_be_bytes()).to_string(),
             String::from_utf8_lossy(&self.minor_version.to_be_bytes()).to_string())?;
      for item in &self.compatible_brands {
        write!(f, "{},", String::from_utf8_lossy(&item.to_be_bytes()).to_string())?;
      }
      write!(f, "]")?;
      Ok(())
    }
  }

  #[test]
  fn test_read_of_ftyp() {
    let mut file = std::fs::File::open("resources/tests/ftyp.mp4").unwrap();
    let header = AtomHeader::new(&mut file).unwrap();
    let atom = header.read_atom(&mut file).unwrap();
    assert_eq!(header.atom_size() as usize, atom.len());
  }

  #[derive(Debug, Default, Clone, Copy)]
  struct WideAtom {
    atom_header: AtomHeader,
  }

  impl WideAtom {
    /// Creates a wide atom given a wide atom header. There is nothing to read from the file since
  /// all the data is in the header
    pub fn new(atom_header: AtomHeader) -> Result<WideAtom>
    {
      Ok(WideAtom { atom_header })
    }
  }

  impl AtomLike for WideAtom {
    fn atom_size(&self) -> u64 { self.atom_header.atom_size() }
    fn atom_type(&self) -> &str { self.atom_header.atom_type() }
    fn atom_location(&self) -> u64 { self.atom_header.atom_location() }
    fn header_size(&self) -> u32 { self.atom_header.header_size() }
  }

  impl std::fmt::Display for WideAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Wide: {}", self.atom_header);
      Ok(())
    }
  }

  #[derive(Debug, Default, Clone, Copy)]
  struct FreeAtom {
    atom_header: AtomHeader,
  }

  impl FreeAtom {
    pub fn new(atom_header: AtomHeader) -> Result<FreeAtom> {
      Ok(FreeAtom { atom_header })
    }
  }

  impl AtomLike for FreeAtom {
    fn atom_size(&self) -> u64 { self.atom_header.atom_size() }
    fn atom_type(&self) -> &str { self.atom_header.atom_type() }
    fn atom_location(&self) -> u64 { self.atom_header.atom_location() }
    fn header_size(&self) -> u32 { self.atom_header.header_size() }
  }

  impl std::fmt::Display for FreeAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Free: {}", self.atom_header)?;
      Ok(())
    }
  }

  #[derive(Debug, Default, Clone, Copy)]
  struct MdiaAtom {
    atom_header: AtomHeader,
  }

  impl MdiaAtom {
    pub fn new(atom_header: AtomHeader) -> Result<MdiaAtom> {
      Ok(MdiaAtom { atom_header })
    }
  }

  impl AtomLike for MdiaAtom {
    fn atom_size(&self) -> u64 { self.atom_header.atom_size() }
    fn atom_type(&self) -> &str { self.atom_header.atom_type() }
    fn atom_location(&self) -> u64 { self.atom_header.atom_location() }
    fn header_size(&self) -> u32 { self.atom_header.header_size() }
  }

  impl std::fmt::Display for MdiaAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Mdat: {}", self.atom_header)?;
      Ok(())
    }
  }
}
