use crate::ray::Ray;
use crate::vec3::Vec3;

const CAMERA_EPSILON: f64 = 1e-12;

/// A pinhole camera. Move it by changing `position` and `look_at`; change
/// its angle of view with `fov_deg` (vertical field of view, in degrees).
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    /// `position`: where the camera sits (the eye).
    /// `look_at`: the point the camera is aimed at.
    /// `up`: rough "world up" direction (usually (0,1,0)); used to derive the
    ///       camera's right/up basis, doesn't need to be exactly perpendicular.
    /// `fov_deg`: vertical field of view in degrees (wider = more of the scene visible).
    /// `aspect_ratio`: image width / height.
    pub fn new(position: Vec3, look_at: Vec3, up: Vec3, fov_deg: f64, aspect_ratio: f64) -> Self {
        let theta = fov_deg.to_radians();
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = aspect_ratio * viewport_height;

        // `w` points from the scene towards the camera. If position and
        // look_at coincide, fall back to the conventional +Z camera axis
        // instead of producing a zero-sized viewport.
        let mut w = position - look_at;
        if w.length_squared() < CAMERA_EPSILON {
            w = Vec3::new(0.0, 0.0, 1.0);
        }
        w = w.normalize();

        // A top-down/bottom-up view makes the usual world-up vector parallel
        // to `w`, so their cross product is zero. Pick a non-parallel fallback
        // up vector in that case to keep a valid camera basis.
        let mut right = up.cross(&w);
        if right.length_squared() < CAMERA_EPSILON {
            let fallback_up = if w.y.abs() < 0.999 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };
            right = fallback_up.cross(&w);
        }

        let u = right.normalize(); // camera right
        let v = w.cross(&u).normalize(); // camera up

        let horizontal = u * viewport_width;
        let vertical = v * viewport_height;
        let lower_left_corner = position - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            origin: position,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    /// `s`, `t` in [0,1] range over the image plane (s: left->right, t: bottom->top).
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let dir = self.lower_left_corner + self.horizontal * s + self.vertical * t - self.origin;
        Ray::new(self.origin, dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_down_camera_keeps_a_valid_image_plane() {
        let camera = Camera::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            4.0 / 3.0,
        );

        let left = camera.get_ray(0.25, 0.5);
        let right = camera.get_ray(0.75, 0.5);
        assert!((left.direction - right.direction).length_squared() > 1e-6);
    }

    #[test]
    fn coincident_position_and_target_still_produce_a_ray() {
        let camera = Camera::new(Vec3::ZERO, Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 60.0, 1.0);

        let ray = camera.get_ray(0.5, 0.5);
        assert!(ray.direction.length_squared() > 0.99);
    }
}
