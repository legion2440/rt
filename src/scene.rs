use crate::camera::Camera;
use crate::hittable::{HitRecord, Hittable};
use crate::light::Light;
use crate::ray::Ray;
use crate::vec3::Color;

/// A complete scene: the objects in it, its lights, ambient light color and
/// a background color for rays that hit nothing.
pub struct Scene {
    pub objects: Vec<Box<dyn Hittable>>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub background: Color,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            camera,
            background: Color::new(0.6, 0.75, 0.95),
        }
    }

    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Find the closest hit among all objects, if any.
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;
        for obj in &self.objects {
            if let Some(rec) = obj.hit(ray, t_min, closest) {
                closest = rec.t;
                result = Some(rec);
            }
        }
        result
    }
}
