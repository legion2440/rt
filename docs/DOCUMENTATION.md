# RT Documentation

This document explains how the ray tracer works, how to run it, and how to create or modify scenes. The shorter project overview is in [`README.md`](../README.md).

## Contents

- [How rendering works](#how-rendering-works)
- [Build and run](#build-and-run)
- [CLI reference](#cli-reference)
- [Scene model](#scene-model)
- [Creating objects](#creating-objects)
- [Materials and textures](#materials-and-textures)
- [Lights and brightness](#lights-and-brightness)
- [Moving and aiming the camera](#moving-and-aiming-the-camera)
- [Particles](#particles)
- [Fluids](#fluids)
- [Adding a custom scene preset](#adding-a-custom-scene-preset)
- [Required renders](#required-renders)
- [PPM output](#ppm-output)
- [Bonus effects](#bonus-effects)
- [Project layout](#project-layout)

## How rendering works

For every output pixel, `Camera::get_ray` creates a ray starting at the camera position and passing through the image plane. `Scene::hit` asks every object implementing `Hittable` for an intersection and keeps the closest hit.

The required primitives are implemented independently:

- `Sphere` — quadratic ray/sphere intersection;
- `Cube` — axis-aligned slab intersection, including rays that start inside the box;
- `Plane` — ray/plane equation;
- `Cylinder` — finite side surface plus top and bottom caps.

Bonus geometry uses the same `Hittable` interface:

- `ParticleCloud` — a deterministic collection of small spherical particles presented as one scene object;
- `FluidSurface` — a finite sinusoidal height field intersected inside its bounding box.

At the closest hit point, `renderer::trace` evaluates a Phong-style lighting model:

- **ambient** keeps a small amount of the base color visible;
- **diffuse** uses `max(0, dot(normal, light_direction))`;
- **specular** adds a view-dependent highlight;
- **shadows** cast a second ray from the hit point to each light. If another object is hit before the light, that light contributes no diffuse/specular term.

When enabled, reflection and refraction recursively trace secondary rays. Recursion is capped by `RenderOptions::max_depth` to guarantee termination.

The final linear colors are clamped, gamma-corrected with gamma 2.0 (`sqrt`) and written as an ASCII PPM (`P3`) image.

## Build and run

### Requirements

- Rust 1.63 or newer
- Cargo
- no external crates

Build an optimized binary:

```bash
cargo build --release
```

Render scene 1 to a file:

```bash
./target/release/rt --scene 1 --out output.ppm
```

Render the isolated bonus scene:

```bash
./target/release/rt --scene 5 --reflect --refract --texture --out bonus.ppm
```

The subject-style stdout form is also supported:

```bash
cargo run --release -- --scene 1 > output.ppm
```

All progress/status lines are printed to `stderr`, so redirected `stdout` contains only PPM image data.

For fast iteration, lower the resolution:

```bash
./target/release/rt --scene 3 --width 200 --height 150 --out preview.ppm
```

## CLI reference

```text
rt [OPTIONS]

--scene <1|2|3|4|5>  Built-in scene; 5 is the bonus showcase (default: 1)
--width <N>          Image width in pixels (default: 800)
--height <N>         Image height in pixels (default: 600)
--out <FILE>         Write PPM to FILE (default: stdout)
--reflect            Enable reflection
--refract            Enable refraction
--texture            Enable procedural textures
--brightness <F>     Multiply all light intensities (default: 1.0)
--camera <x,y,z>     Override camera position
--look-at <x,y,z>    Override camera target
--fov <deg>          Override vertical field of view
--threads <N>        Render worker threads (default: detected CPU count)
-h, --help           Show help
```

Required-feature example:

```bash
./target/release/rt --scene 3 \
  --reflect --refract --texture \
  --out scene3.ppm
```

All bonus categories together:

```bash
./target/release/rt --scene 5 \
  --reflect --refract --texture \
  --out bonus.ppm
```

## Scene model

A `Scene` owns:

- a `Camera`;
- `Vec<Box<dyn Hittable>>` objects;
- a list of point `Light`s;
- a background color.

A minimal scene starts like this:

```rust
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::scene::Scene;
use crate::shapes::{
    Cube, Cylinder, FluidSurface, ParticleCloud, Plane, Sphere,
};
use crate::vec3::{Color, Vec3};

let camera = Camera::new(
    Vec3::new(0.0, 2.0, 6.0), // position
    Vec3::new(0.0, 1.0, 0.0), // look_at
    Vec3::new(0.0, 1.0, 0.0), // world up
    55.0,                     // vertical FOV, degrees
    800.0 / 600.0,            // aspect ratio
);

let mut scene = Scene::new(camera);
```

Add objects with `scene.add(...)` and lights with `scene.add_light(...)`.

## Creating objects

### Sphere

```rust
scene.add(Sphere::new(
    Vec3::new(1.0, 1.0, 1.0),
    1.0,
    Material::new(Color::new(0.85, 0.15, 0.15)),
));
```

Arguments: center, radius, material.

### Cube

```rust
scene.add(Cube::new(
    Vec3::new(0.0, 0.75, 0.0),
    1.5,
    Material::new(Color::new(0.2, 0.35, 0.85)),
));
```

Arguments: center, edge length, material.

`Cube::from_bounds(min, max, material)` is also available for an arbitrary axis-aligned box.

The slab intersection keeps both entering and exiting faces, so a ray originating inside the cube correctly hits the exit surface and receives the correct face normal.

### Plane

```rust
scene.add(Plane::new(
    Vec3::ZERO,
    Vec3::new(0.0, 1.0, 0.0),
    Material::new(Color::new(0.6, 0.6, 0.6)),
));
```

Arguments: any point on the plane, surface normal, material.

### Cylinder

```rust
scene.add(Cylinder::new(
    Vec3::new(2.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    0.8,
    1.6,
    Material::new(Color::new(0.2, 0.75, 0.35)),
));
```

Arguments: center of the base, axis direction, radius, height, material. The cylinder is finite and includes both end caps.

## Materials and textures

A solid material starts with a base color:

```rust
Material::new(Color::new(0.85, 0.15, 0.15))
    .ambient(0.1)
    .diffuse(0.8)
    .specular(0.5)
    .shininess(48.0)
    .reflectivity(0.35)
    .transparency(0.0)
    .ior(1.5);
```

Relevant properties:

- `ambient` — constant visible fraction;
- `diffuse` — matte response to direct light;
- `specular` — highlight strength;
- `shininess` — highlight sharpness;
- `reflectivity` — mirror blend when `--reflect` is enabled;
- `transparency` — refraction blend when `--refract` is enabled;
- `ior` — index of refraction.

Checker texture:

```rust
Material::checker(
    Color::new(0.15, 0.30, 0.85),
    Color::new(0.65, 0.75, 1.00),
    0.4,
)
```

The pattern is evaluated from the 3D hit point and therefore works with any primitive using that material. It is visible only when `--texture` is enabled; otherwise the first checker color is used as a flat color.

Scenes 3 and 4 demonstrate the texture bonus on both the ground plane and the cube.

## Lights and brightness

Each `Light` has a position, color and intensity:

```rust
scene.add_light(Light::white(
    Vec3::new(5.0, 6.0, -3.0),
    1.2,
));
```

Tinted light:

```rust
scene.add_light(Light::new(
    Vec3::new(-3.0, 3.0, 5.0),
    Color::new(0.9, 0.9, 1.0),
    0.3,
));
```

Scenes 1 and 2 use a soft camera-side fill light to keep front-facing surfaces readable. Scene 2 still has lower brightness because its key light is `0.5` versus `1.2` in scene 1.

Scale every light at runtime with:

```bash
./target/release/rt --scene 1 --brightness 0.5 --out dim.ppm
```

## Moving and aiming the camera

In code:

```rust
let camera = Camera::new(
    Vec3::new(6.0, 4.0, -6.0), // position
    Vec3::new(0.0, 1.0, 0.0),  // target
    Vec3::new(0.0, 1.0, 0.0),  // up
    45.0,                       // FOV
    aspect_ratio,
);
```

From the CLI:

```bash
./target/release/rt --scene 3 \
  --camera 6,4,-6 \
  --look-at 0,1,0 \
  --fov 45 \
  --out moved.ppm
```

Top-down views are safe:

```bash
./target/release/rt --scene 3 \
  --camera 0,10,0 \
  --look-at 0,0,0 \
  --out top.ppm
```

When the requested up vector is parallel to the view direction, `Camera::new` chooses a non-parallel fallback up vector. If `position == look_at`, it falls back to the conventional +Z camera axis instead of creating a zero-sized image plane.

## Particles

`ParticleCloud` is implemented in `src/shapes/particle.rs`. Internally it contains small spherical particles, but the renderer sees one `Hittable` object.

Explicit cloud:

```rust
let centers = vec![
    Vec3::new(-1.0, 1.0, 0.0),
    Vec3::new(-0.8, 1.4, 0.1),
    Vec3::new(-1.2, 1.8, -0.1),
];

scene.add(ParticleCloud::new(
    centers,
    0.07,
    Material::new(Color::new(1.0, 0.45, 0.12)),
));
```

Deterministic fountain helper:

```rust
scene.add(ParticleCloud::fountain(
    Vec3::new(-2.0, 0.1, 0.0),
    72,
    Material::new(Color::new(1.0, 0.45, 0.12)),
));
```

The fountain uses deterministic golden-angle placement, so repeated renders are stable and no random-number crate is required.

## Fluids

`FluidSurface` is implemented in `src/shapes/fluid.rs`. It models a finite wavy height field:

```text
y = base + amplitude * sin(f*x) * cos(f*z)
```

Create a water-like patch:

```rust
let water = Material::new(Color::new(0.08, 0.38, 0.82))
    .diffuse(0.45)
    .specular(0.9)
    .shininess(120.0)
    .reflectivity(0.3)
    .transparency(0.35)
    .ior(1.333);

scene.add(FluidSurface::new(
    Vec3::new(1.35, 0.48, 0.0), // center / base height
    1.75,                       // half width in X
    1.55,                       // half width in Z
    0.18,                       // wave amplitude
    2.7,                        // wave frequency
    water,
));
```

The implementation first restricts each ray to the fluid's finite bounding box, samples for a sign change of the implicit surface equation, refines the crossing with bisection, and computes the surface normal from the analytic gradient. With `--reflect --refract`, the water material also participates in recursive optical effects.

## Adding a custom scene preset

The built-in presets live in `src/scenes.rs` and are selected by `scenes::build`. Scenes 1-4 are required; scene 5 is the bonus showcase.

To create scene 6:

1. copy an existing scene-builder function such as `scene_all_objects` and edit its objects, lights and camera;
2. add a new match arm in `scenes::build`, for example `6 => scene_custom()`;
3. update the `--scene` range check in `src/main.rs` so it accepts 6;
4. update the help text from `<1|2|3|4|5>` to include 6;
5. run `cargo fmt` and rebuild.

If the goal is only to move the existing camera or change overall brightness, use CLI overrides instead of creating another preset.

## Required renders

The repository contains the four 800×600 deliverables in `renders/`:

```text
renders/
├── scene1_sphere.ppm
├── scene2_plane_and_cube_lower_brightness.ppm
├── scene3_all_objects.ppm
└── scene4_all_objects_alt_camera.ppm
```

Regenerate them with:

```bash
cargo build --release

./target/release/rt --scene 1 --width 800 --height 600 --reflect --refract --texture --out renders/scene1_sphere.ppm
./target/release/rt --scene 2 --width 800 --height 600 --reflect --refract --texture --out renders/scene2_plane_and_cube_lower_brightness.ppm
./target/release/rt --scene 3 --width 800 --height 600 --reflect --refract --texture --out renders/scene3_all_objects.ppm
./target/release/rt --scene 4 --width 800 --height 600 --reflect --refract --texture --out renders/scene4_all_objects_alt_camera.ppm
```

Scenes 3 and 4 intentionally use the same object/light setup and differ only in the default camera position.

The bonus scene is intentionally not committed as a fifth required image. Generate it locally when needed:

```bash
./target/release/rt --scene 5 --width 800 --height 600 --reflect --refract --texture --out bonus.ppm
```

## PPM output

`src/ppm.rs` writes the plain ASCII `P3` format:

```text
P3
800 600
255
...
```

The image body contains RGB integer samples in row-major order after gamma correction. P3 treats whitespace uniformly, so line breaks do not represent image rows. The writer keeps complete RGB triplets together and wraps text lines to at most 70 characters for broad Netpbm compatibility.

## Bonus effects

The official audit asks four bonus questions; all are implemented:

### Textures

Enable with `--texture`. Procedural checker materials can be attached to any primitive. Scenes 3/4 visibly use a checker material on the cube as well as the ground.

### Reflection and refraction

Enable with `--reflect` and `--refract`. Reflective materials cast recursive mirror rays. Transparent materials use Snell's law and `ior`; total internal reflection is used when refraction is impossible.

Scenes 3/4 demonstrate a glass sphere. Scene 5 gives the fluid surface water-like reflection/refraction parameters.

### Particles

`ParticleCloud` provides a deterministic particle system. Scene 5 renders a 72-particle fountain.

### Fluids

`FluidSurface` provides a finite procedural wavy surface with analytic normals. Scene 5 renders it as a water-like material.

## Project layout

```text
src/
  main.rs       CLI parsing, threaded render loop, entry point
  vec3.rs       vector and color math
  ray.rs        ray representation
  camera.rs     camera basis and per-pixel rays
  material.rs   material properties and checker textures
  light.rs      point lights
  hittable.rs   Hittable trait and HitRecord
  shapes/
    sphere.rs   sphere intersection
    cube.rs     box/slab intersection
    plane.rs    infinite plane intersection
    cylinder.rs finite cylinder intersection
    particle.rs particle-cloud bonus
    fluid.rs    procedural fluid-surface bonus
  scene.rs      scene objects, lights, camera and closest-hit search
  scenes.rs     four required scenes + one bonus showcase scene
  renderer.rs   lighting, shadows, reflection and refraction
  ppm.rs        P3 PPM writer
renders/        four required 800×600 images
docs/           documentation
```

Before evaluation, run:

```bash
cargo fmt -- --check
cargo test
cargo build --release
```
