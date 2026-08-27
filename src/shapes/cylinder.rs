use crate::hittable::{face_normal, HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A finite cylinder (with end caps), defined by the center of its base,
/// an axis direction, a radius and a height along that axis.
pub struct Cylinder {
    pub base: Vec3,
    pub axis: Vec3, // normalized
    pub radius: f64,
    pub height: f64,
    pub material: Material,
}

impl Cylinder {
    pub fn new(base: Vec3, axis: Vec3, radius: f64, height: f64, material: Material) -> Self {
        Cylinder {
            base,
            axis: axis.normalize(),
            radius,
            height,
            material,
        }
    }

    fn cap_hit(&self, ray: &Ray, t_min: f64, t_max: f64, center: Vec3, outward_normal: Vec3) -> Option<f64> {
        let denom = outward_normal.dot(&ray.direction);
        if denom.abs() < 1e-9 {
            return None;
        }
        let t = (center - ray.origin).dot(&outward_normal) / denom;
        if t <= t_min || t >= t_max {
            return None;
        }
        let p = ray.at(t);
        if (p - center).length_squared() <= self.radius * self.radius {
            Some(t)
        } else {
            None
        }
    }
}

impl Hittable for Cylinder {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let top = self.base + self.axis * self.height;
        let oc = ray.origin - self.base;

        let d_dot_a = ray.direction.dot(&self.axis);
        let oc_dot_a = oc.dot(&self.axis);

        let d_perp = ray.direction - self.axis * d_dot_a;
        let oc_perp = oc - self.axis * oc_dot_a;

        let a = d_perp.length_squared();
        let b = 2.0 * d_perp.dot(&oc_perp);
        let c = oc_perp.length_squared() - self.radius * self.radius;

        let mut best_t: Option<f64> = None;
        let mut best_normal = Vec3::ZERO;

        // Side surface.
        if a > 1e-12 {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                let sqrt_d = discriminant.sqrt();
                for root in [(-b - sqrt_d) / (2.0 * a), (-b + sqrt_d) / (2.0 * a)] {
                    if root > t_min && root < t_max {
                        let h = (ray.at(root) - self.base).dot(&self.axis);
                        if h >= 0.0 && h <= self.height && best_t.map_or(true, |bt| root < bt) {
                            let p = ray.at(root);
                            let axis_point = self.base + self.axis * h;
                            let outward_normal = (p - axis_point).normalize();
                            best_t = Some(root);
                            best_normal = outward_normal;
                        }
                    }
                }
            }
        }

        // Bottom cap.
        if let Some(t) = self.cap_hit(ray, t_min, t_max, self.base, -self.axis) {
            if best_t.map_or(true, |bt| t < bt) {
                best_t = Some(t);
                best_normal = -self.axis;
            }
        }
        // Top cap.
        if let Some(t) = self.cap_hit(ray, t_min, t_max, top, self.axis) {
            if best_t.map_or(true, |bt| t < bt) {
                best_t = Some(t);
                best_normal = self.axis;
            }
        }

        let t = best_t?;
        let point = ray.at(t);
        let (normal, front_face) = face_normal(&ray.direction, best_normal);
        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}
