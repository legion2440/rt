//! The 4 required demo scenes, plus the plumbing to override the camera
//! from the command line. This file doubles as a worked example for the
//! documentation: see docs/DOCUMENTATION.md for a walk-through of the code
//! below.

use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::scene::Scene;
use crate::shapes::{Cube, Cylinder, Plane, Sphere};
use crate::vec3::{Color, Vec3};

/// Camera parameters, so they can be overridden from the CLI without
/// touching scene code (see `--camera`, `--look-at`, `--fov`).
#[derive(Clone, Copy, Debug)]
pub struct CameraParams {
    pub position: Vec3,
    pub look_at: Vec3,
    pub fov: f64,
}

pub const GROUND: Color = Color::new(0.55, 0.55, 0.6);
pub const GROUND2: Color = Color::new(0.9, 0.9, 0.9);

fn ground_material() -> Material {
    Material::checker(GROUND, GROUND2, 1.0)
        .diffuse(0.9)
        .specular(0.05)
        .shininess(8.0)
        .ambient(0.15)
}

fn soft_fill_light() -> Light {
    Light::new(Vec3::new(-3.0, 3.0, 5.0), Color::new(0.9, 0.9, 1.0), 0.3)
}

/// Build one of the 4 numbered demo scenes. `aspect_ratio` = width/height of
/// the target image. `camera_override`, when set, replaces the scene's
/// default camera (used by the `--camera`/`--look-at`/`--fov` CLI flags).
pub fn build(preset: u32, aspect_ratio: f64, camera_override: Option<CameraParams>) -> Scene {
    let (mut scene, default_cam) = match preset {
        1 => scene_sphere(),
        2 => scene_plane_and_cube(),
        3 => scene_all_objects(default_camera_3()),
        4 => scene_all_objects(default_camera_4()),
        other => panic!("unknown --scene {other}, expected 1, 2, 3 or 4"),
    };
    let cam = camera_override.unwrap_or(default_cam);
    scene.camera = Camera::new(
        cam.position,
        cam.look_at,
        Vec3::new(0.0, 1.0, 0.0),
        cam.fov,
        aspect_ratio,
    );
    scene
}

/// Scene 1: a sphere above a ground plane, with a bright key light and a
/// soft camera-side fill so the sphere's base color remains readable.
fn scene_sphere() -> (Scene, CameraParams) {
    let cam = CameraParams {
        position: Vec3::new(0.0, 1.6, 5.0),
        look_at: Vec3::new(0.0, 1.0, 0.0),
        fov: 55.0,
    };
    // Camera/aspect get replaced in `build`; pass a placeholder here.
    let mut scene = Scene::new(Camera::new(
        cam.position,
        cam.look_at,
        Vec3::new(0.0, 1.0, 0.0),
        cam.fov,
        1.0,
    ));

    scene.add(Plane::new(
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        ground_material(),
    ));

    scene.add(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Material::new(Color::new(0.85, 0.15, 0.15))
            .diffuse(0.8)
            .specular(0.5)
            .shininess(48.0)
            .reflectivity(0.35),
    ));

    scene.add_light(Light::white(Vec3::new(5.0, 6.0, -3.0), 1.2));
    scene.add_light(soft_fill_light());

    (scene, cam)
}

/// Scene 2: a flat plane and a cube, deliberately lit less brightly than
/// scene 1 (key intensity 0.5 vs 1.2), while retaining the same soft fill.
fn scene_plane_and_cube() -> (Scene, CameraParams) {
    let cam = CameraParams {
        position: Vec3::new(0.0, 2.2, 6.0),
        look_at: Vec3::new(0.0, 1.0, 0.0),
        fov: 55.0,
    };
    let mut scene = Scene::new(Camera::new(
        cam.position,
        cam.look_at,
        Vec3::new(0.0, 1.0, 0.0),
        cam.fov,
        1.0,
    ));

    scene.add(Plane::new(
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        ground_material(),
    ));

    scene.add(Cube::new(
        Vec3::new(0.0, 0.9, 0.0),
        1.8,
        Material::new(Color::new(0.2, 0.35, 0.85))
            .diffuse(0.8)
            .specular(0.4)
            .shininess(32.0)
            .reflectivity(0.15),
    ));

    scene.add_light(Light::white(Vec3::new(5.0, 6.0, -3.0), 0.5));
    scene.add_light(soft_fill_light());

    (scene, cam)
}

fn default_camera_3() -> CameraParams {
    CameraParams {
        position: Vec3::new(0.0, 3.2, 8.5),
        look_at: Vec3::new(0.0, 1.0, 0.0),
        fov: 50.0,
    }
}

/// Same scene as camera 3, but viewed from a different angle (further
/// requirement: same scene, different perspective).
fn default_camera_4() -> CameraParams {
    CameraParams {
        position: Vec3::new(-6.0, 3.5, -6.5),
        look_at: Vec3::new(0.0, 1.0, 0.0),
        fov: 50.0,
    }
}

/// Scene 3/4: at least one of every required primitive. The extra glass
/// sphere demonstrates the refraction bonus in the same render.
fn scene_all_objects(cam: CameraParams) -> (Scene, CameraParams) {
    let mut scene = Scene::new(Camera::new(
        cam.position,
        cam.look_at,
        Vec3::new(0.0, 1.0, 0.0),
        cam.fov,
        1.0,
    ));

    scene.add(Plane::new(
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        ground_material(),
    ));

    scene.add(Sphere::new(
        Vec3::new(-2.3, 1.0, 0.3),
        1.0,
        Material::new(Color::new(0.85, 0.15, 0.15))
            .diffuse(0.8)
            .specular(0.5)
            .shininess(48.0)
            .reflectivity(0.35),
    ));

    // Checker material makes the texture bonus immediately visible on a
    // finite object as well as on the ground plane when --texture is enabled.
    scene.add(Cube::new(
        Vec3::new(0.0, 0.75, -0.5),
        1.5,
        Material::checker(
            Color::new(0.15, 0.3, 0.85),
            Color::new(0.65, 0.75, 1.0),
            0.4,
        )
        .diffuse(0.8)
        .specular(0.4)
        .shininess(32.0)
        .reflectivity(0.15),
    ));

    scene.add(Cylinder::new(
        Vec3::new(2.4, 0.0, 0.3),
        Vec3::new(0.0, 1.0, 0.0),
        0.8,
        1.6,
        Material::new(Color::new(0.2, 0.75, 0.35))
            .diffuse(0.85)
            .specular(0.3)
            .shininess(24.0),
    ));

    // Glass sphere sitting in front, showcasing the refraction bonus when --refract is set.
    scene.add(Sphere::new(
        Vec3::new(0.2, 0.55, 2.2),
        0.55,
        Material::new(Color::new(0.9, 0.95, 1.0))
            .diffuse(0.05)
            .specular(0.9)
            .shininess(120.0)
            .reflectivity(0.1)
            .transparency(0.9)
            .ior(1.5),
    ));

    scene.add_light(Light::white(Vec3::new(5.0, 7.0, -3.0), 1.1));
    scene.add_light(Light::new(
        Vec3::new(-4.0, 4.0, 4.0),
        Color::new(0.6, 0.7, 1.0),
        0.35,
    ));

    (scene, cam)
}
