extern crate dbscan;
extern crate image;
extern crate imageproc;

pub mod decode;
pub mod math;
pub mod modules;
pub mod position;
pub mod position_markers;
pub mod threshold;

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    use image::DynamicImage;
    use image::GenericImage;
    use rusttype;

    fn all_blackbox_files() -> Vec<(String, String, String)> {
        let blackbox_tests = env::var("BLACKBOX_TESTS").unwrap();

        blackbox_tests
            .split(';')
            .flat_map(|index_and_test_dir| {
                let mut index_and_test_dir_iter = index_and_test_dir.split(':');
                let index = String::from(index_and_test_dir_iter.next().unwrap());
                let dir = index_and_test_dir_iter.next().unwrap();
                let files: Vec<_> = fs::read_dir(&dir)
                    .map_err(|e| format!("Could not read: {:?} {:?}", dir, e))
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .filter(|path| path.extension() == Some(&OsString::from("png")))
                    .map(|path| {
                        (
                            index.clone(),
                            String::from(path.parent().unwrap().to_string_lossy()),
                            String::from(path.file_name().unwrap().to_string_lossy()),
                        )
                    })
                    .collect();
                files
            })
            .collect()
    }

    #[test]
    fn it_should_detect_all_barcodes() {
        let files: Vec<_> = all_blackbox_files();
        let font_data: &[u8] = include_bytes!(env!("TEST_FONT"));
        let font: rusttype::Font<'static> = rusttype::Font::from_bytes(font_data).unwrap();
        let mut diff_markers = 0;
        let mut diff_marker_positions = 0;
        let RECONSTRUCTED_PIXEL_SIZE = 4;

        for (index_string, directory, file_name) in &files {
            let input_path: PathBuf = [directory, file_name].iter().collect();
            let output_path: PathBuf = ["test_output", &format!("{}-{}", index_string, file_name)]
                .iter()
                .collect();
            let img = image::open(&input_path)
                .map_err(|e| format!("Could not find: {:?} {:?}", input_path, e))
                .unwrap();
            let grayscale = image::imageops::colorops::grayscale(&img);
            let thresholded = crate::threshold::adaptive_gaussian_threshold(&grayscale, 20., 0);
            let hints = crate::position_markers::detect_position_marker_hints(&thresholded);
            let markers = crate::position_markers::cluster_position_marker_hints(&hints);
            let positions = crate::position::find_estimated_qr_code_positions(&markers);
            let codes = crate::modules::read_modules_for_all_codes(&thresholded, &positions);
            let format_infos: Vec<_> = codes
                .iter()
                .map(|c| crate::decode::decode_format_information(&c))
                .collect();
            println!("{:?}", format_infos);

            // let mut img = DynamicImage::ImageLuma8(grayscale).to_rgb();
            let total_codes_width: u32 = codes
                .iter()
                .map(|m| m.version.number_of_modules() * RECONSTRUCTED_PIXEL_SIZE)
                .sum();
            let mut image = DynamicImage::new_rgb8(
                2 * grayscale.width() + total_codes_width,
                grayscale.height() + total_codes_width,
            )
            .to_rgb();

            image::imageops::replace(&mut image, &img.to_rgb(), 0, 0);
            image::imageops::replace(
                &mut image,
                &DynamicImage::ImageLuma8(thresholded).to_rgb(),
                grayscale.width(),
                0,
            );

            for hint in hints {
                let pixel = image.get_pixel_mut(hint.center.0 as u32, hint.center.1 as u32);
                pixel[0] = 255;
                pixel[1] = 0;
                pixel[2] = 0;
            }

            diff_markers += i32::abs((markers.len() as i32) - 3);
            for marker in markers {
                let rect = imageproc::rect::Rect::at(
                    f64::round(marker.center.0 - marker.size / 2.) as i32,
                    f64::round(marker.center.1 - marker.size / 2.) as i32,
                )
                .of_size(
                    f64::round(marker.size) as u32,
                    f64::round(marker.size) as u32,
                );

                imageproc::drawing::draw_hollow_rect_mut(&mut image, rect, image::Rgb([0, 255, 0]));
            }

            diff_marker_positions += i32::abs((positions.len() as i32) - 1);
            for triangle in positions {
                imageproc::drawing::draw_line_segment_mut(
                    &mut image,
                    (triangle.top_left.0 as f32, triangle.top_left.1 as f32),
                    (triangle.top_right.0 as f32, triangle.top_right.1 as f32),
                    image::Rgb([0, 0, 255]),
                );
                imageproc::drawing::draw_line_segment_mut(
                    &mut image,
                    (triangle.top_left.0 as f32, triangle.top_left.1 as f32),
                    (triangle.bottom_left.0 as f32, triangle.bottom_left.1 as f32),
                    image::Rgb([0, 0, 255]),
                );
                imageproc::drawing::draw_line_segment_mut(
                    &mut image,
                    (triangle.top_right.0 as f32, triangle.top_right.1 as f32),
                    (
                        triangle.bottom_right.0 as f32,
                        triangle.bottom_right.1 as f32,
                    ),
                    image::Rgb([0, 0, 255]),
                );
                imageproc::drawing::draw_line_segment_mut(
                    &mut image,
                    (
                        triangle.bottom_right.0 as f32,
                        triangle.bottom_right.1 as f32,
                    ),
                    (triangle.bottom_left.0 as f32, triangle.bottom_left.1 as f32),
                    image::Rgb([0, 0, 255]),
                );
                imageproc::drawing::draw_text_mut(
                    &mut image,
                    image::Rgb([0, 0, 255]),
                    triangle.bottom_left.0.round() as u32,
                    triangle.bottom_left.1.round() as u32,
                    rusttype::Scale::uniform(16.),
                    &font,
                    &format!("Version: {:?}", triangle.version),
                );
            }

            let mut current_offset = grayscale.width() * 2;
            for code in codes {
                let number_of_modules = code.version.number_of_modules();
                for x in 0..number_of_modules {
                    for y in 0..number_of_modules {
                        let pixel = if code.bits[x as usize][y as usize] {
                            image::Rgb([0, 0, 0])
                        } else {
                            image::Rgb([255, 255, 255])
                        };

                        for s1 in 0..RECONSTRUCTED_PIXEL_SIZE {
                            for s2 in 0..RECONSTRUCTED_PIXEL_SIZE {
                                image.put_pixel(
                                    x * RECONSTRUCTED_PIXEL_SIZE + current_offset + s1,
                                    y * RECONSTRUCTED_PIXEL_SIZE + s2,
                                    pixel,
                                );
                            }
                        }
                    }
                }
                current_offset += number_of_modules * RECONSTRUCTED_PIXEL_SIZE;
            }

            image
                .save(&output_path)
                .map_err(|e| format!("Could not write to {:?} {:?}", output_path, e))
                .unwrap();
        }

        println!(
            "Percentage diff in markers found: {}",
            f64::from(diff_markers) / (3. * files.len() as f64)
        );
        println!(
            "Percentage diff in marker positions found: {}",
            f64::from(diff_marker_positions) / files.len() as f64
        );
    }
}
