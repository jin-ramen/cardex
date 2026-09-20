//! Market-price rows for the card footer.

use crate::tcgdex::{Pricing, TcgPlayerPrice};
use super::style::dim;

/// TCGdex sends `0` rather than omitting a price it doesn't have.
fn nonzero(v: Option<f64>) -> Option<f64> {
    v.filter(|&x| x > 0.0)
}

/// (label, values) pairs — one row per market / print variant.
pub fn rows(p: &Pricing) -> Vec<(String, String)> {
    let mut rows = Vec::new();

    if let Some(cm) = &p.cardmarket {
        let line = |trend: Option<f64>, avg7: Option<f64>, low: Option<f64>| -> Option<String> {
            let mut parts = Vec::new();
            if let Some(v) = nonzero(trend) {
                parts.push(format!("trend {}", money(&cm.unit, v)));
            }
            if let Some(v) = nonzero(avg7) {
                parts.push(format!("7d {}", money(&cm.unit, v)));
            }
            if let Some(v) = nonzero(low) {
                parts.push(format!("low {}", money(&cm.unit, v)));
            }
            (!parts.is_empty()).then(|| parts.join(" · "))
        };
        if let Some(v) = line(cm.trend.or(cm.avg), cm.avg7, cm.low) {
            rows.push((dim("Cardmarket"), v));
        }
        if let Some(v) = line(cm.trend_holo.or(cm.avg_holo), cm.avg7_holo, cm.low_holo) {
            rows.push((format!("  {}", dim("foil")), v));
        }
    }

    if let Some(tp) = &p.tcgplayer {
        let mut variants: Vec<(&String, &TcgPlayerPrice)> = tp.variants.iter().collect();
        variants.sort_by(|a, b| a.0.cmp(b.0));
        let mut first = true;
        for (name, v) in variants {
            let mut parts = Vec::new();
            if let Some(x) = nonzero(v.low_price) {
                parts.push(format!("low {}", money(&tp.unit, x)));
            }
            if let Some(x) = nonzero(v.market_price).or(nonzero(v.mid_price)) {
                parts.push(format!("mkt {}", money(&tp.unit, x)));
            }
            if let Some(x) = nonzero(v.high_price) {
                parts.push(format!("high {}", money(&tp.unit, x)));
            }
            if parts.is_empty() {
                continue;
            }
            if first {
                rows.push((dim("TCGplayer"), String::new()));
                first = false;
            }
            rows.push((format!("  {}", dim(name)), parts.join(" · ")));
        }
    }
    rows
}

fn money(unit: &str, v: f64) -> String {
    // Drop the cents once a card is worth real money, to keep rows short.
    let n = if v >= 100.0 { format!("{v:.0}") } else { format!("{v:.2}") };
    match unit {
        "EUR" => format!("€{n}"),
        "USD" => format!("${n}"),
        "GBP" => format!("£{n}"),
        u => format!("{u} {n}"),
    }
}