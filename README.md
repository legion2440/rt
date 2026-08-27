# rt

A CPU ray tracer (Rust, no external dependencies) rendering spheres, cubes,
planes and cylinders to `.ppm` images, with shadows, adjustable brightness,
a movable camera, and bonus textures/reflection/refraction.

```sh
cargo build --release
./target/release/rt --scene 3 --reflect --refract --texture --out output.ppm
```

See **[docs/DOCUMENTATION.md](docs/DOCUMENTATION.md)** for the full guide:
how the ray tracer works, the command-line reference, and code examples for
creating each object, changing brightness, and moving the camera.

The 4 required demo renders (800x600) are in [renders/](renders/).
