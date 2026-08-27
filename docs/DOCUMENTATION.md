# rt — a CPU Ray Tracer

`rt` is a ray tracer written in Rust (no external dependencies) that renders
a 3D scene of spheres, cubes, planes and cylinders to a `.ppm` image. It
supports moving the camera, changing light brightness, casting shadows, and
three bonus effects — procedural textures, mirror reflection, and glass-like
refraction — each gated behind a command-line flag.

## Contents

- [How it works](#how-it-works)
- [Building and running](#building-and-running)
- [Command-line reference](#command-line-reference)
- [The 4 required renders](#the-4-required-renders)
- [Writing your own scene](#writing-your-own-scene)
  - [Creating a sphere](#creating-a-sphere)
  - [Creating a cube](#creating-a-cube)
  - [Creating a plane](#creating-a-plane)
  - [Creating a cylinder](#creating-a-cylinder)
  - [Changing brightness](#changing-brightness)
  - [Moving/angling the camera](#movingangling-the-camera)
- [Bonus features](#bonus-features)
- [Project layout](#project-layout)

## How it works

For every pixel of the output image, the camera fires one ray into the
scene ([src/camera.rs](../src/camera.rs)). `Scene::hit` ([src/scene.rs](../src/scene.rs))
finds the closest object that ray intersects, out of the 4 primitives
implementing the `Hittable` trait ([src/hittable.rs](../src/hittable.rs),
[src/shapes/](../src/shapes/)).

At the hit point, `renderer::trace` ([src/renderer.rs](../src/renderer.rs))
computes the pixel color with a Phong-style lighting model:

- **ambient** — a constant fraction of the surface color, so objects are
  never pure black even with no direct light;
- **diffuse** — brighter the more directly a light shines on the surface
  (`max(0, dot(normal, direction_to_light))`);
- **specular** — a shiny highlight where the reflected light direction lines
  up with the camera;
- **shadows** — before adding a light's diffuse/specular contribution, a
  second ray ("shadow ray") is cast from the hit point towards that light;
  if it hits another object first, the light is occluded and skipped for
  that pixel, which is what produces cast shadows.

If `--reflect`/`--refract` are enabled and the material has non-zero
`reflectivity`/`transparency`, `trace` additionally casts a reflected and/or
refracted (Snell's law) ray recursively (up to a depth limit) and blends
that result in, which is what produces mirror and glass effects.

Finally, the whole grid of pixel colors is written out as an ASCII `.ppm`
(P3) file ([src/ppm.rs](../src/ppm.rs)).

Rendering is parallelized: rows are split evenly across worker threads
(`std::thread::scope`, `--threads`, defaults to the number of CPUs), so an
800x600 image with all bonuses on renders in well under a second on a
modern machine.

## Building and running

```sh
cargo build --release
./target/release/rt --scene 1 --out output.ppm
```

Or, following the subject's suggested invocation, pipe stdout to a file
(omit `--out` and the image is written to stdout, the log lines go to
stderr so they don't corrupt the image):

```sh
cargo run --release -- --scene 1 > output.ppm
```

To view a `.ppm`, most image viewers/editors that support the format work
(e.g. GIMP), or convert it, e.g. with Python/Pillow:

```sh
python3 -c "from PIL import Image; Image.open('output.ppm').save('output.png')"
```

**Reduce the resolution while testing** — this is the single biggest lever
on render time:

```sh
./target/release/rt --scene 3 --width 200 --height 150 --reflect --refract --out preview.ppm
```

## Command-line reference

```
rt [OPTIONS]

--scene <1|2|3|4>    Which built-in demo scene to render (default: 1)
--width <N>          Image width in pixels (default: 800)
--height <N>         Image height in pixels (default: 600)
--out <FILE>         Write the PPM image to FILE (default: stdout)
--reflect            Enable the reflection bonus
--refract            Enable the refraction bonus
--texture            Enable the procedural checkerboard texture bonus
--brightness <F>     Global light brightness multiplier (default: 1.0)
--camera <x,y,z>     Override the camera position
--look-at <x,y,z>    Override the point the camera looks at
--fov <deg>          Override the camera's vertical field of view
--threads <N>        Render worker threads (default: number of CPUs)
-h, --help           Print usage and exit
```

`--camera`/`--look-at`/`--fov` let you re-frame any of the 4 built-in scenes
from the command line, without touching any code — this is the fastest way
to satisfy "look at the same scene from a different angle":

```sh
./target/release/rt --scene 3 --out a.ppm
./target/release/rt --scene 3 --camera 6,4,-6 --look-at 0,1,0 --fov 45 --out b.ppm
```

## The 4 required renders

Pre-rendered at 800x600 in [renders/](../renders/) (all with `--reflect --refract --texture`):

| File | Scenario |
| --- | --- |
| [scene1_sphere.ppm](../renders/scene1_sphere.ppm) | A sphere |
| [scene2_plane_and_cube_lower_brightness.ppm](../renders/scene2_plane_and_cube_lower_brightness.ppm) | A flat plane + a cube, dimmer light than scene 1 (intensity 0.5 vs 1.2) |
| [scene3_all_objects.ppm](../renders/scene3_all_objects.ppm) | One sphere, one cube, one cylinder, one plane |
| [scene4_all_objects_alt_camera.ppm](../renders/scene4_all_objects_alt_camera.ppm) | Same scene as above, camera moved to another position |

Regenerate them with:

```sh
cargo build --release
./target/release/rt --scene 1 --width 800 --height 600 --reflect --refract --texture --out renders/scene1_sphere.ppm
./target/release/rt --scene 2 --width 800 --height 600 --reflect --refract --texture --out renders/scene2_plane_and_cube_lower_brightness.ppm
./target/release/rt --scene 3 --width 800 --height 600 --reflect --refract --texture --out renders/scene3_all_objects.ppm
./target/release/rt --scene 4 --width 800 --height 600 --reflect --refract --texture --out renders/scene4_all_objects_alt_camera.ppm
```

## Writing your own scene

Scenes are plain Rust, built with a small fluent API — see
[src/scenes.rs](../src/scenes.rs) for the 4 built-in scenes as worked
examples. A scene is a `Scene`, which owns a `Camera`, a list of objects
(anything implementing `Hittable`) and a list of `Light`s:

```rust
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::scene::Scene;
use crate::shapes::{Cube, Cylinder, Plane, Sphere};
use crate::vec3::{Color, Vec3};

let camera = Camera::new(
    Vec3::new(0.0, 2.0, 6.0),   // position (the eye)
    Vec3::new(0.0, 1.0, 0.0),   // look_at (aim point)
    Vec3::new(0.0, 1.0, 0.0),   // world up
    55.0,                       // vertical field of view, in degrees
    800.0 / 600.0,              // aspect ratio (width / height)
);
let mut scene = Scene::new(camera);
```

### Creating a sphere

```rust
scene.add(Sphere::new(
    Vec3::new(1.0, 1.0, 1.0),                 // center — e.g. (1,1,1) as in the subject example
    1.0,                                       // radius
    Material::new(Color::new(0.85, 0.15, 0.15)), // base color (red)
));
```

### Creating a cube

```rust
scene.add(Cube::new(
    Vec3::new(0.0, 0.75, 0.0), // center
    1.5,                        // edge length
    Material::new(Color::new(0.2, 0.35, 0.85)), // base color (blue)
));
```

(`Cube::from_bounds(min, max, material)` is also available if you want an
axis-aligned box that isn't a perfect cube.)

### Creating a plane

```rust
scene.add(Plane::new(
    Vec3::ZERO,                 // any point on the plane
    Vec3::new(0.0, 1.0, 0.0),   // normal (which way it faces) — (0,1,0) = horizontal ground
    Material::new(Color::new(0.6, 0.6, 0.6)),
));
```

### Creating a cylinder

```rust
scene.add(Cylinder::new(
    Vec3::new(2.0, 0.0, 0.0),   // center of the base
    Vec3::new(0.0, 1.0, 0.0),   // axis direction (which way it points, doesn't need to be vertical)
    0.8,                         // radius
    1.6,                         // height
    Material::new(Color::new(0.2, 0.75, 0.35)),
));
```

Every `Material` also has chainable setters (used together with textures/
reflection/refraction, see [Bonus features](#bonus-features)):

```rust
Material::new(Color::new(0.85, 0.15, 0.15))
    .ambient(0.1)       // fraction always visible, even unlit, 0..1
    .diffuse(0.8)        // matte brightness response to light, 0..1
    .specular(0.5)       // shiny highlight strength, 0..1
    .shininess(48.0)     // highlight tightness (higher = smaller/sharper)
    .reflectivity(0.35)  // mirror strength, 0..1 (needs --reflect)
    .transparency(0.9)   // glass strength, 0..1 (needs --refract)
    .ior(1.5);            // index of refraction (needs --refract)
```

### Changing brightness

Every `Light` carries its own brightness (`intensity`), independent of the
others — this is how scene 2 is dimmer than scene 1 in the required renders:

```rust
// A bright light:
scene.add_light(Light::white(Vec3::new(5.0, 6.0, -3.0), 1.2));

// A dim, tinted light:
scene.add_light(Light::new(Vec3::new(-4.0, 4.0, 4.0), Color::new(0.6, 0.7, 1.0), 0.35));
```

You can also scale every light in a scene at once without touching scene
code, using `--brightness` (e.g. `--brightness 0.5` halves all light
intensities):

```sh
./target/release/rt --scene 1 --brightness 0.4 --out dim.ppm
```

### Moving/angling the camera

In code, just change the arguments to `Camera::new` (position, look-at
point, and field of view — see [Creating a sphere](#creating-a-sphere)'s
preamble above). From the command line, without recompiling:

```sh
./target/release/rt --scene 1 --camera 0,1,-5 --look-at 0,1,0 --fov 70 --out side.ppm
```

## Bonus features

All three are opt-in via CLI flags, per the subject's suggestion, so the
default render stays fast:

- **`--texture`** — procedural checkerboard pattern (`Material::checker`),
  evaluated directly from the 3D hit point so it doesn't need UV mapping or
  image assets. See the checkered ground plane in [renders/scene3_all_objects.ppm](../renders/scene3_all_objects.ppm).
- **`--reflect`** — mirror reflection: for a material with `reflectivity >
  0`, the pixel color blends in a recursively-traced reflected ray. See the
  red sphere in the scene 3/4 renders.
- **`--refract`** — refraction (Snell's law) plus total-internal-reflection
  fallback: for a material with `transparency > 0`, light bends through the
  object based on its `ior`. See the small glass sphere in the scene 3/4
  renders.

Both `--reflect` and `--refract` recurse up to a fixed depth (6 bounces) to
guarantee termination.

Particles and fluids are not implemented.

## Project layout

```
src/
  main.rs       CLI parsing, threaded render loop, entry point
  vec3.rs       Vec3 math (also used as Color)
  ray.rs        Ray
  camera.rs     Camera (position/look-at/fov -> per-pixel rays)
  material.rs   Material + Texture (solid / checkerboard)
  light.rs      Point light (position, color, intensity)
  hittable.rs   Hittable trait + HitRecord
  shapes/       Sphere, Cube, Plane, Cylinder (each implements Hittable)
  scene.rs      Scene: objects + lights + camera + background
  scenes.rs     The 4 required demo scenes (worked examples)
  renderer.rs   trace(): shading, shadows, reflection, refraction
  ppm.rs        P3 PPM writer
renders/        The 4 required 800x600 .ppm deliverables
docs/           This file
```
