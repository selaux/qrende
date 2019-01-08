use crate::math::*;
use crate::position_markers::PositionMarker;

const DIMENSIONS_THRESHOLD: f64 = 0.1;
const MARKER_SIZE_THRESHOLD: f64 = 0.2;

#[derive(Debug)]
pub struct PositionMarkerTriple {
  pub top_left: (f64, f64),
  pub top_right: (f64, f64),
  pub bottom_left: (f64, f64),
  pub mean_size: f64,
}

fn approx_eq(x: f64, y: f64) -> bool {
  f64::abs(x - y) < DIMENSIONS_THRESHOLD
}

fn find_position_marker_triples(markers: &[PositionMarker]) -> Vec<PositionMarkerTriple> {
  let sqrt_2 = f64::sqrt(2.);
  let number_of_markers = markers.len();
  if number_of_markers < 3 {
    return vec![];
  }

  let mut position_marker_triples = vec![];
  let mut pairwise_distances = vec![vec![0.; number_of_markers]; number_of_markers];
  for i in 0..number_of_markers {
    for y in 0..number_of_markers {
      pairwise_distances[i][y] = euclidean_distance(markers[i].center, markers[y].center);
    }
  }

  for index1 in 0..number_of_markers {
    for index2 in 0..number_of_markers {
      for index3 in 0..number_of_markers {
        if index1 == index2 || index1 == index3 || index2 == index3 {
          continue;
        }

        let distance_1_to_2 = pairwise_distances[index1][index2];
        let distance_1_to_3 = pairwise_distances[index1][index3];
        let distance_2_to_3 = pairwise_distances[index2][index3];
        let total_norm = 1. + 1. + sqrt_2;
        let total_distance = distance_1_to_2 + distance_1_to_3 + distance_2_to_3;
        let normalized_distance_1_to_2 = total_norm * distance_1_to_2 / total_distance;
        let normalized_distance_1_to_3 = total_norm * distance_1_to_3 / total_distance;
        let normalized_distance_2_to_3 = total_norm * distance_2_to_3 / total_distance;

        if approx_eq(normalized_distance_1_to_2, 1.)
          && approx_eq(normalized_distance_1_to_3, 1.)
          && approx_eq(normalized_distance_2_to_3, sqrt_2)
        {
          let marker1 = &markers[index1];
          let marker2 = &markers[index2];
          let marker3 = &markers[index3];
          let mean_marker_size = (marker1.size + marker2.size + marker3.size) / 3.;
          let marker_sizes_match =
            [marker1.size, marker2.size, marker3.size]
              .iter()
              .all(|marker_size| {
                (marker_size - mean_marker_size) / mean_marker_size < MARKER_SIZE_THRESHOLD
              });
          let angle1 = angle(
            vec_between_points(marker1.center, marker2.center),
            vec_between_points(marker1.center, marker3.center),
          );

          if angle1 < 0. && marker_sizes_match {
            position_marker_triples.push(PositionMarkerTriple {
              top_left: markers[index1].center,
              top_right: markers[index2].center,
              bottom_left: markers[index3].center,
              mean_size: (marker1.size + marker2.size + marker3.size) / 3.,
            });
          }
        }
      }
    }
  }

  position_marker_triples
}

#[derive(Debug, Clone)]
pub struct QRCodeVersion(u32);

impl QRCodeVersion {
  pub fn from_estimated_number_of_modules(number_of_modules: f64) -> QRCodeVersion {
    let f_version = ((number_of_modules - 17.) / 4.).round();
    QRCodeVersion(f_version as u32)
  }

  pub fn number_of_modules(&self) -> u32 {
    4 * self.0 + 17
  }
}

#[derive(Debug)]
pub struct QRCodePositionEstimation {
  pub top_left: (f64, f64),
  pub top_right: (f64, f64),
  pub bottom_left: (f64, f64),
  pub bottom_right: (f64, f64),
  pub version: QRCodeVersion,
}

pub fn find_estimated_qr_code_positions(
  markers: &[PositionMarker],
) -> Vec<QRCodePositionEstimation> {
  let position_marker_triples = find_position_marker_triples(markers);

  position_marker_triples
    .iter()
    .map(|triple| {
      let estimated_module_size = triple.mean_size / 7.;
      let half_position_marker_size = 3.5 * estimated_module_size;
      let top_left_to_top_right_direction =
        vec_norm(vec_between_points(triple.top_left, triple.top_right));
      let top_left_to_bottom_left_direction =
        vec_norm(vec_between_points(triple.top_left, triple.bottom_left));
      let top_left = vec_add(
        vec_add(
          triple.top_left,
          vec_scalar_mul(
            top_left_to_top_right_direction,
            -1. * half_position_marker_size,
          ),
        ),
        vec_scalar_mul(
          top_left_to_bottom_left_direction,
          -1. * half_position_marker_size,
        ),
      );
      let top_right = vec_add(
        vec_add(
          triple.top_right,
          vec_scalar_mul(top_left_to_top_right_direction, half_position_marker_size),
        ),
        vec_scalar_mul(
          top_left_to_bottom_left_direction,
          -1. * half_position_marker_size,
        ),
      );
      let bottom_left = vec_add(
        vec_add(
          triple.bottom_left,
          vec_scalar_mul(
            top_left_to_top_right_direction,
            -1. * half_position_marker_size,
          ),
        ),
        vec_scalar_mul(top_left_to_bottom_left_direction, half_position_marker_size),
      );
      let mean_edge_length =
        (euclidean_distance(top_left, top_right) + euclidean_distance(top_left, bottom_left)) / 2.;
      let estimated_number_of_modules = mean_edge_length / estimated_module_size;
      let version = QRCodeVersion::from_estimated_number_of_modules(estimated_number_of_modules);
      let number_of_modules = f64::from(version.number_of_modules());
      let module_size = mean_edge_length / number_of_modules;
      let bottom_right_1 = vec_add(
        top_right,
        vec_scalar_mul(
          top_left_to_bottom_left_direction,
          number_of_modules * module_size,
        ),
      );
      let bottom_right_2 = vec_add(
        bottom_left,
        vec_scalar_mul(
          top_left_to_top_right_direction,
          number_of_modules * module_size,
        ),
      );
      let bottom_right = (
        (bottom_right_1.0 + bottom_right_2.0) / 2.,
        (bottom_right_1.1 + bottom_right_2.1) / 2.,
      );

      QRCodePositionEstimation {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
        version,
      }
    })
    .collect()
}
