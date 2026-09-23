//! Card layout: a framed, type-coloured info panel beside the art.

use std::io::{self, Write};

use image::RgbImage;

use crate::tcgdex::{Ability, Attack, Card, TcgDexClient, Weakness};
use super::frame::{beside, Frame};
use super::image as art;
use super::pricing;
use super::style::{
    bar, bold, dim, fg, holo, is_rare, italic, legal_badges, rarity_badge, shade, term_cols,
    type_rgb, variant_badges, wrap, Rgb, ENERGY_RGB, GOLD, TRAINER_RGB,
};

/// Text width inside the frame (excluding the border and its 1-char padding).
/// Sized to the terminal at render time, clamped to this range.
const INNER_MIN: usize = 50;
const INNER_MAX: usize = 80;
/// Used when the terminal width can't be read (piped output, tests).
const FALLBACK_COLS: usize = 100;
const INDENT: usize = 4;
const HP_BAR_CELLS: usize = 12;
const HP_MAX: usize = 340;

/// Every card's art is drawn at exactly this many lines, so cards are the
/// same size regardless of how much text they carry. The info box is centred
/// beside it — shorter for an Item, taller for a VMAX — and a box that
/// outgrows the art simply extends past it.
const ART_ROWS: usize = 24;

/// Layout padding, in cells.
const PAD_TOP: usize = 1;
const PAD_BOTTOM: usize = 1;
const PAD_LEFT: usize = 2;
const GUTTER: usize = 3;

// ───────────────────────────────────────────────────────────────────────
// Public entry point
// ───────────────────────────────────────────────────────────────────────

/// Fetch the artwork and print the card to stdout. If the art can't be
/// loaded the card still prints, text-only, with a note on stderr.
pub async fn render_card(client: &TcgDexClient, c: &Card) -> anyhow::Result<()> {
    let art = match client.card_art(c).await {
        Ok(art) => art,
        Err(e) => {
            eprintln!("{}", dim(&format!("(no artwork: {e})")));
            None
        }
    };
    write_card(&mut io::stdout().lock(), c, art.as_ref())?;
    Ok(())
}

/// Layout only: art (already decoded) beside a framed info panel.
fn write_card(w: &mut impl Write, c: &Card, img: Option<&RgbImage>) -> io::Result<()> {
    let inner = inner_width(img.is_some());
    let right = card_lines(c, inner);
    let left = img.map(|img| art::fit(img, ART_ROWS)).unwrap_or_default();

    let margin = " ".repeat(PAD_LEFT);
    for _ in 0..PAD_TOP {
        writeln!(w)?;
    }
    for line in beside(&left, &right, GUTTER) {
        writeln!(w, "{margin}{line}")?;
    }
    for _ in 0..PAD_BOTTOM {
        writeln!(w)?;
    }
    Ok(())
}

/// Box text width that fills the terminal, leaving room for margin,
/// art, gutter and the frame's own 4 columns.
fn inner_width(has_art: bool) -> usize {
    let cols = term_cols(FALLBACK_COLS);
    let art = if has_art { art::cols_for(ART_ROWS) + GUTTER } else { 0 };
    cols.saturating_sub(PAD_LEFT + art + 4).clamp(INNER_MIN, INNER_MAX)
}

// ───────────────────────────────────────────────────────────────────────
// Card body
// ───────────────────────────────────────────────────────────────────────

fn card_lines(c: &Card, inner: usize) -> Vec<String> {
    let accent = accent_for(c);
    let mut f = Frame::new(inner, accent);

    f.top();
    push_header(&mut f, c, accent);

    // ── Abilities / attacks / card text / held item ──────────────────
    let has_body = !c.abilities.is_empty()
        || !c.attacks.is_empty()
        || c.effect.is_some()
        || c.item.is_some();
    if has_body {
        f.rule();
        let mut first = true;
        let mut gap = |f: &mut Frame| {
            if !first {
                f.blank();
            }
            first = false;
        };
        for ab in &c.abilities {
            gap(&mut f);
            push_ability(&mut f, ab, accent);
        }
        for a in &c.attacks {
            gap(&mut f);
            push_attack(&mut f, a, accent);
        }
        if let Some(effect) = &c.effect {
            gap(&mut f);
            push_text(&mut f, effect, 0);
        }
        if let Some(item) = &c.item {
            gap(&mut f);
            f.row(&format!("{} {}", fg("⚙ Held item", accent), bold(&item.name)));
            push_text(&mut f, &item.effect, INDENT);
        }
    }

    // ── Weakness / resistance / retreat ──────────────────────────────
    if c.is_pokemon() {
        f.rule();
        let mut left: Vec<String> = Vec::new();
        if !c.weaknesses.is_empty() {
            left.push(format!("{} {}", dim("Weakness"), modifiers(&c.weaknesses)));
        }
        if !c.resistances.is_empty() {
            left.push(format!("{} {}", dim("Resistance"), modifiers(&c.resistances)));
        }
        let retreat = if c.retreat > 0 {
            fg(&"●".repeat(c.retreat as usize), shade(accent, 0.75))
        } else {
            dim("free")
        };
        f.split(&left.join("  "), &format!("{} {retreat}", dim("Retreat")));
    }

    // ── Flavour text ─────────────────────────────────────────────────
    if let Some(desc) = &c.description {
        f.rule();
        let lines = wrap(desc, f.inner() - 2);
        let n = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            let open = if i == 0 { "“" } else { " " };
            let close = if i + 1 == n { "”" } else { "" };
            f.row(&dim(&italic(&format!("{open}{line}{close}"))));
        }
    }

    // ── Footer ───────────────────────────────────────────────────────
    f.rule();
    let set = format!(
        "{} {}",
        fg(&c.set.name, shade(accent, 0.8)),
        dim(&format!("· {}", c.number()))
    );
    f.split(&set, &c.rarity.as_deref().map(rarity_badge).unwrap_or_default());

    let illus = c
        .illustrator
        .as_deref()
        .map(|i| dim(&format!("Illus. {i}")))
        .unwrap_or_default();
    f.split(&illus, &variant_badges(&c.variants));

    if c.regulation_mark.is_some() || c.legal.is_some() {
        let mark = c
            .regulation_mark
            .as_deref()
            .map(|m| format!("{} ", fg(&format!("[{m}]"), GOLD)))
            .unwrap_or_default();
        let legal = c.legal.as_ref().map(|l| legal_badges(l.standard, l.expanded)).unwrap_or_default();
        let updated = c
            .updated
            .as_deref()
            .map(|u| dim(&format!("upd. {}", &u[..u.len().min(10)])))
            .unwrap_or_default();
        f.split(&format!("{mark}{legal}"), &updated);
    }

    if !c.boosters.is_empty() {
        let names = c.boosters.iter().map(|b| b.name.as_str()).collect::<Vec<_>>().join(", ");
        for line in wrap(&format!("Packs: {names}"), f.inner()) {
            f.row(&dim(&line));
        }
    }

    // ── Market prices ────────────────────────────────────────────────
    if let Some(p) = &c.pricing {
        let rows = pricing::rows(p);
        if !rows.is_empty() {
            f.rule();
            for (label, values) in rows {
                f.split(&label, &fg(&values, GOLD));
            }
        }
    }

    f.bottom();
    f.lines
}

