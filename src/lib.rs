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

const BLACK_INNER_RATIO: f64 = 3.;
const THRESHOLD: f64 = 0.5;

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
    Found {
        start: u32,
        end: u32,
    },
}

pub fn is_white(pixel: &Luma<u8>) -> bool {
    pixel[0] == 255
}

pub fn calculate_ratio(other_count: u32, my_count: u32) -> f64 {
    f64::from(my_count) / f64::from(other_count)
}

pub fn exceeds_ratio(other_count: u32, my_count: u32, ratio: f64) -> bool {
    calculate_ratio(other_count, my_count) > ratio * (1. + THRESHOLD)
}

pub fn below_ratio(other_count: u32, my_count: u32, ratio: f64) -> bool {
    calculate_ratio(other_count, my_count) < ratio * (1. - THRESHOLD)
}

fn advance_state(state: &ScanState, pos: u32, pixel: &Luma<u8>) -> ScanState {
    let is_white = is_white(pixel);

    match state {
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
            let new_count = if is_white {
                white_inner1_count + 1
            } else {
                *white_inner1_count
            };
            let white_exceeds_ratio = exceeds_ratio(*black_border1_count, new_count, 1.);
            let white_below_ratio = below_ratio(*black_border1_count, new_count, 1.);

            match (is_white, white_exceeds_ratio, white_below_ratio) {
                (true, true, _) => ScanState::InWhite,
                (true, false, _) => ScanState::WhiteInner1 {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: new_count,
                },
                (false, _, true) => ScanState::BlackBorder1 {
                    start: pos,
                    black_border1_count: 1,
                },
                (false, _, false) => ScanState::BlackInner {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: 1,
                },
            }
        }
        ScanState::BlackInner {
            start,
            black_border1_count,
            white_inner1_count,
            black_inner_count,
        } => {
            let new_count = black_inner_count + 1;
            let black_exceeds_ratio =
                exceeds_ratio(*white_inner1_count, new_count, BLACK_INNER_RATIO);
            let black_below_ratio =
                below_ratio(*white_inner1_count, *black_inner_count, BLACK_INNER_RATIO);

            match (is_white, black_exceeds_ratio, black_below_ratio) {
                (false, true, _) => ScanState::BlackBorder1 {
                    start: pos - black_inner_count,
                    black_border1_count: *black_inner_count,
                },
                (false, false, _) => ScanState::BlackInner {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: new_count,
                },
                (true, _, true) => ScanState::InWhite,
                (true, _, false) => ScanState::WhiteInner2 {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: *black_inner_count,
                    white_inner2_count: 1,
                },
            }
        }
        ScanState::WhiteInner2 {
            start,
            black_border1_count,
            white_inner1_count,
            black_inner_count,
            white_inner2_count,
        } => {
            let new_count = white_inner2_count + 1;
            let white_exceeds_ratio =
                exceeds_ratio(*black_inner_count, new_count, 1. / BLACK_INNER_RATIO);
            let white_below_ratio = below_ratio(
                *black_inner_count,
                *black_inner_count,
                1. / BLACK_INNER_RATIO,
            );

            match (is_white, white_exceeds_ratio, white_below_ratio) {
                (true, true, _) => ScanState::InWhite,
                (true, false, _) => ScanState::WhiteInner2 {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: *black_inner_count,
                    white_inner2_count: new_count,
                },
                (false, _, true) => ScanState::BlackBorder1 {
                    start: pos,
                    black_border1_count: 1,
                },
                (false, _, false) => ScanState::BlackBorder2 {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: *black_inner_count,
                    white_inner2_count: *white_inner2_count,
                    black_border2_count: 1,
                },
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
            let new_count = black_border2_count + 1;
            let black_exceeds_ratio = exceeds_ratio(*white_inner2_count, new_count, 1.);
            let black_below_ratio = below_ratio(*white_inner2_count, *black_border2_count, 1.);

            match (is_white, black_exceeds_ratio, black_below_ratio) {
                (false, true, _) => ScanState::BlackBorder1 {
                    start: pos - black_inner_count,
                    black_border1_count: *black_inner_count,
                },
                (false, false, _) => ScanState::BlackBorder2 {
                    start: *start,
                    black_border1_count: *black_border1_count,
                    white_inner1_count: *white_inner1_count,
                    black_inner_count: *black_inner_count,
                    white_inner2_count: *white_inner2_count,
                    black_border2_count: new_count,
                },
                (true, _, true) => ScanState::InWhite,
                (true, _, false) => ScanState::Found {
                    start: *start,
                    end: pos - 1,
                },
            }
        }
    }
}

pub fn detect_position_markers_vertical(image: &Image<Luma<u8>>) -> Vec<(u32, u32)> {
    let mut found: Vec<(u32, u32)> = vec![];
    for x in 0..image.width() {
        let mut state = if is_white(image.get_pixel(x, 0)) {
            ScanState::InWhite
        } else {
            ScanState::InBlack
        };
        for y in 1..image.height() {
            state = advance_state(&state, y, image.get_pixel(x, y));
            if let ScanState::Found { start, end } = state {
                let middle = (start + end) / 2;
                found.push((x, middle));
            }
        }
    }
    for y in 0..image.height() {
        let mut state = if is_white(image.get_pixel(0, y)) {
            ScanState::InWhite
        } else {
            ScanState::InBlack
        };
        for x in 1..image.width() {
            state = advance_state(&state, x, image.get_pixel(x, y));
            if let ScanState::Found { start, end } = state {
                let middle = (start + end) / 2;
                found.push((middle, y));
            }
        }
    }
    found
}

#[derive(Debug)]
pub struct Point(pub u32, pub u32);

#[derive(Debug)]
pub struct Polygon(pub Vec<Point>);

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
            let found = crate::detect_position_markers_vertical(&thresholded);

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
