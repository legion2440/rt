use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Information about where a ray hit an object.
pub struct HitRecord {
    pub t: f64,
    pub point: Vec3,
    /// Surface normal, always facing against the incoming ray (i.e. towards
    /// whichever side the ray came from).
    pub normal: Vec3,
    /// True if the ray hit the outside of the surface (entering the object),
    /// false if it hit from the inside (exiting). Needed to get refraction's
    /// eta ratio the right way round — unlike `normal`, this is NOT
    /// recoverable from `normal` alone, since `normal` is always flipped to
    /// oppose the ray.
    pub front_face: bool,
    pub material: Material,
}

/// Anything a ray can intersect. All 4 required primitives (Sphere, Cube,
/// Plane, Cylinder) implement this trait; adding a 5th primitive only
/// requires implementing `hit` for it.
pub trait Hittable: Send + Sync {
    /// Return the closest hit (if any) with `t` in `(t_min, t_max)`.
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

/// Orient `outward_normal` so it always points against the incoming ray,
/// and report whether the ray hit the outside (`front_face`) or inside of
/// the surface. `outward_normal` must be unit length and must genuinely
/// point away from the object (not pre-flipped).
pub fn face_normal(ray_dir: &Vec3, outward_normal: Vec3) -> (Vec3, bool) {
    let front_face = ray_dir.dot(&outward_normal) < 0.0;
    let normal = if front_face {
        outward_normal
    } else {
        -outward_normal
    };
    (normal, front_face)
}
