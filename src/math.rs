pub fn vec_length(vec: (f64, f64)) -> f64 {
  f64::sqrt(f64::powi(vec.0, 2) + f64::powi(vec.1, 2))
}

pub fn vec_between_points(first: (f64, f64), second: (f64, f64)) -> (f64, f64) {
  (second.0 - first.0, second.1 - first.1)
}

pub fn vec_norm(vec: (f64, f64)) -> (f64, f64) {
  let length = vec_length(vec);
  (vec.0 / length, vec.1 / length)
}

pub fn vec_scalar_mul(vec: (f64, f64), scalar: f64) -> (f64, f64) {
  (vec.0 * scalar, vec.1 * scalar)
}

pub fn vec_add(first: (f64, f64), second: (f64, f64)) -> (f64, f64) {
  (first.0 + second.0, first.1 + second.1)
}

pub fn euclidean_distance(first: (f64, f64), second: (f64, f64)) -> f64 {
  vec_length(vec_between_points(first, second))
}

pub fn angle(first: (f64, f64), second: (f64, f64)) -> f64 {
  first.1.atan2(first.0) - second.1.atan2(second.0)
}