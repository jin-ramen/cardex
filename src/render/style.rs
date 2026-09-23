//! ANSI colour and text primitives — nothing in here knows what a card is.

use crate::tcgdex::Variant;

pub type Rgb = (u8, u8, u8);

pub const GOLD: Rgb = (255, 200, 60);
pub const GREEN: Rgb = (90, 200, 120);
pub const TRAINER_RGB: Rgb = (120, 150, 190);
pub const ENERGY_RGB: Rgb = (190, 190, 190);

pub fn type_rgb(t: &str) -> Rgb {
    match t {
        "Grass" => (120, 200, 80),
        "Fire" => (240, 128, 48),
        "Water" => (104, 144, 240),
        "Lightning" => (248, 208, 48),
        "Psychic" => (248, 88, 136),
        "Fighting" => (192, 48, 40),
        "Darkness" => (112, 88, 72),
        "Metal" => (184, 184, 208),
        "Fairy" => (238, 153, 172),
        "Dragon" => (112, 56, 248),
        _ => (200, 200, 200),
    }
}

// ── SGR wrappers ─────────────────────────────────────────────────────

/// Foreground colour. Resets only the colour (`39m`), so it composes
/// inside `bold`/`dim` without cancelling them.
pub fn fg(s: &str, (r, g, b): Rgb) -> String {
    format!("\x1b[38;2;{r};{g};{b}m{s}\x1b[39m")
}
pub fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[22m")
}
pub fn dim(s: &str) -> String {
    format!("\x1b[2m{s}\x1b[22m")
}
pub fn italic(s: &str) -> String {
    format!("\x1b[3m{s}\x1b[23m")
}

// ── Colour maths ─────────────────────────────────────────────────────

/// Multiply a colour by `k` (0.0 = black, 1.0 = unchanged).
pub fn shade((r, g, b): Rgb, k: f32) -> Rgb {
    ((r as f32 * k) as u8, (g as f32 * k) as u8, (b as f32 * k) as u8)
}

pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

pub fn hsv(h: f32, s: f32, v: f32) -> Rgb {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}

// ── Flourishes ───────────────────────────────────────────────────────

/// Rainbow-foil text — each character gets its own hue.
pub fn holo(s: &str) -> String {
    let n = s.chars().count().max(1) as f32;
    s.chars()
        .enumerate()
        .map(|(i, ch)| bold(&fg(&ch.to_string(), hsv(300.0 * i as f32 / n, 0.55, 1.0))))
        .collect()
}

/// `▰▰▰▰▱▱` — `filled` out of `cells`.
pub fn bar(filled: usize, cells: usize, accent: Rgb) -> String {
    let filled = filled.min(cells);
    format!(
        "{}{}",
        fg(&"▰".repeat(filled), accent),
        fg(&"▱".repeat(cells - filled), shade(accent, 0.35)),
    )
}

pub fn is_rare(rarity: Option<&str>) -> bool {
    match rarity.map(str::to_ascii_lowercase) {
        None => false,
        Some(r) => !(r.contains("common") || r == "none"),
    }
}

/// "Double Rare ★★" — rarity name plus a glyph that scales with it.
pub fn rarity_badge(rarity: &str) -> String {
    let r = rarity.to_ascii_lowercase();
    let glyph = if r.contains("promo") {
        fg("✦", GOLD)
    } else if r.contains("uncommon") {
        dim("◆")
    } else if r.contains("common") {
        dim("●")
    } else if r.contains("hyper") || r.contains("rainbow") || r.contains("secret") || r.contains("gold") {
        holo("★★★")
    } else if r.contains("ultra") || r.contains("special") || r.contains("illustration") || r.contains("double") {
        holo("★★")
    } else if r.contains("rare") || r.contains("holo") {
        fg("★", GOLD)
    } else {
        dim("◇")
    };
    format!("{} {glyph}", dim(rarity))
}

pub fn variant_badges(v: &Variant) -> String {
    let mut out = Vec::new();
    if v.normal        { out.push(dim("normal")); }
    if v.reverse       { out.push(fg("reverse", (80, 200, 220))); }
    if v.holo          { out.push(fg("holo", (240, 190, 60))); }
    if v.first_edition { out.push(fg("1st ed", (250, 230, 100))); }
    if v.w_promo       { out.push(fg("W promo", (220, 100, 220))); }
    out.join("  ")
}

/// "Standard ✓  Expanded ✗"
pub fn legal_badges(standard: bool, expanded: bool) -> String {
    let tick = |ok: bool| if ok { fg("✓", GREEN) } else { dim("✗") };
    format!("{} {}  {} {}", dim("Standard"), tick(standard), dim("Expanded"), tick(expanded))
}

// ── Text measurement ─────────────────────────────────────────────────

/// Terminal width in columns, or `fallback` when it can't be read
/// (piped output, tests).
pub fn term_cols(fallback: usize) -> usize {
    terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(fallback)
}

/// On-screen width, ignoring ANSI escape sequences.
pub fn vis_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            width += 1;
        }
    }
    width
}

/// Greedy word-wrap on plain (un-styled) text.
pub fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in s.split_whitespace() {
        let cur = lines.last_mut().unwrap();
        let cur_w = cur.chars().count();
        if !cur.is_empty() && cur_w + 1 + word.chars().count() > width {
            lines.push(word.to_string());
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
    }
    lines
}