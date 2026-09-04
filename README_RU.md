# RT — CPU Ray Tracer

CPU ray tracer на Rust для задания 01-edu `rt`. Проект рендерит сферы, кубы, плоскости и цилиндры в ASCII PPM (`P3`), поддерживает перемещение камеры, настройку света, отбрасываемые тени, процедурные текстуры, отражение и преломление.

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

### Рендер полной сцены

```bash
./target/release/rt \
  --scene 3 \
  --reflect \
  --refract \
  --texture \
  --out output.ppm
```

Поддерживается и вариант из subject с выводом изображения через `stdout`. Служебные сообщения идут в `stderr`, поэтому PPM не портится:

```bash
cargo run --release -- --scene 1 > output.ppm
```

Для быстрых проверок уменьши разрешение:

```bash
./target/release/rt --scene 3 --width 200 --height 150 --out preview.ppm
```

## 📝 О проекте

Для каждого пикселя камера выпускает луч в сцену. Рендерер ищет ближайшее пересечение с объектами и рассчитывает поверхность по Phong-подобной модели освещения:

- **ambient** — базовая составляющая;
- **diffuse** — ламбертовское освещение от видимых источников;
- **specular** — блики, зависящие от направления камеры;
- **shadows** — от точки пересечения к каждому источнику отправляется shadow ray; если другой объект перекрывает свет, вклад этого источника не учитывается.

Опциональные отражение и преломление рекурсивно трассируют вторичные лучи с ограниченной глубиной. Procedural checker texture вычисляется непосредственно по 3D-координатам точки пересечения.

Строки изображения рендерятся параллельно через `std::thread::scope`. Финальный результат записывается в текстовый PPM `P3` с безопасным переносом длинных строк.

Подробное описание API и примеры создания объектов находятся в [docs/DOCUMENTATION.md](docs/DOCUMENTATION.md).

## ✅ Обязательные возможности

| Требование | Реализация |
| --- | --- |
| Сфера | `src/shapes/sphere.rs` |
| Куб | `src/shapes/cube.rs` |
| Плоскость | `src/shapes/plane.rs` |
| Цилиндр | `src/shapes/cylinder.rs` |
| Перемещение объектов | Координаты/центры/оси конструкторов в `src/scenes.rs` |
| Перемещение и направление камеры | `--camera`, `--look-at`, `--fov` |
| Изменение яркости | Интенсивность каждого света и глобальный `--brightness` |
| Тени | Shadow rays в `src/renderer.rs` |
| Изменяемое разрешение | `--width`, `--height` |
| PPM | ASCII `P3` writer в `src/ppm.rs` |

Базис камеры содержит fallback для строго верхнего/нижнего ракурса и случая, когда `position` совпадает с `look_at`, поэтому типичные camera overrides не схлопывают изображение в один луч.

## 🖼️ Обязательные demo-сцены

Четыре файла для аудита лежат в [`renders/`](renders/) и имеют разрешение **800×600**.

| Файл | Сцена |
| --- | --- |
| `scene1_sphere.ppm` | Сфера над плоскостью, яркий key light + мягкий fill light |
| `scene2_plane_and_cube_lower_brightness.ppm` | Плоскость + куб, key light слабее, чем в scene 1 |
| `scene3_all_objects.ppm` | Все обязательные примитивы + стеклянная сфера для демонстрации преломления |
| `scene4_all_objects_alt_camera.ppm` | Та же сцена, что scene 3, но с другой позицией камеры |

Перегенерация всех четырёх:

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

Scene 2 использует тот же мягкий fill light, что и scene 1, но key light имеет меньшую интенсивность (`0.5` вместо `1.2`). Это сохраняет требуемую разницу по яркости и одновременно делает переднюю грань куба читаемой.

## 🎛️ CLI

```text
rt [OPTIONS]

--scene <1|2|3|4>    Demo-сцена (по умолчанию: 1)
--width <N>          Ширина (по умолчанию: 800)
--height <N>         Высота (по умолчанию: 600)
--out <FILE>         Записать PPM в файл (по умолчанию: stdout)
--reflect            Включить отражение
--refract            Включить преломление
--texture            Включить процедурные текстуры
--brightness <F>     Общий множитель яркости (по умолчанию: 1.0)
--camera <x,y,z>     Переопределить позицию камеры
--look-at <x,y,z>    Переопределить точку взгляда
--fov <deg>          Переопределить вертикальный FOV
--threads <N>        Количество worker threads
-h, --help           Показать help
```

