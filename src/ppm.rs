use crate::vec3::Color;
use std::io::{self, Write};

/// Convert a linear-space color component in [0,1] to a gamma-corrected byte,
/// giving a more natural-looking image (gamma 2.0, i.e. sqrt).
fn to_byte(c: f64) -> u8 {
    let gamma_corrected = c.max(0.0).sqrt();
    (gamma_corrected.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Write a PPM (P3, ASCII) image to `out`, given a flat row-major buffer of
/// colors (top-left to bottom-right, i.e. row 0 is the top row of the image).
pub fn write_ppm<W: Write>(out: &mut W, width: usize, height: usize, pixels: &[Color]) -> io::Result<()> {
    debug_assert_eq!(pixels.len(), width * height);

    let mut writer = io::BufWriter::new(out);
    writeln!(writer, "P3")?;
    writeln!(writer, "{} {}", width, height)?;
    writeln!(writer, "255")?;

    for row in pixels.chunks(width) {
        let mut line = String::with_capacity(width * 12);
        for (i, c) in row.iter().enumerate() {
            if i > 0 {
                line.push(' ');
            }
            line.push_str(&format!(
                "{} {} {}",
                to_byte(c.x),
                to_byte(c.y),
                to_byte(c.z)
            ));
        }
        writeln!(writer, "{}", line)?;
    }
    writer.flush()
}
