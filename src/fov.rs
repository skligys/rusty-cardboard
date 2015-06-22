extern crate cgmath;

use cgmath::{Intersect, Matrix4, Point3, Ray2, Vector2, Vector3};

use world::{Chunk, Point2, Segment2};

/// Field of view.
pub struct Fov {
  pub vertex: Point2<f32>,  // in xz plane
  pub center_angle_degrees: f32,  // in xz plane, starting from (0, 0, -1), clockwise
  pub angle_degrees: f32,
}

const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 60.0;

impl Fov {
  pub fn inc_center_angle(&mut self, degrees: f32) {
    self.center_angle_degrees += degrees;
    if self.center_angle_degrees > 360.0 {
      self.center_angle_degrees -= 360.0;
    }
  }

  /// A view matrix, eye is at (p.x, p.y + 2.12, p.z), rotating in horizontal plane clockwise
  /// (thus the world is rotating counter-clockwise) and looking at
  /// (p.x + sin α, p.y + 2.12, p.z - cos α).
  pub fn view_matrix(&self, p: &Point3<i32>) -> Matrix4<f32> {
    let y = p.y as f32 + 2.12;  // 0.5 for half block under feet + 1.62 up to eye height.
    let (s, c) = self.center_angle_degrees.to_radians().sin_cos();

    let eye = Point3::new(p.x as f32, y, p.z as f32);
    // Start with α == 0, looking at (p.x, y, p.z - 1).
    let center = Point3::new(p.x as f32 + s, y, p.z as f32 - c);
    let up = Vector3::new(0.0, 1.0, 0.0);
    Matrix4::look_at(&eye, &center, &up)
  }

  /// Perspective projection matrix as frustum matrix.
  pub fn projection_matrix(&self, width: i32, height: i32) -> Matrix4<f32> {
    let inverse_aspect = height as f32 / width as f32;
    let field_of_view = self.angle_degrees.to_radians();

    let right = NEAR_PLANE * (field_of_view / 2.0).tan();
    let left = -right;
    let top = right * inverse_aspect;
    let bottom = -top;
    let near = NEAR_PLANE;
    let far = FAR_PLANE;
    cgmath::frustum(left, right, bottom, top, near, far)
  }

  fn point_visible(&self, xz: &Point2<i32>) -> bool {
    let x_diff = xz.x as f32 - self.vertex.x;
    let z_diff = xz.z as f32 - self.vertex.z;
    // All angles in range [0; 360).
    let point_angle_degrees = x_diff.atan2(-z_diff).to_degrees().normalize();
    let left_angle_degrees = (self.center_angle_degrees - 0.5 * self.angle_degrees).normalize();
    let right_angle_degrees = (self.center_angle_degrees + 0.5 * self.angle_degrees).normalize();

    fn between(left: f32, point: f32, right: f32) -> bool {
      point >= left && point <= right
    }

    if left_angle_degrees < right_angle_degrees {
      // Regular case, no wrapping.
      between(left_angle_degrees, point_angle_degrees, right_angle_degrees)
    } else {
      // One of left/right wrapped.
      if point_angle_degrees >= left_angle_degrees {
        // Point has not wrapped.
        point_angle_degrees <= right_angle_degrees + 360.0
      } else {
        // Point has wrapped.
        between(left_angle_degrees, point_angle_degrees + 360.0, right_angle_degrees + 360.0)
      }
    }
  }

  fn segment_visible(&self, seg: &Segment2<i32>) -> bool {
    // Three cases:
    // 1. If start or end is visible, the segment is visible.
    // 2. If both start and end are outside the FOV on the same side, the segment is invisible.
    // 3. If start and end are outside the FOW on different sides, the segment is visible iff it
    // intersects the FOV's center ray.

    // Case 1.
    if self.point_visible(&seg.start) || self.point_visible(&seg.end) {
      return true;
    }

    // Cases 2.
    let center_angle_opposite = (self.center_angle_degrees + 180.0).normalize();
    let left_angle = (self.center_angle_degrees + 0.5 * self.angle_degrees).normalize();
    let left_fov_center_angle = 0.5 * (center_angle_opposite + left_angle);
    let left_fov = Fov {
      vertex: self.vertex.clone(),
      center_angle_degrees: left_fov_center_angle.normalize(),
      angle_degrees: 0.5 * (360.0 - self.angle_degrees),
    };
    if left_fov.point_visible(&seg.start) && left_fov.point_visible(&seg.end) {
      return false;
    }

    let right_angle = (self.center_angle_degrees - 0.5 * self.angle_degrees).normalize();
    let right_fov_center_angle = 0.5 * (center_angle_opposite + right_angle);
    let right_fov = Fov {
      vertex: self.vertex.clone(),
      center_angle_degrees: right_fov_center_angle.normalize(),
      angle_degrees: 0.5 * (360.0 - self.angle_degrees),
    };
    if right_fov.point_visible(&seg.start) && right_fov.point_visible(&seg.end) {
      return false;
    }

    // Case 3.
    let (s, c) = self.center_angle_degrees.to_radians().sin_cos();
    let center_ray = Ray2::new(self.vertex.as_cgmath(), Vector2::new(s, -c));
    let intersection = (center_ray, seg.as_cgmath()).intersection();
    intersection.is_some()
  }

