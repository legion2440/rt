use crate::hittable::{face_normal, HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// An infinite flat plane defined by a point on the plane and its normal.
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Plane {
            point,
            normal: normal.normalize(),
            material,
        }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denom = self.normal.dot(&ray.direction);
        if denom.abs() < 1e-9 {
            return None; // Ray is parallel to the plane.
        }
        let t = (self.point - ray.origin).dot(&self.normal) / denom;
        if t <= t_min || t >= t_max {
            return None;
        }
        let point = ray.at(t);
        let (normal, front_face) = face_normal(&ray.direction, self.normal);
        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}