fn push_header(f: &mut Frame, c: &Card, accent: Rgb) {
    // Line 1: name (+ suffix) ..... HP
    let mut name = if is_rare(c.rarity.as_deref()) {
        holo(&c.name)
    } else {
        bold(&fg(&c.name, accent))
    };
    if let Some(sfx) = &c.suffix {
        if !c.name.to_lowercase().contains(&sfx.to_lowercase()) {
            name.push(' ');
            name.push_str(&fg(sfx, GOLD));
        }
    }
    let hp = c
        .hp
        .map(|hp| format!("{} {}", bold(&fg(&hp.to_string(), accent)), dim("HP")))
        .unwrap_or_default();
    f.split(&format!("{} {name}", fg("▌", accent)), &hp);

    // Line 2: category-specific meta ..... HP bar
    let mut meta: Vec<String> = Vec::new();
    if c.is_pokemon() {
        if let Some(stage) = &c.stage {
            meta.push(dim(stage));
        }
        if let Some(lv) = c.level_str() {
            meta.push(dim(&format!("LV.{lv}")));
        }
        if let Some(types) = &c.types {
            meta.push(types.iter().map(|t| fg(t, type_rgb(t))).collect::<Vec<_>>().join(" "));
        }
    } else {
        meta.push(fg(&c.category, accent));
        if let Some(t) = c.trainer_type.as_deref().or(c.energy_type.as_deref()) {
            meta.push(dim(t));
        }
    }
    let hp_bar = c
        .hp
        .map(|hp| bar((hp as usize * HP_BAR_CELLS / HP_MAX).max(1), HP_BAR_CELLS, accent))
        .unwrap_or_default();
    f.split(&format!("  {}", meta.join(&dim(" · "))), &hp_bar);

    // Line 3 (Pokémon only): dex number · evolves from
    let mut lineage: Vec<String> = Vec::new();
    if let Some(dex) = c.dex_label() {
        lineage.push(fg(&dex, shade(accent, 0.7)));
    }
    if let Some(from) = &c.evolve_from {
        lineage.push(dim(&format!("evolves from {from}")));
    }
    if !lineage.is_empty() {
        f.row(&format!("  {}", lineage.join(&dim(" · "))));
    }
}

fn push_ability(f: &mut Frame, ab: &Ability, accent: Rgb) {
    f.row(&format!("{} {}", fg(&format!("◈ {}", ab.kind), accent), bold(&ab.name)));
    push_text(f, &ab.effect, INDENT);
}

fn push_attack(f: &mut Frame, a: &Attack, accent: Rgb) {
    let cost = if a.cost.is_empty() {
        dim("—")
    } else {
        a.cost.iter().map(|t| fg("●", type_rgb(t))).collect()
    };
    let damage = a.damage_str().map(|d| bold(&fg(&d, accent))).unwrap_or_default();
    f.split(&format!("{cost}  {}", bold(&a.name)), &damage);
    if let Some(effect) = &a.effect {
        push_text(f, effect, INDENT);
    }
}

fn push_text(f: &mut Frame, text: &str, indent: usize) {
    let pad = " ".repeat(indent);
    for line in wrap(text, f.inner() - indent) {
        f.row(&format!("{pad}{}", dim(&line)));
    }
}

fn modifiers(ms: &[Weakness]) -> String {
    ms.iter()
        .map(|m| format!("{}{}", fg("●", type_rgb(&m.weakness_type)), m.value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn accent_for(c: &Card) -> Rgb {
    if let Some(t) = c.types.as_deref().and_then(|t| t.first()) {
        return type_rgb(t);
    }
    if c.is_trainer() {
        TRAINER_RGB
    } else if c.is_energy() {
        // "Fire Energy" → Fire colour, when a type word appears in the name.
        c.name
            .split_whitespace()
            .find(|w| !w.eq_ignore_ascii_case("energy"))
            .map(type_rgb)
            .unwrap_or(ENERGY_RGB)
    } else {
        type_rgb("Colorless")
    }
}