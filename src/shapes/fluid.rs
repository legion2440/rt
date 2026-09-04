use crate::hittable::{face_normal, HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A finite procedural wavy surface used to model a simple fluid sheet.
///
/// Height is defined by `y = base + amplitude * sin(f*x) * cos(f*z)` inside
/// a finite X/Z rectangle. Ray intersections are found inside the surface's
/// bounding box and refined by bisection.
pub struct FluidSurface {
    pub center: Vec3,
    pub half_x: f64,
    pub half_z: f64,
    pub amplitude: f64,
    pub frequency: f64,
    pub material: Material,
}

impl FluidSurface {
    pub fn new(
        center: Vec3,
        half_x: f64,
        half_z: f64,
        amplitude: f64,
        frequency: f64,
        material: Material,
    ) -> Self {
        FluidSurface {
            center,
            half_x,
            half_z,
            amplitude: amplitude.abs(),
            frequency,
            material,
        }
    }

    fn height_at(&self, x: f64, z: f64) -> f64 {
        self.center.y
            + self.amplitude
                * ((x - self.center.x) * self.frequency).sin()
                * ((z - self.center.z) * self.frequency).cos()
    }

    fn surface_fn(&self, p: Vec3) -> f64 {
        p.y - self.height_at(p.x, p.z)
    }

    fn outward_normal(&self, p: Vec3) -> Vec3 {
        let x = (p.x - self.center.x) * self.frequency;
        let z = (p.z - self.center.z) * self.frequency;
        let dh_dx = self.amplitude * self.frequency * x.cos() * z.cos();
        let dh_dz = -self.amplitude * self.frequency * x.sin() * z.sin();
        Vec3::new(-dh_dx, 1.0, -dh_dz).normalize()
    }

    fn bounding_interval(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<(f64, f64)> {
        let pad = 1e-4;
        let min = Vec3::new(
            self.center.x - self.half_x,
            self.center.y - self.amplitude - pad,
            self.center.z - self.half_z,
        );
        let max = Vec3::new(
            self.center.x + self.half_x,
            self.center.y + self.amplitude + pad,
            self.center.z + self.half_z,
        );

        let orig = [ray.origin.x, ray.origin.y, ray.origin.z];
        let dir = [ray.direction.x, ray.direction.y, ray.direction.z];
        let bmin = [min.x, min.y, min.z];
        let bmax = [max.x, max.y, max.z];
        let mut enter = t_min;
        let mut exit = t_max;

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
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }
            enter = enter.max(t0);
            exit = exit.min(t1);
            if enter >= exit {
                return None;
            }
        }

        if enter.is_finite() && exit.is_finite() {
            Some((enter, exit))
        } else {
            None
        }
    }
}

impl Hittable for FluidSurface {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let (enter, exit) = self.bounding_interval(ray, t_min, t_max)?;
        let steps = 96usize;
        let mut prev_t = enter;
        let mut prev_f = self.surface_fn(ray.at(prev_t));

        for i in 1..=steps {
            let t = enter + (exit - enter) * (i as f64 / steps as f64);
            let f = self.surface_fn(ray.at(t));

            if f.abs() < 1e-8 || prev_f.signum() != f.signum() {
                let mut lo = prev_t;
                let mut hi = t;
                let mut flo = prev_f;

                for _ in 0..18 {
                    let mid = (lo + hi) * 0.5;
                    let fm = self.surface_fn(ray.at(mid));
                    if fm.abs() < 1e-10 {
                        lo = mid;
                        hi = mid;
                        break;
                    }
                    if flo.signum() != fm.signum() {
                        hi = mid;
                    } else {
                        lo = mid;
                        flo = fm;
                    }
                }

                let root = (lo + hi) * 0.5;
                if root <= t_min || root >= t_max {
                    return None;
                }
                let point = ray.at(root);
                if (point.x - self.center.x).abs() > self.half_x + 1e-8
                    || (point.z - self.center.z).abs() > self.half_z + 1e-8
                {
                    return None;
                }
                let outward_normal = self.outward_normal(point);
                let (normal, front_face) = face_normal(&ray.direction, outward_normal);
                return Some(HitRecord {
                    t: root,
                    point,
                    normal,
                    front_face,
                    material: self.material,
                });
            }

            prev_t = t;
            prev_f = f;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Color;

    #[test]
    fn vertical_ray_hits_fluid_surface() {
        let fluid = FluidSurface::new(
            Vec3::new(0.0, 0.5, 0.0),
            2.0,
            2.0,
            0.15,
            2.0,
            Material::new(Color::new(0.1, 0.4, 0.9)),
        );
        let ray = Ray::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = fluid.hit(&ray, 0.001, f64::INFINITY).expect("fluid hit");
        assert!((hit.point.y - 0.5).abs() < 1e-5);
    }
}
