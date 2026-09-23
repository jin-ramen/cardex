//! Decoded images → half-block terminal lines. No I/O here: the client
//! fetches and decodes, this module only rasterises.

use image::imageops::{resize, FilterType};
use image::{RgbImage, RgbaImage};

use super::style::vis_width;

/// A Pokémon card is 63 × 88 mm.
const CARD_ASPECT: f32 = 63.0 / 88.0;
/// Pixel rows per terminal line — 2, because `image_lines` draws ▀ half-blocks.
const PX_PER_LINE: usize = 2;

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

// ───────────────────────────────────────────────────────────────────────
// Sprites: transparent PNGs (set logos)
// ───────────────────────────────────────────────────────────────────────

/// Alpha below this is drawn as an empty cell, so the terminal background
/// shows through instead of a black box.
const ALPHA_CUTOFF: u8 = 96;

/// Render a transparent image at exactly `rows` lines, at its own aspect
/// ratio, never wider than `max_cols`. Every line is padded to `max_cols`
/// (centred) so sprites line up in a list.
pub fn sprite(img: &RgbaImage, rows: usize, max_cols: usize) -> Vec<String> {
    let (iw, ih) = img.dimensions();
    let target_h = (rows * PX_PER_LINE) as u32;
    let mut w = (target_h as f32 * iw as f32 / ih as f32).round().max(1.0) as u32;
    let mut h = target_h;
    if w as usize > max_cols {
        w = max_cols as u32;
        h = (w as f32 * ih as f32 / iw as f32).round().max(1.0) as u32;
        h += h % 2; // keep an even pixel height for the half-block pairs
    }
    let scaled = resize(img, w, h, FilterType::Lanczos3);

    let mut lines = sprite_lines(&scaled);
    let pad_l = (max_cols - w as usize) / 2;
    let pad_r = max_cols - w as usize - pad_l;
    for l in &mut lines {
        *l = format!("{}{l}{}", " ".repeat(pad_l), " ".repeat(pad_r));
    }
    // A logo that hit `max_cols` may be shorter than `rows`; centre it.
    let blank = " ".repeat(max_cols);
    while lines.len() < rows {
        if lines.len() % 2 == 0 { lines.insert(0, blank.clone()) } else { lines.push(blank.clone()) }
    }
    lines
}

/// Like `image_lines`, but transparent pixels become empty cells.
fn sprite_lines(img: &RgbaImage) -> Vec<String> {
    let (width, height) = img.dimensions();
    let mut lines = Vec::with_capacity(height as usize / 2);

    for y in (0..height).step_by(2) {
        let mut line = String::with_capacity(width as usize * 40);
        for x in 0..width {
            let t = img.get_pixel(x, y);
            let b = img.get_pixel(x, (y + 1).min(height - 1));
            let (t_on, b_on) = (t[3] >= ALPHA_CUTOFF, b[3] >= ALPHA_CUTOFF);
            match (t_on, b_on) {
                (true, true) => line.push_str(&format!(
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                    t[0], t[1], t[2], b[0], b[1], b[2]
                )),
                (true, false) => line.push_str(&format!("\x1b[49m\x1b[38;2;{};{};{}m▀", t[0], t[1], t[2])),
                (false, true) => line.push_str(&format!("\x1b[49m\x1b[38;2;{};{};{}m▄", b[0], b[1], b[2])),
                (false, false) => line.push_str("\x1b[49m "),
            }
        }
        line.push_str("\x1b[0m");
        lines.push(line);
    }
    lines
}