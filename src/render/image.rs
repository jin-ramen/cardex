//! Card artwork → half-block terminal lines.

use image::imageops::{resize, FilterType};
use image::RgbImage;

use super::style::vis_width;

/// A Pokémon card is 63 × 88 mm.
const CARD_ASPECT: f32 = 63.0 / 88.0;
/// Pixel rows per terminal line — 2, because `image_lines` draws ▀ half-blocks.
const PX_PER_LINE: usize = 2;

/// Decode only. Sizing happens once, in `fit`, from the full-res source —
/// resizing here as well would resample twice and blur the art.
pub fn prepare(bytes: &[u8]) -> Result<RgbImage, image::ImageError> {
    Ok(image::load_from_memory(bytes)?.to_rgb8())
}

/// Terminal columns the art occupies when drawn at `rows` lines.
pub fn cols_for(rows: usize) -> usize {
    let h = (rows * PX_PER_LINE) as f32;
    (h * CARD_ASPECT).round().max(1.0) as usize
}

/// Render `img` at exactly `rows` lines, at real card proportions.
pub fn fit(img: &RgbImage, rows: usize) -> Vec<String> {
    let h = (rows * PX_PER_LINE) as u32;
    let w = cols_for(rows) as u32;
    let mut lines = image_lines(&resize(img, w, h.max(1), FilterType::Lanczos3));

    // `h` is even so this should already be exact; guard anyway.
    if lines.len() > rows {
        let extra = lines.len() - rows;
        lines.drain(..extra / 2);
        lines.truncate(rows);
    } else if lines.len() < rows && !lines.is_empty() {
        let blank = " ".repeat(vis_width(&lines[0]));
        lines.resize(rows, blank);
    }
    lines
}

/// One ▀ per column: foreground = top pixel, background = bottom pixel.
pub fn image_lines(img: &RgbImage) -> Vec<String> {
    let (width, height) = img.dimensions();
    let mut lines = Vec::with_capacity(height as usize / 2);

    for y in (0..height).step_by(2) {
        let mut line = String::with_capacity(width as usize * 40);
        for x in 0..width {
            let t = img.get_pixel(x, y);
            let b = img.get_pixel(x, (y + 1).min(height - 1));
            line.push_str(&format!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                t[0], t[1], t[2], b[0], b[1], b[2]
            ));
        }
        line.push_str("\x1b[0m");
        lines.push(line);
    }
    lines
}