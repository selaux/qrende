use image::Luma;
use imageproc::definitions::Image;
use imageproc::filter;

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
  let filtered = filter::box_filter(image, radius, radius);
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
  let filtered = filter::gaussian_blur_f32(image, sigma);
  imageproc::map::map_colors2(image, &filtered, |color, mean| {
    let value = if color[0] > mean[0].saturating_sub(diff_threshold) {
      255
    } else {
      0
    };
    image::Luma([value])
  })
}