Изменение ракурса без правки кода:

```bash
./target/release/rt --scene 3 --out front.ppm
./target/release/rt --scene 3 \
  --camera 6,4,-6 \
  --look-at 0,1,0 \
  --fov 45 \
  --out side.ppm
```

Работает и строгий вид сверху:

```bash
./target/release/rt --scene 3 \
  --camera 0,10,0 \
  --look-at 0,0,0 \
  --out top.ppm
```

## ✨ Бонусы

### Процедурные текстуры

Включение:

```bash
--texture
```

`Material::checker(...)` создаёт двухцветный checker pattern. В scene 3/4 он виден и на земле, и на кубе.

### Отражение

Включение:

```bash
--reflect
```

Материалы с `reflectivity > 0` рекурсивно трассируют отражённые лучи.

### Преломление

Включение:

```bash
--refract
```

Прозрачные материалы используют закон Снеллиуса и `ior`; при невозможности преломления используется total internal reflection. Дополнительная стеклянная сфера в scene 3/4 демонстрирует этот бонус.

Частицы и жидкости не реализованы.

## 🧱 Создание и изменение сцен

Четыре встроенные сцены находятся в [`src/scenes.rs`](src/scenes.rs). Для обычных экспериментов аудита достаточно изменить существующую сцену или использовать CLI overrides камеры и яркости.

Примеры создания объектов:

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

Чтобы добавить новый numbered preset, например scene 5:

1. скопируй одну из функций построения сцены в `src/scenes.rs` и измени объекты/свет/камеру;
2. добавь `5 => your_scene_function()` в `match` внутри `scenes::build`;
3. расширь в `src/main.rs` валидацию `--scene` и текст help с `1..=4` на новый диапазон;
4. пересобери проект через `cargo build --release`.

Если нужно только переместить объекты, изменить яркость или камеру существующей сцены, новый preset не нужен.

## 🧪 Проверка

Перед аудитом:

```bash
cargo fmt -- --check
cargo test
cargo build --release
```

Unit-тесты закрывают основные robustness-фиксы:

- top-down и degenerate camera basis;
- луч, начинающийся внутри куба, с корректным попаданием в exit face;
- количество P3 samples и ограничение длины строк.

Быстрый smoke test:

```bash
./target/release/rt --scene 1 --width 160 --height 120 --out /tmp/scene1.ppm
./target/release/rt --scene 3 --width 160 --height 120 \
  --reflect --refract --texture --out /tmp/scene3.ppm
```

Корректный файл начинается с:

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

- Рендерер намеренно использует один primary ray на пиксель; anti-aliasing не входит в scope задания.
- Глубина рекурсии reflection/refraction ограничена для гарантированного завершения.
- Бонусные эффекты включаются флагами и не замедляют default render.
- Runtime status идёт в `stderr`; данные изображения — в `stdout`, если не указан `--out`.
- Корневой `output.ppm` добавлен в `.gitignore`, чтобы временные рендеры не попадали в репозиторий; четыре обязательных файла остаются в `renders/`.

## 🧑‍💻 Авторы

- Atabek Furkat [**@abakhram**](https://01.tomorrow-school.ai/intra/astanahub/users/8197)
- Nazar Yestayev [**@nyestaye**](https://01.tomorrow-school.ai/intra/astanahub/users/4468)
- Sultan Yersultan [**@syersult**](https://01.tomorrow-school.ai/intra/astanahub/users/4423)
- Maksat Kapan [**@mkapan**](https://01.tomorrow-school.ai/intra/astanahub/users/3597)
- Daniyar Shadykhanov [**@dshadykh**](https://01.tomorrow-school.ai/intra/astanahub/users/2418)
- Taalaibek Adzhikulov [**@tadzhiku**](https://01.tomorrow-school.ai/intra/astanahub/users/2158)
