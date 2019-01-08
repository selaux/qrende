use image::imageops::colorops;
use image::{DynamicImage, Luma};
use imageproc::definitions::Image;
use std::collections::HashMap;

const EXPECTED_RATIOS: [f64; 5] = [1., 1., 3., 1., 1.];
const SYMMETRY_THRESHOLD: f64 = 0.4;
const VARIANCE_THRESHOLD: f64 = 0.5;

#[derive(Debug)]
struct ScanResult {
  start: u32,
  end: u32,
  black_border1_count: u32,
  white_inner1_count: u32,
  black_inner_count: u32,
  white_inner2_count: u32,
  black_border2_count: u32,
}

impl ScanResult {
  fn middle(&self) -> f64 {
    [
      f64::from(self.start),
      f64::from(self.black_border1_count),
      f64::from(self.white_inner1_count),
      f64::from(self.black_inner_count) / 2.,
    ]
    .iter()
    .sum()
  }

  fn size(&self) -> f64 {
    let total_ratios: f64 = EXPECTED_RATIOS.iter().sum();
    let inner_ratio = EXPECTED_RATIOS[2];
    (total_ratios * f64::from(self.black_inner_count)) / inner_ratio
  }
}

#[derive(Debug)]
enum ScanState {
  InWhite,
  InBlack,
  BlackBorder1 {
    start: u32,
    black_border1_count: u32,
  },
  WhiteInner1 {
    start: u32,
    black_border1_count: u32,
    white_inner1_count: u32,
  },
  BlackInner {
    start: u32,
    black_border1_count: u32,
    white_inner1_count: u32,
    black_inner_count: u32,
  },
  WhiteInner2 {
    start: u32,
    black_border1_count: u32,
    white_inner1_count: u32,
    black_inner_count: u32,
    white_inner2_count: u32,
  },
  BlackBorder2 {
    start: u32,
    black_border1_count: u32,
    white_inner1_count: u32,
    black_inner_count: u32,
    white_inner2_count: u32,
    black_border2_count: u32,
  },
  Found(ScanResult),
}

pub fn is_white(pixel: Luma<u8>) -> bool {
  pixel[0] == 255
}

fn is_symmetric(scan_result: &ScanResult) -> bool {
  let one = [
    scan_result.black_border1_count,
    scan_result.white_inner1_count,
  ];
  let two = [
    scan_result.black_border2_count,
    scan_result.white_inner2_count,
  ];
  let total = f64::from(one.iter().chain(two.iter()).sum::<u32>());
  let sum: f64 = one
    .iter()
    .zip(two.iter())
    .map(|(got, expected)| f64::abs(f64::from(*got) - f64::from(*expected)))
    .map(|s| s / total)
    .sum::<f64>();

  sum < SYMMETRY_THRESHOLD
}

fn ratios_match(scan_result: &ScanResult) -> bool {
  let scan_result_widths = [
    scan_result.black_border1_count,
    scan_result.white_inner1_count,
    scan_result.black_inner_count,
    scan_result.white_inner2_count,
    scan_result.black_border2_count,
  ];
  let ratios_total: f64 = EXPECTED_RATIOS.iter().sum();
  let scan_result_total = f64::from(scan_result_widths.iter().sum::<u32>());

  let module_size = scan_result_total / ratios_total;
  let max_variance = VARIANCE_THRESHOLD * module_size;

  scan_result_widths
    .iter()
    .zip(EXPECTED_RATIOS.iter())
    .all(|(width, ratio)| f64::abs(ratio * module_size - f64::from(*width)) < ratio * max_variance)
}

fn is_valid_match(scan_result: &ScanResult) -> bool {
  is_symmetric(scan_result) && ratios_match(scan_result)
}