  pub fn chunk_visible(&self, chunk: &Chunk) -> bool {
    let bounds = chunk.block_bounds();
    let min_x = bounds.min.x;
    let max_x = bounds.max.x;
    let min_z = bounds.min.z;
    let max_z = bounds.max.z;
    let min_x_min_z = Point2::new(min_x, min_z);
    let min_x_max_z = Point2::new(min_x, max_z);
    let max_x_min_z = Point2::new(max_x, min_z);
    let max_x_max_z = Point2::new(max_x, max_z);

    let xz_edges = [
      Segment2::new(min_x_min_z.clone(), min_x_max_z.clone()),
      Segment2::new(min_x_max_z, max_x_max_z.clone()),
      Segment2::new(max_x_max_z, max_x_min_z.clone()),
      Segment2::new(max_x_min_z, min_x_min_z),
    ];

    xz_edges.iter().any(|e| self.segment_visible(e))
  }
}

/// Normalizes an angle in degrees into range [0; 360).
trait NormalizeDegrees {
  fn normalize(self) -> Self;
}

impl NormalizeDegrees for f32 {
  fn normalize(self) -> f32 {
    let floor = (self / 360.0).floor();
    self - 360.0 * floor
  }
}

#[cfg(test)]
mod tests {
  use super::Fov;
  use world::{Point2, Segment2};

  #[test]
  fn point_visible_close_to_left() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 79.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_close_to_left() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 81.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(!fov.point_visible(&xz))
  }

  #[test]
  fn point_visible_close_to_right() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 11.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_close_to_right() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 9.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(!fov.point_visible(&xz))
  }

  #[test]
  fn point_visible_non_zero_vertex_along_z_axis() {
    let fov = Fov {
      vertex: Point2::new(0.0, 2.0),
      center_angle_degrees: 0.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(0, 1);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_non_zero_vertex_along_z_axis() {
    let fov = Fov {
      vertex: Point2::new(0.0, -2.0),
      center_angle_degrees: 0.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(0, -1);
    assert!(!fov.point_visible(&xz))
  }

  #[test]
  fn point_visible_non_zero_vertex_along_x_axis() {
    let fov = Fov {
      vertex: Point2::new(-2.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(-1, 0);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_non_zero_vertex_along_x_axis() {
    let fov = Fov {
      vertex: Point2::new(2.0, 0.0),
      center_angle_degrees: 0.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, 0);
    assert!(!fov.point_visible(&xz))
  }

  #[test]
  fn segment_visible_both_ends_inside_fov() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 34.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, -1), Point2::new(0, -1));
    assert!(fov.segment_visible(&seg))
  }

  #[test]
  fn segment_visible_start_inside_fov() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 36.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, -1), Point2::new(0, -1));
    assert!(fov.segment_visible(&seg))
  }

  #[test]
  fn segment_visible_end_inside_fov() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 360.0 - 34.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, -1), Point2::new(0, -1));
    assert!(fov.segment_visible(&seg))
  }

  #[test]
  fn segment_invisible_both_ends_outside_fov_on_the_same_side_1() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, 1), Point2::new(1, 2));
    assert!(!fov.segment_visible(&seg))
  }

  #[test]
  fn segment_invisible_both_ends_outside_fov_on_the_same_side_2() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, -1), Point2::new(1, -2));
    assert!(!fov.segment_visible(&seg))
  }

  #[test]
  fn segment_visible_both_ends_outside_fov_on_different_sides() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, 1), Point2::new(1, -1));
    assert!(fov.segment_visible(&seg))
  }

  #[test]
  fn segment_invisible_both_ends_outside_fov_on_different_sides() {
    let fov = Fov {
      vertex: Point2::new(0.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(-1, 1), Point2::new(-1, -1));
    assert!(!fov.segment_visible(&seg))
  }

  #[test]
  fn segment_visible_both_ends_outside_fov_on_different_sides_non_zero_vertex_along_x_axis() {
    let fov = Fov {
      vertex: Point2::new(-2.0, 0.0),
      center_angle_degrees: 90.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(-1, 1), Point2::new(-1, -1));
    assert!(fov.segment_visible(&seg))
  }

  #[test]
  fn segment_visible_both_ends_outside_fov_on_different_sides_non_zero_vertex_along_z_axis() {
    let fov = Fov {
      vertex: Point2::new(0.0, 2.0),
      center_angle_degrees: 0.0,
      angle_degrees: 70.0,
    };
    let seg = Segment2::new(Point2::new(1, 1), Point2::new(-1, 1));
    assert!(fov.segment_visible(&seg))
  }
}
