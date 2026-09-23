//! Set list (name + counts) and a full set page with logo, metadata and
//! a multi-column card index.

use std::io::{self, Write};

use image::RgbaImage;

use crate::tcgdex::{CardBrief, CardCount, Set, SetBrief, TcgDexClient};
use super::frame::{beside, Frame};
use super::image as art;
use super::style::{bold, dim, fg, legal_badges, term_cols, vis_width, GOLD};

const PAD_LEFT: usize = 2;
const GUTTER: usize = 3;
const FALLBACK_COLS: usize = 100;
/// Logo beside the box: 8 lines tall, up to 36 columns wide.
const LOGO_ROWS: usize = 8;
const LOGO_COLS: usize = 36;
/// The symbol sits in the info box beside the name: 2 lines, 6-column slot.
const SYMBOL_ROWS: usize = 2;
const SYMBOL_COLS: usize = 6;
/// Card index columns: local number width and the longest name shown.
const NUM_COL: usize = 4;
const NAME_MAX: usize = 22;
const COL_GAP: usize = 4;

/// Set accent — no type to key off, so a warm neutral.
const SET_RGB: (u8, u8, u8) = (220, 180, 120);

// ───────────────────────────────────────────────────────────────────────
// Set list
// ───────────────────────────────────────────────────────────────────────

/// ```text
///   168 sets
///
///   ▌ Darkness Ablaze  swsh3
///     189 cards · +12 secret
/// ```
/// Print the set list to stdout.
pub fn render_sets(sets: &[SetBrief]) -> io::Result<()> {
    write_sets(&mut io::stdout().lock(), sets)
}

fn write_sets(w: &mut impl Write, sets: &[SetBrief]) -> io::Result<()> {
    let margin = " ".repeat(PAD_LEFT);

    writeln!(w)?;
    writeln!(w, "{margin}{}", bold(&format!("{} sets", sets.len())))?;
    writeln!(w)?;

    for s in sets {
        writeln!(w, "{margin}{} {}  {}", fg("▌", SET_RGB), bold(&s.name), dim(&s.id))?;
        writeln!(w, "{margin}  {}", dim(&count_summary(&s.card_count)))?;
    }
    writeln!(w)?;
    Ok(())
}

// ───────────────────────────────────────────────────────────────────────
// Single set
// ───────────────────────────────────────────────────────────────────────

/// ```text
///                 ╭──────────────────────────────────────────────────╮
///   ████  ████    │ ▄▀▄  ▌ Darkness Ablaze                     swsh3 │
///   ██ ████ ██    │ ▀▄▀    Sword & Shield · released 2020-08-14      │
///   ████  ████    ├──────────────────────────────────────────────────┤
///    (logo)       │ 189 official · 201 total · 143 reverse · 21 holo │
///                 │ Standard ✗  Expanded ✓                           │
///                 ╰──────────────────────────────────────────────────╯
///
///   201 cards
///      1  Butterfree V           68  Crawdaunt              135  ...
/// ```
/// Fetch the logo and symbol (both at once) and print the set page to
/// stdout. Either image failing just leaves it off.
pub async fn render_set(client: &TcgDexClient, s: &Set) -> anyhow::Result<()> {
    let (logo, symbol) = tokio::join!(client.set_logo(s), client.set_symbol(s));
    let (logo, symbol) = (logo.ok().flatten(), symbol.ok().flatten());
    write_set(&mut io::stdout().lock(), s, logo.as_ref(), symbol.as_ref())?;
    Ok(())
}

