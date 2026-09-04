use crate::vec3::Color;
use std::io::{self, Write};

const MAX_PPM_LINE_LEN: usize = 70;

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

    // P3 treats any whitespace as a separator, so line boundaries do not have
    // to match image rows. Keep complete RGB triplets together while wrapping
    // output to the traditional Netpbm 70-character line limit.
    let mut line = String::with_capacity(MAX_PPM_LINE_LEN);
    for c in pixels {
        let pixel = format!("{} {} {}", to_byte(c.x), to_byte(c.y), to_byte(c.z));
        let separator_len = if line.is_empty() { 0 } else { 1 };

        if !line.is_empty() && line.len() + separator_len + pixel.len() > MAX_PPM_LINE_LEN {
            writeln!(writer, "{}", line)?;
            line.clear();
        }

        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&pixel);
    }

    if !line.is_empty() {
        writeln!(writer, "{}", line)?;
    }

    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Vec3;

    #[test]
    fn ppm_body_lines_stay_within_limit_and_keep_all_samples() {
        let pixels = vec![Vec3::white(); 40];
        let mut out = Vec::new();
        write_ppm(&mut out, 10, 4, &pixels).expect("write PPM");

        let text = String::from_utf8(out).expect("ASCII PPM");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "P3");
        assert_eq!(lines[1], "10 4");
        assert_eq!(lines[2], "255");
        assert!(lines[3..].iter().all(|line| line.len() <= MAX_PPM_LINE_LEN));

        let samples = lines[3..]
            .iter()
            .flat_map(|line| line.split_whitespace())
            .count();
        assert_eq!(samples, 40 * 3);
    }
}
