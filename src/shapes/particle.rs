use crate::hittable::{face_normal, HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
struct Particle {
    center: Vec3,
    radius: f64,
}

/// A deterministic cloud of small spherical particles.
///
/// The cloud is represented as one `Hittable`, so a scene can add dozens of
/// particles without exposing implementation details to the renderer.
pub struct ParticleCloud {
    particles: Vec<Particle>,
    material: Material,
}

impl ParticleCloud {
    /// Build a cloud from explicit particle centers and a shared radius.
    pub fn new(centers: Vec<Vec3>, radius: f64, material: Material) -> Self {
        let particles = centers
            .into_iter()
            .map(|center| Particle { center, radius })
            .collect();
        ParticleCloud {
            particles,
            material,
        }
    }

    /// Build a static fountain-like particle plume with deterministic positions.
    pub fn fountain(origin: Vec3, count: usize, material: Material) -> Self {
        let mut centers = Vec::with_capacity(count);
        let denom = count.saturating_sub(1).max(1) as f64;

        for i in 0..count {
            let t = i as f64 / denom;
            let phase = i as f64 * 2.399_963_229_728_653; // golden angle
            let spread = 0.12 + 0.85 * t;
            let x = origin.x + phase.cos() * spread;
            let z = origin.z + phase.sin() * spread * 0.72;
            let jitter = ((i % 7) as f64 - 3.0) * 0.025;
            let y = origin.y + 0.25 + 3.2 * t - 1.9 * t * t + jitter;
            centers.push(Vec3::new(x, y, z));
        }

        ParticleCloud::new(centers, 0.065, material)
    }
}

impl Hittable for ParticleCloud {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;

        for particle in &self.particles {
            let oc = ray.origin - particle.center;
            let a = ray.direction.length_squared();
            let half_b = oc.dot(&ray.direction);
            let c = oc.length_squared() - particle.radius * particle.radius;
            let discriminant = half_b * half_b - a * c;
            if discriminant < 0.0 {
                continue;
            }

            let sqrt_d = discriminant.sqrt();
            let mut root = (-half_b - sqrt_d) / a;
            if root <= t_min || root >= closest {
                root = (-half_b + sqrt_d) / a;
                if root <= t_min || root >= closest {
                    continue;
                }
            }

            let point = ray.at(root);
            let outward_normal = (point - particle.center) / particle.radius;
            let (normal, front_face) = face_normal(&ray.direction, outward_normal);
            closest = root;
            result = Some(HitRecord {
                t: root,
                point,
                normal,
                front_face,
                material: self.material,
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Color;

    #[test]
    fn particle_cloud_can_be_hit() {
        let cloud = ParticleCloud::new(
            vec![Vec3::ZERO],
            0.5,
            Material::new(Color::new(1.0, 0.5, 0.2)),
        );
        let ray = Ray::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cloud.hit(&ray, 0.001, f64::INFINITY).expect("particle hit");
        assert!((hit.t - 1.5).abs() < 1e-9);
    }
}
