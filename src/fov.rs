extern crate cgmath;

use cgmath::{Matrix4, Point3, Vector3};

use world::Point2;

/// Field of view.
pub struct Fov {
  pub center_angle_degrees: f32,  // in xz plane, from (0, 0, -1).
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
    // All in range [0; 360).
    let point_angle_degrees = (xz.x as f32).atan2(-xz.z as f32).to_degrees().normalize();
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
  use world::Point2;

  #[test]
  fn point_visible_close_to_left() {
    let fov = Fov {
      center_angle_degrees: 79.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_close_to_left() {
    let fov = Fov {
      center_angle_degrees: 81.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(!fov.point_visible(&xz))
  }

  #[test]
  fn point_visible_close_to_right() {
    let fov = Fov {
      center_angle_degrees: 11.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(fov.point_visible(&xz))
  }

  #[test]
  fn point_invisible_close_to_right() {
    let fov = Fov {
      center_angle_degrees: 9.0,
      angle_degrees: 70.0,
    };
    let xz = Point2::new(1, -1);
    assert!(!fov.point_visible(&xz))
  }
}
