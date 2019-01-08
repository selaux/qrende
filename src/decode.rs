use super::modules::QRCodeModules;
use super::position::QRCodeVersion;
use bitvec::{bitvec, BigEndian, BitVec};

struct FormatInformationIteratorUpperRightLowerLeft {
  current_index: usize,
  number_of_modules: usize,
}

impl FormatInformationIteratorUpperRightLowerLeft {
  fn new(version: &QRCodeVersion) -> Self {
    FormatInformationIteratorUpperRightLowerLeft {
      current_index: 0,
      number_of_modules: version.number_of_modules() as usize,
    }
  }
}

impl Iterator for FormatInformationIteratorUpperRightLowerLeft {
  type Item = (usize, usize);

  fn next(&mut self) -> Option<(usize, usize)> {
    if self.current_index <= 7 {
      self.current_index += 1;
      return Some((self.number_of_modules - 1 - self.current_index, 8));
    }
    if self.current_index <= 14 {
      self.current_index += 1;
      return Some((8, self.number_of_modules - 8 + self.current_index - 8));
    }
    None
  }
}

#[derive(Debug)]
pub enum ErrorCorrectionLevel {
  L,
  M,
  Q,
  H,
}

impl ErrorCorrectionLevel {
  fn from_bits(bits: [bool; 2]) -> Self {
    match bits {
      [false, true] => ErrorCorrectionLevel::L,
      [false, false] => ErrorCorrectionLevel::M,
      [true, true] => ErrorCorrectionLevel::Q,
      [true, false] => ErrorCorrectionLevel::H,
    }
  }
}

#[derive(Debug)]
pub enum Mask {
  M000,
  M001,
  M010,
  M011,
  M100,
  M101,
  M110,
  M111,
}

impl Mask {
  fn from_bits(bits: [bool; 3]) -> Self {
    match bits {
      [false, false, false] => Mask::M000,
      [false, false, true] => Mask::M001,
      [false, true, false] => Mask::M010,
      [false, true, true] => Mask::M011,
      [true, false, false] => Mask::M100,
      [true, false, true] => Mask::M101,
      [true, true, false] => Mask::M110,
      [true, true, true] => Mask::M111,
    }
  }
}

#[derive(Debug)]
pub struct FormatInformation {
  error_correction_level: ErrorCorrectionLevel,
  mask: Mask,
}

pub fn decode_format_information(modules: &QRCodeModules) -> FormatInformation {
  let format_mask = bitvec![1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0];
  let format_iterator1 = FormatInformationIteratorUpperRightLowerLeft::new(&modules.version);
  let mut format_vec: BitVec = format_iterator1.map(|(x, y)| modules.bits[x][y]).collect();
  format_vec ^= format_mask.iter();

  let error_correction_level_bits: [bool; 2] = [format_vec[0], format_vec[1]];
  let mask_bits: [bool; 3] = [format_vec[2], format_vec[3], format_vec[4]];

  FormatInformation {
    error_correction_level: ErrorCorrectionLevel::from_bits(error_correction_level_bits),
    mask: Mask::from_bits(mask_bits),
  }
}
