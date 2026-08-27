use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub};

/// A 3-component vector used for points, directions, normals and RGB colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn splat(v: f64) -> Self {
        Vec3::new(v, v, v)
    }

    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            *self
        } else {
            *self / len
        }
    }

    /// Component-wise multiplication (used for tinting colors by light color).
    pub fn mul_v(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    /// Reflect `self` (an incoming direction) around normal `n` (must be normalized).
    pub fn reflect(&self, n: &Vec3) -> Vec3 {
        *self - *n * (2.0 * self.dot(n))
    }

    /// Refract `self` (incoming, normalized, pointing towards the surface) through a
    /// surface with normal `n` (normalized, pointing against the incoming ray), given
    /// the ratio of indices of refraction `eta_ratio` = eta_incident / eta_transmitted.
    /// Returns `None` on total internal reflection.
    pub fn refract(&self, n: &Vec3, eta_ratio: f64) -> Option<Vec3> {
        let uv = self.normalize();
        let cos_theta = (-uv).dot(n).min(1.0);
        let r_out_perp = (uv + *n * cos_theta) * eta_ratio;
        let k = 1.0 - r_out_perp.length_squared();
        if k < 0.0 {
            None
        } else {
            let r_out_parallel = *n * (-k.sqrt());
            Some(r_out_perp + r_out_parallel)
        }
    }

    pub fn clamp(&self, lo: f64, hi: f64) -> Vec3 {
        Vec3::new(
            self.x.clamp(lo, hi),
            self.y.clamp(lo, hi),
            self.z.clamp(lo, hi),
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, o: Vec3) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, s: f64) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f64) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, s: f64) {
        self.x /= s;
        self.y /= s;
        self.z /= s;
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 index out of range"),
        }
    }
}

/// RGB color; components are conventionally in [0.0, 1.0] but may exceed 1.0
/// transiently before final tone-mapping/clamping at output time.
pub type Color = Vec3;

impl Color {
    pub fn white() -> Color {
        Color::new(1.0, 1.0, 1.0)
    }
    pub fn black() -> Color {
        Color::ZERO
    }
    /// Build a color from 0-255 byte components.
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }
}
