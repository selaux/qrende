extern crate image;
extern crate imageproc;

use image::Luma;
use imageproc::definitions::Image;

pub fn threshold(image: &Image<Luma<u8>>, threshold: u8) -> Image<Luma<u8>> {
    imageproc::map::map_colors(image, |color| {
        let value = if color[0] > threshold { 255 } else { 0 };
        image::Luma([value])
    })
}

pub fn adaptive_box_threshold(
    image: &Image<Luma<u8>>,
    radius: u32,
    diff_threshold: u8,
) -> Image<Luma<u8>> {
    let filtered = imageproc::filter::box_filter(image, radius, radius);
    imageproc::map::map_colors2(image, &filtered, |color, mean| {
        let value = if color[0] > mean[0].saturating_sub(diff_threshold) {
            255
        } else {
            0
        };
        image::Luma([value])
    })
}

pub fn adaptive_gaussian_threshold(
    image: &Image<Luma<u8>>,
    sigma: f32,
    diff_threshold: u8,
) -> Image<Luma<u8>> {
    let filtered = imageproc::filter::gaussian_blur_f32(image, sigma);
    imageproc::map::map_colors2(image, &filtered, |color, mean| {
        let value = if color[0] > mean[0].saturating_sub(diff_threshold) {
            255
        } else {
            0
        };
        image::Luma([value])
    })
}

const EXPECTED_RATIOS: [f64; 5] = [1., 1., 3., 1., 1.];
const SYMMETRY_THRESHOLD: f64 = 0.4;
const VARIANCE_THRESHOLD: f64 = 0.2;

#[derive(Debug)]
struct PositionMarkerHint {
    start: u32,
    end: u32,
    black_border1_count: u32,
    white_inner1_count: u32,
    black_inner_count: u32,
    white_inner2_count: u32,
    black_border2_count: u32,
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
    Found(PositionMarkerHint),
}

pub fn is_white(pixel: Luma<u8>) -> bool {
    pixel[0] == 255
}

fn is_symmetric(scan_result: &PositionMarkerHint) -> bool {
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

fn ratios_match(scan_result: &PositionMarkerHint) -> bool {
    let expected_total: f64 = EXPECTED_RATIOS.iter().sum();
    let expected = EXPECTED_RATIOS.iter().map(|f| f / expected_total);
    let result = [
        scan_result.black_border1_count,
        scan_result.white_inner1_count,
        scan_result.black_inner_count,
        scan_result.white_inner2_count,
        scan_result.black_border2_count,
    ];
    let total = f64::from(result.iter().sum::<u32>());
    let got = result.iter().map(|v| f64::from(*v) / total);
    let sum: f64 = got
        .zip(expected)
        .map(|(got, expected)| f64::abs(got - expected))
        .sum();

    sum < VARIANCE_THRESHOLD
}

fn is_valid_match(scan_result: &PositionMarkerHint) -> bool {
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
                ScanState::Found(PositionMarkerHint {
                    start: *start,
                    end: pos - 1,
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

pub fn detect_position_marker_hints(image: &Image<Luma<u8>>) -> Vec<(u32, u32)> {
    let mut found: Vec<(u32, u32)> = vec![];

    for x in 0..image.width() {
        let mut state = if is_white(*image.get_pixel(x, 0)) {
            ScanState::InWhite
        } else {
            ScanState::InBlack
        };
        let mut y: u32 = 1;

        while y < image.height() {
            let (new_y, new_state) = advance_state(&state, y, y + 1, *image.get_pixel(x, y));
            y = new_y;
            state = new_state;
            if let ScanState::Found(scan_result) = &state {
                let middle = (scan_result.start + scan_result.end) / 2;
                found.push((x, middle));
            }
        }
    }

    for y in 0..image.height() {
        let mut state = if is_white(*image.get_pixel(0, y)) {
            ScanState::InWhite
        } else {
            ScanState::InBlack
        };
        let mut x: u32 = 1;

        while x < image.width() {
            let (new_x, new_state) = advance_state(&state, x, x + 1, *image.get_pixel(x, y));
            x = new_x;
            state = new_state;
            if let ScanState::Found(scan_result) = &state {
                let middle = (scan_result.start + scan_result.end) / 2;
                found.push((middle, y));
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    use image::DynamicImage;

    #[test]
    fn it_should_detect_all_barcodes() {
        let blackbox_tests = PathBuf::from(env::var("BLACKBOX_TESTS").unwrap());
        let files: Vec<_> = fs::read_dir(&blackbox_tests)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension() == Some(&OsString::from("png")))
            .collect();

        for file in files {
            let img = image::open(&file).unwrap();
            let output_path: PathBuf = [
                PathBuf::from("test_output"),
                PathBuf::from(file.file_name().unwrap()),
            ]
            .iter()
            .collect();
            let grayscale = image::imageops::colorops::grayscale(&img);
            let thresholded = crate::adaptive_gaussian_threshold(&grayscale, 20., 0);
            let found = crate::detect_position_marker_hints(&thresholded);

            let mut image = DynamicImage::ImageLuma8(thresholded).to_rgb();
            for (x, y) in found {
                let pixel = image.get_pixel_mut(x, y);
                pixel[0] = 255;
                pixel[1] = 0;
                pixel[2] = 0;
            }

            image.save(&output_path).unwrap();
        }
    }
}
