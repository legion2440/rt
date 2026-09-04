use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::{Color, Vec3};

/// Rendering options that gate the (potentially slow) bonus features.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub reflections: bool,
    pub refractions: bool,
    pub textures: bool,
    pub max_depth: u32,
    /// Global brightness multiplier applied to every light in the scene.
    pub brightness: f64,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            reflections: false,
            refractions: false,
            textures: false,
            max_depth: 5,
            brightness: 1.0,
        }
    }
}

const SHADOW_BIAS: f64 = 1e-4;

/// Trace a ray into the scene and return the color it resolves to. Recurses
/// for reflection/refraction (when enabled) up to `opts.max_depth`.
pub fn trace(scene: &Scene, ray: &Ray, opts: &RenderOptions, depth: u32) -> Color {
    if depth == 0 {
        return Color::black();
    }

    let hit = match scene.hit(ray, 1e-4, f64::INFINITY) {
        Some(h) => h,
        None => return background_color(scene, ray),
    };

    let base_color = if opts.textures {
        hit.material.color_at(hit.point)
    } else {
        hit.material.flat_color()
    };
    let mut color = Color::black();

    // Ambient term: always present, independent of shadows.
    color += base_color * hit.material.ambient;

    // Diffuse + specular contribution from every light, attenuated by shadow rays.
    for light in &scene.lights {
        let to_light = light.position - hit.point;
        let distance = to_light.length();
        let light_dir = to_light / distance;

        let in_shadow = scene
            .hit(
                &Ray::new(hit.point + hit.normal * SHADOW_BIAS, light_dir),
                1e-4,
                distance - SHADOW_BIAS,
            )
            .is_some();

        if in_shadow {
            continue;
        }

        let intensity = light.intensity * opts.brightness;

        let diff = hit.normal.dot(&light_dir).max(0.0);
        if diff > 0.0 && hit.material.diffuse > 0.0 {
            color += base_color.mul_v(&light.color) * (hit.material.diffuse * diff * intensity);
        }

        if hit.material.specular > 0.0 {
            let reflected = (-light_dir).reflect(&hit.normal);
            let view_dir = -ray.direction;
            let spec = reflected
                .dot(&view_dir)
                .max(0.0)
                .powf(hit.material.shininess);
            if spec > 0.0 {
                color += light.color * (hit.material.specular * spec * intensity);
            }
        }
    }

    // Bonus: reflection.
    if opts.reflections && hit.material.reflectivity > 0.0 {
        let reflect_dir = ray.direction.reflect(&hit.normal);
        let reflect_ray = Ray::new(hit.point + hit.normal * SHADOW_BIAS, reflect_dir);
        let reflected_color = trace(scene, &reflect_ray, opts, depth - 1);
        color =
            color * (1.0 - hit.material.reflectivity) + reflected_color * hit.material.reflectivity;
    }

    // Bonus: refraction.
    if opts.refractions && hit.material.transparency > 0.0 {
        // hit.normal always opposes the incoming ray already (see `face_normal`);
        // only the eta ratio flips depending on which side we're entering from.
        let eta_ratio = if hit.front_face {
            1.0 / hit.material.ior
        } else {
            hit.material.ior
        };
        let refracted_color = match ray.direction.refract(&hit.normal, eta_ratio) {
            Some(refracted_dir) => {
                let refract_ray = Ray::new(hit.point - hit.normal * SHADOW_BIAS, refracted_dir);
                trace(scene, &refract_ray, opts, depth - 1)
            }
            None => {
                // Total internal reflection: bounce instead.
                let reflect_dir = ray.direction.reflect(&hit.normal);
                let reflect_ray = Ray::new(hit.point + hit.normal * SHADOW_BIAS, reflect_dir);
                trace(scene, &reflect_ray, opts, depth - 1)
            }
        };
        color =
            color * (1.0 - hit.material.transparency) + refracted_color * hit.material.transparency;
    }

    color
}

/// A simple vertical sky gradient for rays that escape the scene, blending
/// from the scene's background color (up) to white (down/horizon).
fn background_color(scene: &Scene, ray: &Ray) -> Color {
    let t = 0.5 * (ray.direction.normalize().y + 1.0);
    Vec3::white() * (1.0 - t) + scene.background * t
}
