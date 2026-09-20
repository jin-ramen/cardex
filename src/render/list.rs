//! Search results: cards grouped by set, one line each.

use std::io::{self, Write};

use crate::tcgdex::CardBrief;
use super::style::{bold, dim, fg, hsv};

const PAD_LEFT: usize = 2;
/// Right-aligned width of the local number column ("004", "SWSH261").
const NUM_COL: usize = 7;

/// ```text
///   87 cards matching “charizard” · 31 sets
///
///   ▌ base1 · 1
///         4  Charizard  base1-4
///   ▌ swsh3 · 3
///        20  Charizard VMAX  swsh3-20
/// ```
pub fn render_list(w: &mut impl Write, query: &str, cards: &[CardBrief]) -> io::Result<()> {
    let margin = " ".repeat(PAD_LEFT);

    writeln!(w)?;
    if cards.is_empty() {
        writeln!(w, "{margin}{}", dim(&format!("No cards matching “{query}”")))?;
        writeln!(w)?;
        return Ok(());
    }

    let groups = group_by_set(cards);
    writeln!(
        w,
        "{margin}{} {}",
        bold(&format!("{} cards", cards.len())),
        dim(&format!("matching “{query}” · {} sets", groups.len()))
    )?;
    writeln!(w)?;

    for (i, (set, cards)) in groups.iter().enumerate() {
        // Each set gets its own hue so the eye can jump between groups.
        let accent = hsv((i as f32 * 47.0) % 360.0, 0.5, 0.95);
        writeln!(
            w,
            "{margin}{} {} {}",
            fg("▌", accent),
            bold(&fg(set, accent)),
            dim(&format!("· {}", cards.len()))
        )?;
        for c in cards {
            writeln!(
                w,
                "{margin}  {}  {}  {}",
                dim(&format!("{:>NUM_COL$}", c.local_id)),
                c.name,
                dim(&c.id)
            )?;
        }
    }
    writeln!(w)?;
    Ok(())
}

/// Cards ordered by set id, then numeric local id, then bucketed per set.
fn group_by_set(cards: &[CardBrief]) -> Vec<(String, Vec<&CardBrief>)> {
    let mut sorted: Vec<&CardBrief> = cards.iter().collect();
    sorted.sort_by_key(|c| (set_of(&c.id).to_string(), local_key(&c.local_id)));

    let mut groups: Vec<(String, Vec<&CardBrief>)> = Vec::new();
    for c in sorted {
        let set = set_of(&c.id);
        match groups.last_mut() {
            Some((s, v)) if s == set => v.push(c),
            _ => groups.push((set.to_string(), vec![c])),
        }
    }
    groups
}

/// "swsh3-136" → "swsh3"
fn set_of(id: &str) -> &str {
    id.rsplit_once('-').map(|(s, _)| s).unwrap_or(id)
}

/// Numeric ids sort as numbers ("4" before "20"); promo-style ids ("SWSH261")
/// fall back to string order after them.
fn local_key(s: &str) -> (u32, String) {
    (s.parse().unwrap_or(u32::MAX), s.to_string())
}