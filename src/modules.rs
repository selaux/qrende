use crate::math::*;
use crate::position::{QRCodePositionEstimation, QRCodeVersion};
use bitvec::BitVec;
use image::Luma;
use imageproc::definitions::Image;

pub struct QRCodeModules {
  pub version: QRCodeVersion,
  pub bits: Vec<BitVec>,
}

fn find_marker_centers_along_border(
  n: usize,
  first_point: (f64, f64),
  second_point: (f64, f64),
) -> Vec<(f64, f64)> {
  let vector = vec_between_points(first_point, second_point);
  let normalized = vec_norm(vector);
  let module_size = vec_length(vector) / n as f64;

  (0..n)
    .map(|index| {
      vec_add(
        first_point,
        vec_scalar_mul(normalized, module_size * (0.5 + index as f64)),
      )
    })
    .collect()
}

fn find_intersection_point(
  section1_start: (f64, f64),
  section1_end: (f64, f64),
  section2_start: (f64, f64),
  section2_end: (f64, f64),
) -> (f64, f64) {
  let (ax, ay) = section1_start;
  let (bx, by) = vec_between_points(section1_start, section1_end);
  let (cx, cy) = section2_start;
  let (dx, dy) = vec_between_points(section2_start, section2_end);
  let u = (bx * (cy - ay) + by * (ax - cx)) / (dx * by - dy * bx);

  vec_add(section2_start, (u * dx, u * dy))
}

fn read_modules(image: &Image<Luma<u8>>, position: &QRCodePositionEstimation) -> QRCodeModules {
  let number_of_modules = position.version.number_of_modules() as usize;
  let image_width_minus_1 = image.width() - 1;
  let image_height_minus_1 = image.height() - 1;
  let points_y_left =
    find_marker_centers_along_border(number_of_modules, position.top_left, position.bottom_left);
  let points_y_right =
    find_marker_centers_along_border(number_of_modules, position.top_right, position.bottom_right);
  let points_x_top =
    find_marker_centers_along_border(number_of_modules, position.top_left, position.top_right);
  let points_x_bottom = find_marker_centers_along_border(
    number_of_modules,
    position.bottom_left,
    position.bottom_right,
  );
  let mut bits: Vec<BitVec> = Vec::with_capacity(number_of_modules);

  for x in 0..number_of_modules {
    let mut bits_x = BitVec::with_capacity(number_of_modules);

    for y in 0..number_of_modules {
      let point_y_left = points_y_left[y];
      let point_y_right = points_y_right[y];
      let point_x_top = points_x_top[x];
      let point_x_bottom = points_x_bottom[x];
      let intersection =
        find_intersection_point(point_y_left, point_y_right, point_x_top, point_x_bottom);
      let clamped_intersection = (
        (intersection.0.max(0.).round() as u32).min(image_width_minus_1),
        (intersection.1.max(0.).round() as u32).min(image_height_minus_1),
      );

      let pixel_at_intersection = image.get_pixel(clamped_intersection.0, clamped_intersection.1);

      bits_x.push(pixel_at_intersection[0] == 0);
    }

    bits.push(bits_x);
  }

  QRCodeModules {
    version: position.version.clone(),
    bits,
  }
}

pub fn read_modules_for_all_codes(
  image: &Image<Luma<u8>>,
  positions: &[QRCodePositionEstimation],
) -> Vec<QRCodeModules> {
  positions
    .iter()
    .map(|position| read_modules(image, position))
    .collect()
}
