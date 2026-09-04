# RT — CPU Ray Tracer

CPU ray tracer на Rust для задания 01-edu `rt`. Проект рендерит сферы, кубы, плоскости и цилиндры в ASCII PPM (`P3`), поддерживает перемещение камеры, настройку света, тени, процедурные текстуры, отражение, преломление, частицы и процедурную поверхность жидкости.

· [English version](README.md)

## 📋 Содержание

- [🚀 Быстрый старт](#-быстрый-старт)
- [📝 О проекте](#-о-проекте)
- [✅ Обязательные возможности](#-обязательные-возможности)
- [🖼️ Обязательные demo-сцены](#️-обязательные-demo-сцены)
- [🎛️ CLI](#️-cli)
- [✨ Бонусы](#-бонусы)
- [🧱 Создание и изменение сцен](#-создание-и-изменение-сцен)
- [🧪 Проверка](#-проверка)
- [📁 Структура проекта](#-структура-проекта)
- [⚠️ Примечания](#️-примечания)
- [🧑‍💻 Авторы](#-авторы)

## 🚀 Быстрый старт

### Требования

- Rust 1.63 или новее
- Cargo
- сторонние Rust-зависимости не используются

### Сборка

```bash
cargo build --release
```

### Обязательная сцена

```bash
./target/release/rt \
  --scene 3 \
  --reflect \
  --refract \
  --texture \
  --out output.ppm
```

### Демонстрация всех бонусов

```bash
./target/release/rt \
  --scene 5 \
  --reflect \
  --refract \
  --texture \
  --out bonus.ppm
```

Scene 5 содержит детерминированный фонтан частиц и ограниченную волнистую поверхность жидкости. Reflection/refraction делают жидкость похожей на воду, а частицы остаются обычной ray-traced геометрией.

Поддерживается и вариант из subject с выводом изображения через `stdout`. Служебные сообщения идут в `stderr`, поэтому PPM не портится:

```bash
cargo run --release -- --scene 1 > output.ppm
```

Для быстрых проверок уменьши разрешение:

```bash
./target/release/rt --scene 3 --width 200 --height 150 --out preview.ppm
```

## 📝 О проекте

Для каждого пикселя камера выпускает луч в сцену. Рендерер ищет ближайшее пересечение и рассчитывает поверхность по Phong-подобной модели:

- **ambient** — базовая составляющая;
- **diffuse** — ламбертовское освещение;
- **specular** — блики;
- **shadows** — shadow ray к каждому источнику света.

Reflection и refraction рекурсивно трассируют вторичные лучи с ограниченной глубиной. Checker texture вычисляется по 3D-координатам точки пересечения.

Бонусная геометрия использует тот же `Hittable`, что и обязательные примитивы:

- `ParticleCloud` хранит набор маленьких сферических частиц как один scene object;
- `FluidSurface` трассирует ограниченный синусоидальный height field и вычисляет нормали из аналитического градиента.

Строки изображения рендерятся параллельно через `std::thread::scope`. Результат записывается в текстовый PPM `P3` с переносом строк не длиннее 70 символов.

Подробный API: [docs/DOCUMENTATION.md](docs/DOCUMENTATION.md).

## ✅ Обязательные возможности

| Требование | Реализация |
| --- | --- |
| Сфера | `src/shapes/sphere.rs` |
| Куб | `src/shapes/cube.rs` |
| Плоскость | `src/shapes/plane.rs` |
| Цилиндр | `src/shapes/cylinder.rs` |
| Перемещение объектов | Координаты/центры/оси в `src/scenes.rs` |
| Камера | `--camera`, `--look-at`, `--fov` |
| Яркость | Интенсивность света + `--brightness` |
| Тени | Shadow rays в `src/renderer.rs` |
| Разрешение | `--width`, `--height` |
| PPM | ASCII `P3` writer в `src/ppm.rs` |

Камера имеет fallback для top-down/bottom-up ракурсов и случая `position == look_at`.

## 🖼️ Обязательные demo-сцены

Четыре файла для аудита лежат в [`renders/`](renders/) и имеют разрешение **800×600**.

| Файл | Сцена |
| --- | --- |
| `scene1_sphere.ppm` | Красная сфера над плоскостью, bright key + soft fill |
| `scene2_plane_and_cube_lower_brightness.ppm` | Синий куб + плоскость, key light слабее scene 1 |
| `scene3_all_objects.ppm` | Все обязательные примитивы, checker-куб и стеклянная сфера |
| `scene4_all_objects_alt_camera.ppm` | Та же сцена, что scene 3, с другой камеры |

Перегенерация:

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

У обязательной красной сферы и синего куба reflectivity специально уменьшена, чтобы их собственный цвет визуально доминировал. Более сильные reflection/refraction демонстрируются в scenes 3 и 5.

## 🎛️ CLI

```text
rt [OPTIONS]

--scene <1|2|3|4|5>  Demo-сцена; 5 — bonus showcase (default: 1)
--width <N>          Ширина (default: 800)
--height <N>         Высота (default: 600)
--out <FILE>         PPM-файл (default: stdout)
--reflect            Включить отражение
--refract            Включить преломление
--texture            Включить процедурные текстуры
--brightness <F>     Общий множитель яркости
--camera <x,y,z>     Позиция камеры
--look-at <x,y,z>    Точка взгляда
--fov <deg>          Вертикальный FOV
--threads <N>        Worker threads
-h, --help           Help
```

Другой ракурс без изменения кода:

```bash
./target/release/rt --scene 3 \
  --camera 6,4,-6 \
  --look-at 0,1,0 \
  --fov 45 \
  --out side.ppm
```

Top-down:

```bash
./target/release/rt --scene 3 \
  --camera 0,10,0 \
  --look-at 0,0,0 \
  --out top.ppm
```

## ✨ Бонусы

Все четыре бонусных вопроса официального audit реализованы.

| Бонус | Реализация | Демонстрация |
| --- | --- | --- |
| Textures | `Material::checker(...)` | Земля и куб в scenes 3/4 |
| Reflection / refraction | Recursive rays, Snell + IOR | Glass sphere в 3/4, вода в 5 |
| Particles | `ParticleCloud` | Фонтан в scene 5 |
| Fluids | `FluidSurface` | Волнистая вода в scene 5 |

### Процедурные текстуры

```bash
--texture
```

`Material::checker(...)` создаёт двухцветный checker pattern. В scenes 3/4 он виден и на земле, и на кубе.

### Отражение

```bash
--reflect
```

Материалы с `reflectivity > 0` рекурсивно трассируют отражённые лучи.

### Преломление

```bash
--refract
```

Прозрачные материалы используют закон Снеллиуса и `ior`. Стеклянная сфера в scenes 3/4 показывает refraction; вода в scene 5 использует `ior = 1.333`.

### Частицы

`ParticleCloud` хранит множество маленьких частиц, но реализует один `Hittable`. `ParticleCloud::fountain(...)` создаёт повторяемый фонтан без `rand` и сторонних зависимостей.

```rust
scene.add(ParticleCloud::fountain(
    Vec3::new(-2.0, 0.1, 0.0),
    72,
    Material::new(Color::new(1.0, 0.45, 0.12)),
));
```

### Жидкости

`FluidSurface` — конечный height field:

```text
y = base + amplitude * sin(f*x) * cos(f*z)
```

Пересечение ищется внутри bounding box и уточняется bisection; normal вычисляется аналитически.

```rust
scene.add(FluidSurface::new(
    Vec3::new(1.35, 0.48, 0.0),
    1.75,
    1.55,
    0.18,
    2.7,
    water_material,
));
```

Все бонусы вместе:

```bash
./target/release/rt --scene 5 --reflect --refract --texture --out bonus.ppm
```

## 🧱 Создание и изменение сцен

Встроенные сцены находятся в [`src/scenes.rs`](src/scenes.rs). Scenes 1-4 — обязательные, scene 5 — изолированная bonus demo.

Примеры обязательных объектов:

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

Чтобы добавить scene 6:

1. скопируй scene-builder в `src/scenes.rs`;
2. добавь `6 => your_scene_function()` в `scenes::build`;
3. расширь CLI validation/help в `src/main.rs` с `1..=5`;
4. `cargo build --release`.

Если нужно только переместить объект, изменить свет или камеру, новый preset не нужен.

## 🧪 Проверка

```bash
cargo fmt -- --check
cargo test
cargo build --release
```

Unit-тесты проверяют:

- degenerate/top-down camera basis;
- cube ray from inside + правильную exit-face normal;
- P3 sample count и line wrapping;
- прямое пересечение `ParticleCloud`;
- прямое пересечение `FluidSurface`.

Smoke tests:

```bash
./target/release/rt --scene 1 --width 160 --height 120 --out scene1.ppm
./target/release/rt --scene 3 --width 160 --height 120 \
  --reflect --refract --texture --out scene3.ppm
./target/release/rt --scene 5 --width 320 --height 240 \
  --reflect --refract --texture --out bonus.ppm
```

PPM начинается с:

```text
P3
<width> <height>
255
```

## 📁 Структура проекта

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
│   │   ├── fluid.rs
│   │   ├── particle.rs
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

## ⚠️ Примечания

- Один primary ray на пиксель; anti-aliasing не входит в scope.
- Глубина reflection/refraction ограничена.
- Particle bonus статический и детерминированный, а не animation system.
- Fluid bonus — ray-traced procedural wavy surface, не Navier-Stokes simulation.
- Runtime status идёт в `stderr`; image data — в `stdout`, если нет `--out`.
- В `renders/` остаются только четыре обязательных PPM; bonus render генерируется локально при необходимости.

## 🧑‍💻 Авторы

- Atabek Furkat [**@abakhram**](https://01.tomorrow-school.ai/intra/astanahub/users/8197)
- Nazar Yestayev [**@nyestaye**](https://01.tomorrow-school.ai/intra/astanahub/users/4468)
- Sultan Yersultan [**@syersult**](https://01.tomorrow-school.ai/intra/astanahub/users/4423)
- Maksat Kapan [**@mkapan**](https://01.tomorrow-school.ai/intra/astanahub/users/3597)
- Daniyar Shadykhanov [**@dshadykh**](https://01.tomorrow-school.ai/intra/astanahub/users/2418)
- Taalaibek Adzhikulov [**@tadzhiku**](https://01.tomorrow-school.ai/intra/astanahub/users/2158)
