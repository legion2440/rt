# RT — CPU Ray Tracer

A CPU ray tracer written in Rust for the 01-edu `rt` assignment. It renders spheres, cubes, planes and cylinders to ASCII PPM (`P3`) images with movable cameras, configurable lighting, cast shadows, procedural textures, reflection and refraction.

· [Русская версия](README_RU.md)

## 📋 TOC

- [🚀 Quick start](#-quick-start)
- [📝 About](#-about)
- [✅ Required features](#-required-features)
- [🖼️ Required demo scenes](#️-required-demo-scenes)
- [🎛️ CLI reference](#️-cli-reference)
- [✨ Bonus features](#-bonus-features)
- [🧱 Creating and changing scenes](#-creating-and-changing-scenes)
- [🧪 Verification](#-verification)
- [📁 Project structure](#-project-structure)
- [⚠️ Notes](#️-notes)
- [🧑‍💻 Authors](#-authors)

## 🚀 Quick start

### Requirements

- Rust 1.63 or newer
- Cargo
- no third-party Rust dependencies

### Build

```bash
cargo build --release
```

### Render a complete scene

```bash
./target/release/rt \
  --scene 3 \
  --reflect \
  --refract \
  --texture \
  --out output.ppm
```

The subject-style stdout workflow is also supported. Rendering logs are written to `stderr`, so they do not corrupt the image stream:

```bash
cargo run --release -- --scene 1 > output.ppm
```

For quick tests, reduce the resolution:

```bash
./target/release/rt --scene 3 --width 200 --height 150 --out preview.ppm
```

## 📝 About

For every output pixel, the camera emits a ray into the scene. The renderer finds the closest intersection among all objects and evaluates the surface with a Phong-style lighting model:

- **ambient** — a small base contribution;
- **diffuse** — Lambertian lighting from visible light sources;
- **specular** — view-dependent highlights;
- **shadows** — a shadow ray is cast from the hit point towards each light and suppresses that light when another object blocks it.

Optional reflection and refraction recursively trace secondary rays up to a fixed depth. Procedural checker textures are evaluated directly from the 3D hit point.

Rows are rendered in parallel with `std::thread::scope`. The final image is written as a plain-text `P3` PPM file with conservative line wrapping.

See [docs/DOCUMENTATION.md](docs/DOCUMENTATION.md) for the detailed API guide and object-construction examples.

## ✅ Required features

| Requirement | Implementation |
| --- | --- |
| Sphere | `src/shapes/sphere.rs` |
| Cube | `src/shapes/cube.rs` |
| Flat plane | `src/shapes/plane.rs` |
| Cylinder | `src/shapes/cylinder.rs` |
| Move objects | Constructor coordinates/centers/axes in `src/scenes.rs` |
| Move/aim camera | `--camera`, `--look-at`, `--fov` |
| Change brightness | Per-light intensity and global `--brightness` |
| Shadows | Shadow rays in `src/renderer.rs` |
| Adjustable resolution | `--width`, `--height` |
| PPM output | ASCII `P3` writer in `src/ppm.rs` |

The camera basis includes fallbacks for top-down/bottom-up views and coincident `position` / `look_at` values, so common camera overrides do not collapse the image plane.

## 🖼️ Required demo scenes

The four audit renders are stored in [`renders/`](renders/) at **800×600**.

| File | Scene |
| --- | --- |
| `scene1_sphere.ppm` | Sphere above a plane, bright key light plus soft fill |
| `scene2_plane_and_cube_lower_brightness.ppm` | Plane + cube, with a lower key-light intensity than scene 1 |
| `scene3_all_objects.ppm` | All required primitives, plus the glass sphere used to demonstrate refraction |
| `scene4_all_objects_alt_camera.ppm` | The same scene as scene 3 from another camera position |

Regenerate all four with:

```bash
cargo build --release

./target/release/rt --scene 1 --width 800 --height 600 \
  --reflect --refract --texture \
  --out renders/scene1_sphere.ppm

./target/release/rt --scene 2 --width 800 --height 600 \
  --reflect --refract --texture \
  --out renders/scene2_plane_and_cube_lower_brightness.ppm

./target/release/rt --scene 3 --width 800 --height 600 \
  --reflect --refract --texture \
  --out renders/scene3_all_objects.ppm

./target/release/rt --scene 4 --width 800 --height 600 \
  --reflect --refract --texture \
  --out renders/scene4_all_objects_alt_camera.ppm
```

Scene 2 keeps the same soft fill as scene 1 but uses a dimmer key light (`0.5` instead of `1.2`), preserving the assignment's lower-brightness comparison while keeping the cube readable from the camera side.

## 🎛️ CLI reference

```text
rt [OPTIONS]

--scene <1|2|3|4>    Demo scene to render (default: 1)
--width <N>          Image width (default: 800)
--height <N>         Image height (default: 600)
--out <FILE>         Write PPM to FILE (default: stdout)
--reflect            Enable reflection bonus
--refract            Enable refraction bonus
--texture            Enable procedural textures
--brightness <F>     Global light multiplier (default: 1.0)
--camera <x,y,z>     Override camera position
--look-at <x,y,z>    Override camera target
--fov <deg>          Override vertical field of view
--threads <N>        Worker threads (default: detected CPU count)
-h, --help           Show help
```

Move the camera without changing code:

```bash
./target/release/rt --scene 3 --out front.ppm
./target/release/rt --scene 3 \
  --camera 6,4,-6 \
  --look-at 0,1,0 \
  --fov 45 \
  --out side.ppm
```

Top-down views are supported as well:

```bash
./target/release/rt --scene 3 \
  --camera 0,10,0 \
  --look-at 0,0,0 \
  --out top.ppm
```

## ✨ Bonus features

### Procedural textures

Enable with:

```bash
--texture
```

`Material::checker(...)` provides a two-color checker pattern. The required scene 3/4 setup demonstrates it on both the ground and the cube.

### Reflection

Enable with:

```bash
--reflect
```

Materials with non-zero `reflectivity` recursively trace mirror rays.

### Refraction

Enable with:

```bash
--refract
```

Transparent materials use Snell's law with an index of refraction (`ior`) and fall back to total internal reflection when refraction is impossible. The additional glass sphere in scenes 3/4 demonstrates this feature.

Particles and fluids are not implemented.

## 🧱 Creating and changing scenes

The four built-in scenes live in [`src/scenes.rs`](src/scenes.rs). For normal audit experiments, edit an existing scene or use CLI camera/brightness overrides.

Typical object construction:

```rust
scene.add(Sphere::new(
    Vec3::new(1.0, 1.0, 1.0),
    1.0,
    Material::new(Color::new(0.85, 0.15, 0.15)),
));

scene.add(Cube::new(
    Vec3::new(0.0, 0.75, 0.0),
    1.5,
    Material::new(Color::new(0.2, 0.35, 0.85)),
));

scene.add(Plane::new(
    Vec3::ZERO,
    Vec3::new(0.0, 1.0, 0.0),
    Material::new(Color::new(0.6, 0.6, 0.6)),
));

scene.add(Cylinder::new(
    Vec3::new(2.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    0.8,
    1.6,
    Material::new(Color::new(0.2, 0.75, 0.35)),
));
```

To add a new numbered preset such as scene 5:

1. copy one of the scene-builder functions in `src/scenes.rs` and change its objects/lights/camera;
2. add `5 => your_scene_function()` to the `match` inside `scenes::build`;
3. extend the `--scene` validation and help text in `src/main.rs` from `1..=4` to include the new preset;
4. rebuild with `cargo build --release`.

For changing only object positions, brightness or the existing camera, no new preset is necessary.

## 🧪 Verification

Run the local checks before evaluation:

```bash
cargo fmt -- --check
cargo test
cargo build --release
```

The unit tests cover the important robustness fixes:

- top-down and degenerate camera basis handling;
- cube rays that start inside the box and must hit the exit face;
- P3 output sample count and line-length wrapping.

A fast functional smoke test:

```bash
./target/release/rt --scene 1 --width 160 --height 120 --out /tmp/scene1.ppm
./target/release/rt --scene 3 --width 160 --height 120 \
  --reflect --refract --texture --out /tmp/scene3.ppm
```

A valid generated file starts with:

```text
P3
<width> <height>
255
```

## 📁 Project structure

```text
rt/
├── docs/
│   └── DOCUMENTATION.md
├── renders/
│   ├── scene1_sphere.ppm
│   ├── scene2_plane_and_cube_lower_brightness.ppm
│   ├── scene3_all_objects.ppm
│   └── scene4_all_objects_alt_camera.ppm
├── src/
│   ├── shapes/
│   │   ├── cube.rs
│   │   ├── cylinder.rs
│   │   ├── plane.rs
│   │   ├── sphere.rs
│   │   └── mod.rs
│   ├── camera.rs
│   ├── hittable.rs
│   ├── light.rs
│   ├── main.rs
│   ├── material.rs
│   ├── ppm.rs
│   ├── ray.rs
│   ├── renderer.rs
│   ├── scene.rs
│   ├── scenes.rs
│   └── vec3.rs
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
└── README_RU.md
```

## ⚠️ Notes

- The renderer intentionally uses one primary ray per pixel; anti-aliasing is outside the assignment scope.
- Reflection/refraction recursion is capped to guarantee termination.
- Bonus effects are opt-in, keeping the default render lightweight.
- Runtime status is printed to `stderr`; image data goes to `stdout` unless `--out` is supplied.
- Root-level `output.ppm` is ignored so temporary renders are not accidentally committed; the four required deliverables remain under `renders/`.

## 🧑‍💻 Authors

- Atabek Furkat [**@abakhram**](https://01.tomorrow-school.ai/intra/astanahub/users/8197)
- Nazar Yestayev [**@nyestaye**](https://01.tomorrow-school.ai/intra/astanahub/users/4468)
- Sultan Yersultan [**@syersult**](https://01.tomorrow-school.ai/intra/astanahub/users/4423)
- Maksat Kapan [**@mkapan**](https://01.tomorrow-school.ai/intra/astanahub/users/3597)
- Daniyar Shadykhanov [**@dshadykh**](https://01.tomorrow-school.ai/intra/astanahub/users/2418)
- Taalaibek Adzhikulov [**@tadzhiku**](https://01.tomorrow-school.ai/intra/astanahub/users/2158)
