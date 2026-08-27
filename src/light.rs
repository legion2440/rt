use crate::vec3::{Color, Vec3};

/// A point light: emits light equally in all directions from a single point.
/// `intensity` is the brightness multiplier — this is what you tune to make
/// a scene brighter or darker (see documentation "Changing brightness").
#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f64,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f64) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }

    pub fn white(position: Vec3, intensity: f64) -> Self {
        Light::new(position, Color::white(), intensity)
    }
}
