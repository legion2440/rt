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
        // Slab method: intersect the ray with each pair of axis-aligned planes.
        // Track both the entering and exiting faces so rays that start inside
        // the cube correctly hit the far/exit face and receive its normal.
        let mut t_near = f64::NEG_INFINITY;
        let mut t_far = f64::INFINITY;
        let mut near_axis = 0usize;
        let mut near_sign = -1.0_f64;
        let mut far_axis = 0usize;
        let mut far_sign = 1.0_f64;

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
            let min_t = (bmin[axis] - orig[axis]) * inv_d;
            let max_t = (bmax[axis] - orig[axis]) * inv_d;
            let (axis_near, axis_far, axis_near_sign, axis_far_sign) = if min_t <= max_t {
                (min_t, max_t, -1.0, 1.0)
            } else {
                (max_t, min_t, 1.0, -1.0)
            };

            if axis_near > t_near {
                t_near = axis_near;
                near_axis = axis;
                near_sign = axis_near_sign;
            }
            if axis_far < t_far {
                t_far = axis_far;
                far_axis = axis;
                far_sign = axis_far_sign;
            }

            if t_near > t_far {
                return None;
            }
        }

        let (t, hit_axis, hit_sign) = if t_near > t_min && t_near < t_max {
            (t_near, near_axis, near_sign)
        } else if t_far > t_min && t_far < t_max {
            (t_far, far_axis, far_sign)
        } else {
            return None;
        };

        let point = ray.at(t);
        let mut outward_normal = Vec3::ZERO;
        match hit_axis {
            0 => outward_normal.x = hit_sign,
            1 => outward_normal.y = hit_sign,
            _ => outward_normal.z = hit_sign,
        }

        let (normal, front_face) = face_normal(&ray.direction, outward_normal);
        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Color;

    #[test]
    fn ray_starting_inside_hits_exit_face() {
        let cube = Cube::new(Vec3::ZERO, 2.0, Material::new(Color::new(0.2, 0.3, 0.4)));
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));

        let hit = cube.hit(&ray, 1e-4, f64::INFINITY).expect("exit face");
        assert!((hit.t - 1.0).abs() < 1e-9);
        assert!((hit.point.x - 1.0).abs() < 1e-9);
        assert!(!hit.front_face);
        assert_eq!(hit.normal, Vec3::new(-1.0, 0.0, 0.0));
    }
}
