mod camera;
mod hittable;
mod light;
mod material;
mod ppm;
mod ray;
mod renderer;
mod scene;
mod scenes;
mod shapes;
mod vec3;

use renderer::RenderOptions;
use scene::Scene;
use scenes::CameraParams;
use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;
use vec3::{Color, Vec3};

struct Config {
    scene: u32,
    width: usize,
    height: usize,
    out: Option<String>,
    reflect: bool,
    refract: bool,
    texture: bool,
    brightness: f64,
    threads: usize,
    camera: Option<Vec3>,
    look_at: Option<Vec3>,
    fov: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scene: 1,
            width: 800,
            height: 600,
            out: None,
            reflect: false,
            refract: false,
            texture: false,
            brightness: 1.0,
            threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
            camera: None,
            look_at: None,
            fov: None,
        }
    }
}

fn parse_vec3(s: &str) -> Result<Vec3, String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("expected \"x,y,z\", got \"{s}\""));
    }
    let mut v = [0.0f64; 3];
    for (i, p) in parts.iter().enumerate() {
        v[i] = p.trim().parse().map_err(|_| format!("invalid number \"{p}\" in \"{s}\""))?;
    }
    Ok(Vec3::new(v[0], v[1], v[2]))
}

fn print_usage() {
    eprintln!(
        r#"rt - a CPU ray tracer

USAGE:
    rt [OPTIONS]

OPTIONS:
    --scene <1|2|3|4>       Which demo scene to render (default: 1)
    --width <N>             Image width in pixels (default: 800)
    --height <N>            Image height in pixels (default: 600)
    --out <FILE>            Write the PPM image to FILE (default: stdout)
    --reflect               Enable the reflection bonus
    --refract               Enable the refraction bonus
    --texture               Enable the procedural texture bonus
    --brightness <F>        Global light brightness multiplier (default: 1.0)
    --camera <x,y,z>        Override the camera position
    --look-at <x,y,z>       Override the point the camera looks at
    --fov <deg>             Override the camera's vertical field of view
    --threads <N>           Render worker threads (default: number of CPUs)
    -h, --help              Print this help and exit

EXAMPLES:
    rt --scene 1 --out renders/scene1_sphere.ppm
    rt --scene 3 --reflect --refract --texture --out renders/all_objects.ppm
    rt --scene 3 --camera 6,4,6 --look-at 0,1,0 --out renders/other_angle.ppm
    rt --scene 1 --width 320 --height 240 > preview.ppm

See docs/DOCUMENTATION.md for the full guide (creating objects, lights, moving the camera).
"#
    );
}

fn parse_args() -> Result<Option<Config>, String> {
    let mut cfg = Config::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        macro_rules! next_val {
            () => {
                args.next().ok_or_else(|| format!("missing value for {arg}"))?
            };
        }
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--scene" => cfg.scene = next_val!().parse().map_err(|_| "invalid --scene")?,
            "--width" => cfg.width = next_val!().parse().map_err(|_| "invalid --width")?,
            "--height" => cfg.height = next_val!().parse().map_err(|_| "invalid --height")?,
            "--out" => cfg.out = Some(next_val!()),
            "--reflect" => cfg.reflect = true,
            "--refract" => cfg.refract = true,
            "--texture" => cfg.texture = true,
            "--brightness" => cfg.brightness = next_val!().parse().map_err(|_| "invalid --brightness")?,
            "--camera" => cfg.camera = Some(parse_vec3(&next_val!())?),
            "--look-at" => cfg.look_at = Some(parse_vec3(&next_val!())?),
            "--fov" => cfg.fov = Some(next_val!().parse().map_err(|_| "invalid --fov")?),
            "--threads" => cfg.threads = next_val!().parse().map_err(|_| "invalid --threads")?,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if !(1..=4).contains(&cfg.scene) {
        return Err(format!("--scene must be 1, 2, 3 or 4 (got {})", cfg.scene));
    }
    if cfg.width == 0 || cfg.height == 0 {
        return Err("--width/--height must be > 0".to_string());
    }
    if cfg.threads == 0 {
        return Err("--threads must be > 0".to_string());
    }

    Ok(Some(cfg))
}

/// Render the whole image, splitting rows across `threads` worker threads.
fn render(scene: &Scene, opts: &RenderOptions, width: usize, height: usize, threads: usize) -> Vec<Color> {
    let mut pixels = vec![Color::black(); width * height];
    let chunk_rows = height.div_ceil(threads).max(1);

    std::thread::scope(|s| {
        for (chunk_idx, chunk) in pixels.chunks_mut(width * chunk_rows).enumerate() {
            let row_start = chunk_idx * chunk_rows;
            s.spawn(move || {
                for (local_row, row_pixels) in chunk.chunks_mut(width).enumerate() {
                    let row = row_start + local_row;
                    // Image row 0 is the top of the image, but our camera's `t`
                    // parameter is bottom-up, so flip vertically here.
                    let t = 1.0 - (row as f64 + 0.5) / height as f64;
                    for (col, pixel) in row_pixels.iter_mut().enumerate() {
                        let u = (col as f64 + 0.5) / width as f64;
                        let ray = scene.camera.get_ray(u, t);
                        *pixel = renderer::trace(scene, &ray, opts, opts.max_depth).clamp(0.0, 1.0);
                    }
                }
            });
        }
    });

    pixels
}

fn run() -> Result<(), String> {
    let cfg = match parse_args()? {
        Some(cfg) => cfg,
        None => {
            print_usage();
            return Ok(());
        }
    };

    let aspect_ratio = cfg.width as f64 / cfg.height as f64;
    let camera_override = if cfg.camera.is_some() || cfg.look_at.is_some() || cfg.fov.is_some() {
        // Reasonable defaults for whichever of position/look_at/fov wasn't overridden.
        Some(CameraParams {
            position: cfg.camera.unwrap_or(Vec3::new(0.0, 2.0, 6.0)),
            look_at: cfg.look_at.unwrap_or(Vec3::new(0.0, 1.0, 0.0)),
            fov: cfg.fov.unwrap_or(55.0),
        })
    } else {
        None
    };

    let scene = scenes::build(cfg.scene, aspect_ratio, camera_override);

    let opts = RenderOptions {
        reflections: cfg.reflect,
        refractions: cfg.refract,
        textures: cfg.texture,
        max_depth: 6,
        brightness: cfg.brightness,
    };

    eprintln!(
        "rt: rendering scene {} at {}x{} (reflect={} refract={} texture={} threads={})...",
        cfg.scene, cfg.width, cfg.height, cfg.reflect, cfg.refract, cfg.texture, cfg.threads
    );
    let start = Instant::now();
    let pixels = render(&scene, &opts, cfg.width, cfg.height, cfg.threads);
    eprintln!("rt: done in {:.2?}", start.elapsed());

    match cfg.out {
        Some(path) => {
            let mut file = File::create(&path).map_err(|e| format!("cannot create {path}: {e}"))?;
            ppm::write_ppm(&mut file, cfg.width, cfg.height, &pixels).map_err(|e| e.to_string())?;
            eprintln!("rt: wrote {path}");
        }
        None => {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            ppm::write_ppm(&mut lock, cfg.width, cfg.height, &pixels).map_err(|e| e.to_string())?;
            lock.flush().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rt: error: {e}");
            ExitCode::FAILURE
        }
    }
}
