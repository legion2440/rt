use crate::vec3::Color;

/// Surface appearance: how a hit point turns into a base color, and how it
/// responds to light (ambient/diffuse/specular) and to the bonus effects
/// (reflection, refraction).
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub texture: Texture,
    /// Fraction of ambient light reflected back regardless of light visibility, [0,1].
    pub ambient: f64,
    /// Fraction of diffuse (Lambertian) light reflected, [0,1].
    pub diffuse: f64,
    /// Fraction of specular (shiny highlight) light reflected, [0,1].
    pub specular: f64,
    /// Specular highlight tightness (higher = smaller, sharper highlight).
    pub shininess: f64,
    /// Mirror reflectivity, [0,1]. Only applied when the --reflect bonus flag is on.
    pub reflectivity: f64,
    /// Transparency / refraction strength, [0,1]. Only applied when --refract is on.
    pub transparency: f64,
    /// Index of refraction (e.g. 1.5 for glass) used when transparency > 0.
    pub ior: f64,
}

/// How a material picks its base color at a given surface point.
#[derive(Clone, Copy, Debug)]
pub enum Texture {
    /// A single flat color everywhere on the surface.
    Solid(Color),
    /// A procedural 2-color checkerboard pattern (bonus, enabled with --texture).
    /// `scale` controls the size of each square.
    Checker(Color, Color, f64),
}

impl Material {
    pub fn new(color: Color) -> Self {
        Material {
            texture: Texture::Solid(color),
            ambient: 0.1,
            diffuse: 0.8,
            specular: 0.3,
            shininess: 32.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.5,
        }
    }

    pub fn checker(c1: Color, c2: Color, scale: f64) -> Self {
        let mut m = Material::new(c1);
        m.texture = Texture::Checker(c1, c2, scale);
        m
    }

    // -- Builder-style setters: chain them when constructing a scene. --

    pub fn ambient(mut self, v: f64) -> Self {
        self.ambient = v;
        self
    }
    pub fn diffuse(mut self, v: f64) -> Self {
        self.diffuse = v;
        self
    }
    pub fn specular(mut self, v: f64) -> Self {
        self.specular = v;
        self
    }
    pub fn shininess(mut self, v: f64) -> Self {
        self.shininess = v;
        self
    }
    pub fn reflectivity(mut self, v: f64) -> Self {
        self.reflectivity = v;
        self
    }
    pub fn transparency(mut self, v: f64) -> Self {
        self.transparency = v;
        self
    }
    pub fn ior(mut self, v: f64) -> Self {
        self.ior = v;
        self
    }

    /// Resolve the base color of this material at a given surface point.
    /// Used when the --texture bonus flag is enabled.
    pub fn color_at(&self, p: crate::vec3::Vec3) -> Color {
        match self.texture {
            Texture::Solid(c) => c,
            Texture::Checker(c1, c2, scale) => {
                let s = 1.0 / scale;
                let sum = (p.x * s).floor() as i64 + (p.y * s).floor() as i64 + (p.z * s).floor() as i64;
                if sum.rem_euclid(2) == 0 {
                    c1
                } else {
                    c2
                }
            }
        }
    }

    /// A single flat color for this material, ignoring any procedural pattern.
    /// Used when the --texture bonus flag is disabled.
    pub fn flat_color(&self) -> Color {
        match self.texture {
            Texture::Solid(c) => c,
            Texture::Checker(c1, _, _) => c1,
        }
    }
}
