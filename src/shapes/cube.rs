use crate::hittable::{face_normal, HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// An axis-aligned cube (box), defined by its center and side length.
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    /// Build a cube centered at `center` with edge length `side`.
    pub fn new(center: Vec3, side: f64, material: Material) -> Self {
        let h = side / 2.0;
        Cube {
            min: center - Vec3::new(h, h, h),
            max: center + Vec3::new(h, h, h),
            material,
        }
    }

    /// Build a general axis-aligned box from explicit min/max corners.
    #[allow(dead_code)]
    pub fn from_bounds(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube { min, max, material }
    }
}

impl Hittable for Cube {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Slab method: intersect the ray with each pair of axis-aligned planes,
        // narrowing [t_near, t_far] as we go, tracking which axis produced the
        // tightest near bound (that's the axis of the hit face / normal).
        let mut t_near = t_min;
        let mut t_far = t_max;
        let mut near_axis = 0usize;
        let mut near_sign = -1.0_f64;

        let orig = [ray.origin.x, ray.origin.y, ray.origin.z];
        let dir = [ray.direction.x, ray.direction.y, ray.direction.z];
        let bmin = [self.min.x, self.min.y, self.min.z];
        let bmax = [self.max.x, self.max.y, self.max.z];

        for axis in 0..3 {
            if dir[axis].abs() < 1e-12 {
                if orig[axis] < bmin[axis] || orig[axis] > bmax[axis] {
                    return None;
                }
                continue;
            }
            let inv_d = 1.0 / dir[axis];
            let mut t0 = (bmin[axis] - orig[axis]) * inv_d;
            let mut t1 = (bmax[axis] - orig[axis]) * inv_d;
            let mut sign = -1.0;
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
                sign = 1.0;
            }
            if t0 > t_near {
                t_near = t0;
                near_axis = axis;
                near_sign = sign;
            }
            t_far = t_far.min(t1);
            if t_near >= t_far {
                return None;
            }
        }

        if t_near <= t_min || t_near >= t_max {
            return None;
        }

        let point = ray.at(t_near);
        let mut outward_normal = Vec3::ZERO;
        match near_axis {
            0 => outward_normal.x = near_sign,
            1 => outward_normal.y = near_sign,
            _ => outward_normal.z = near_sign,
        }

        let (normal, front_face) = face_normal(&ray.direction, outward_normal);
        Some(HitRecord {
            t: t_near,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}
