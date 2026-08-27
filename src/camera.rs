use crate::ray::Ray;
use crate::vec3::Vec3;

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

        let w = (position - look_at).normalize(); // points from scene towards camera
        let u = up.cross(&w).normalize(); // camera right
        let v = w.cross(&u); // camera up

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