fn advance_state(state: &ScanState, pos: u32, next_pos: u32, pixel: Luma<u8>) -> (u32, ScanState) {
  let is_white = is_white(pixel);
  let new_state = match state {
    ScanState::Found { .. } => {
      if is_white {
        ScanState::InWhite
      } else {
        ScanState::InBlack
      }
    }
    ScanState::InWhite => {
      if is_white {
        ScanState::InWhite
      } else {
        ScanState::BlackBorder1 {
          start: pos,
          black_border1_count: 1,
        }
      }
    }
    ScanState::InBlack => {
      if is_white {
        ScanState::InWhite
      } else {
        ScanState::InBlack
      }
    }
    ScanState::BlackBorder1 {
      start,
      black_border1_count,
    } => {
      if is_white {
        ScanState::WhiteInner1 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: 1,
        }
      } else {
        ScanState::BlackBorder1 {
          start: *start,
          black_border1_count: black_border1_count + 1,
        }
      }
    }
    ScanState::WhiteInner1 {
      start,
      black_border1_count,
      white_inner1_count,
    } => {
      if is_white {
        ScanState::WhiteInner1 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: white_inner1_count + 1,
        }
      } else {
        ScanState::BlackInner {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: 1,
        }
      }
    }
    ScanState::BlackInner {
      start,
      black_border1_count,
      white_inner1_count,
      black_inner_count,
    } => {
      if is_white {
        ScanState::WhiteInner2 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: *black_inner_count,
          white_inner2_count: 1,
        }
      } else {
        ScanState::BlackInner {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: black_inner_count + 1,
        }
      }
    }
    ScanState::WhiteInner2 {
      start,
      black_border1_count,
      white_inner1_count,
      black_inner_count,
      white_inner2_count,
    } => {
      if is_white {
        ScanState::WhiteInner2 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: *black_inner_count,
          white_inner2_count: white_inner2_count + 1,
        }
      } else {
        ScanState::BlackBorder2 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: *black_inner_count,
          white_inner2_count: *white_inner2_count,
          black_border2_count: 1,
        }
      }
    }
    ScanState::BlackBorder2 {
      start,
      black_border1_count,
      white_inner1_count,
      black_inner_count,
      white_inner2_count,
      black_border2_count,
    } => {
      if is_white {
        ScanState::Found(ScanResult {
          start: *start,
          end: pos,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: *black_inner_count,
          white_inner2_count: *white_inner2_count,
          black_border2_count: *black_border2_count,
        })
      } else {
        ScanState::BlackBorder2 {
          start: *start,
          black_border1_count: *black_border1_count,
          white_inner1_count: *white_inner1_count,
          black_inner_count: *black_inner_count,
          white_inner2_count: *white_inner2_count,
          black_border2_count: black_border2_count + 1,
        }
      }
    }
  };

  if let ScanState::Found(scan_result) = &new_state {
    if is_valid_match(scan_result) {
      (next_pos, new_state)
    } else {
      (scan_result.start, ScanState::InBlack)
    }
  } else {
    (next_pos, new_state)
  }
}

#[derive(Debug, Clone)]
pub struct PositionMarkerHint {
  pub center: (f64, f64),
  pub size: f64,
}

pub fn detect_position_marker_hints(image: &Image<Luma<u8>>) -> Vec<PositionMarkerHint> {
  let mut found: Vec<PositionMarkerHint> = vec![];

  for x in 0..image.width() {
    let mut state = ScanState::InWhite;
    let mut y: u32 = 0;

    while y < image.height() {
      let (new_y, new_state) = advance_state(&state, y, y + 1, *image.get_pixel(x, y));
      y = new_y;
      state = new_state;
      if let ScanState::Found(scan_result) = &state {
        found.push(PositionMarkerHint {
          center: (f64::from(x), scan_result.middle()),
          size: scan_result.size(),
        });
      }
    }
  }

  for y in 0..image.height() {
    let mut state = ScanState::InWhite;
    let mut x: u32 = 0;

    while x < image.width() {
      let (new_x, new_state) = advance_state(&state, x, x + 1, *image.get_pixel(x, y));
      x = new_x;
      state = new_state;
      if let ScanState::Found(scan_result) = &state {
        found.push(PositionMarkerHint {
          center: (scan_result.middle(), f64::from(y)),
          size: scan_result.size(),
        });
      }
    }
  }
  found
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionMarker {
  pub center: (f64, f64),
  pub size: f64,
}

pub fn cluster_position_marker_hints(hints: &[PositionMarkerHint]) -> Vec<PositionMarker> {
  let centers: Vec<_> = hints.iter().map(|h| vec![h.center.0, h.center.1]).collect();
  let classifications = dbscan::cluster(4., 9, &centers);
  let mut clusters: HashMap<usize, Vec<&PositionMarkerHint>> = HashMap::new();

  for (index, c) in classifications.iter().enumerate() {
    let hint = &hints[index];
    match c {
      dbscan::Classification::Noise => {}
      dbscan::Classification::Core(cluster) | dbscan::Classification::Edge(cluster) => {
        clusters
          .entry(*cluster)
          .and_modify(|v| v.push(hint))
          .or_insert_with(|| vec![hint]);
      }
    }
  }

  let markers: Vec<_> = clusters
    .values()
    .map(|hints| {
      let number_of_hints = hints.len();
      let mean_size = hints.iter().map(|hint| hint.size).sum::<f64>() / number_of_hints as f64;
      let mean_center_x =
        hints.iter().map(|hint| hint.center.0).sum::<f64>() / number_of_hints as f64;
      let mean_center_y =
        hints.iter().map(|hint| hint.center.1).sum::<f64>() / number_of_hints as f64;

      PositionMarker {
        center: (mean_center_x, mean_center_y),
        size: mean_size,
      }
    })
    .collect();

  markers
}

pub fn detect_position_markers(image: &DynamicImage) -> Vec<PositionMarker> {
  let grayscale = colorops::grayscale(image);
  let thresholded = crate::threshold::adaptive_gaussian_threshold(&grayscale, 20., 0);
  let hints = detect_position_marker_hints(&thresholded);
  cluster_position_marker_hints(&hints)
}