fn write_set(
    w: &mut impl Write,
    s: &Set,
    logo: Option<&RgbaImage>,
    symbol: Option<&RgbaImage>,
) -> io::Result<()> {
    let margin = " ".repeat(PAD_LEFT);
    let cols = term_cols(FALLBACK_COLS);

    // ── Logo (left) beside the info box (right) ──────────────────────
    let logo_lines = logo.map(|l| art::sprite(l, LOGO_ROWS, LOGO_COLS)).unwrap_or_default();
    let logo_cols = logo_lines.first().map(|l| vis_width(l) + GUTTER).unwrap_or(0);
    let inner = cols.saturating_sub(PAD_LEFT + logo_cols + 4).clamp(50, 80);
    let mut f = Frame::new(inner, SET_RGB);
    f.top();

    // Header: symbol (2 lines) beside name / meta.
    let sym = symbol
        .map(|img| art::sprite(img, SYMBOL_ROWS, SYMBOL_COLS))
        .unwrap_or_else(|| vec![" ".repeat(SYMBOL_COLS); SYMBOL_ROWS]);
    f.split(
        &format!("{} {} {}", sym[0], fg("▌", SET_RGB), bold(&fg(&s.name, GOLD))),
        &dim(&s.id),
    );
    let mut meta = vec![fg(&s.serie.name, SET_RGB), dim(&format!("released {}", s.release_date))];
    if let Some(code) = &s.tcg_online {
        meta.push(dim(&format!("TCGO {code}")));
    }
    f.row(&format!("{}   {}", sym[1], meta.join(&dim(" · "))));

    f.rule();
    f.row(&count_detail(&s.card_count));
    f.row(&legal_badges(s.legal.standard, s.legal.expanded));
    if let Some(boosters) = s.boosters.as_deref().filter(|b| !b.is_empty()) {
        let names = boosters.iter().map(|b| b.name.as_str()).collect::<Vec<_>>().join(", ");
        f.row(&dim(&format!("Packs: {names}")));
    }
    f.bottom();

    writeln!(w)?;
    for line in beside(&logo_lines, &f.lines, GUTTER) {
        writeln!(w, "{margin}{line}")?;
    }

    // ── Card index ───────────────────────────────────────────────────
    writeln!(w)?;
    writeln!(w, "{margin}{}", bold(&format!("{} cards", s.cards.len())))?;
    for line in card_grid(&s.cards, cols.saturating_sub(PAD_LEFT)) {
        writeln!(w, "{margin}{line}")?;
    }
    writeln!(w)?;
    Ok(())
}

/// Cards laid out column-major (read down, then across), as many columns
/// as fit in `width`. Names longer than `NAME_MAX` are cut with `…`.
fn card_grid(cards: &[CardBrief], width: usize) -> Vec<String> {
    let mut sorted: Vec<&CardBrief> = cards.iter().collect();
    sorted.sort_by_key(|c| local_key(&c.local_id));

    let name_w = sorted
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(0)
        .min(NAME_MAX);
    let cell_w = NUM_COL + 2 + name_w;
    let ncols = ((width + COL_GAP) / (cell_w + COL_GAP)).max(1);
    let nrows = (sorted.len() + ncols - 1) / ncols;

    let cell = |c: &CardBrief| {
        let name: String = if c.name.chars().count() > NAME_MAX {
            c.name.chars().take(NAME_MAX - 1).chain(std::iter::once('…')).collect()
        } else {
            c.name.clone()
        };
        let text = format!("{}  {name}", dim(&format!("{:>NUM_COL$}", c.local_id)));
        format!("{text}{}", " ".repeat(cell_w.saturating_sub(vis_width(&text))))
    };

    (0..nrows)
        .map(|r| {
            (0..ncols)
                .filter_map(|c| sorted.get(c * nrows + r))
                .map(|c| cell(c))
                .collect::<Vec<_>>()
                .join(&" ".repeat(COL_GAP))
                .trim_end()
                .to_string()
        })
        .collect()
}

// ───────────────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────────────

/// "189 cards · +12 secret"
fn count_summary(c: &CardCount) -> String {
    if c.total > c.official {
        format!("{} cards · +{} secret", c.official, c.total - c.official)
    } else {
        format!("{} cards", c.official)
    }
}

/// "189 official · 201 total · 143 reverse · 21 holo · 4 1st ed"
fn count_detail(c: &CardCount) -> String {
    let n = |v: u32, label: &str| format!("{} {}", bold(&v.to_string()), dim(label));
    let mut parts = vec![n(c.official, "official"), n(c.total, "total")];
    if let Some(v) = c.reverse.filter(|&v| v > 0) {
        parts.push(n(v, "reverse"));
    }
    if let Some(v) = c.holo.filter(|&v| v > 0) {
        parts.push(n(v, "holo"));
    }
    if let Some(v) = c.first_ed.filter(|&v| v > 0) {
        parts.push(n(v, "1st ed"));
    }
    parts.join(&dim(" · "))
}

/// Numeric ids sort as numbers ("4" before "20"); promo-style ids ("SWSH261")
/// fall back to string order after them.
fn local_key(s: &str) -> (u32, String) {
    (s.parse().unwrap_or(u32::MAX), s.to_string())
}