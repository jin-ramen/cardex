//! A rounded, type-coloured box with gradient dividers. Knows nothing
//! about cards — just rows, padding and borders.

use super::style::{fg, lerp, shade, vis_width, Rgb};

pub struct Frame {
    accent: Rgb,
    border: Rgb,
    inner: usize,
    pub lines: Vec<String>,
}

impl Frame {
    /// `inner` is the text width between the borders (padding excluded).
    pub fn new(inner: usize, accent: Rgb) -> Self {
        Self { accent, border: shade(accent, 0.55), inner, lines: Vec::new() }
    }

    /// Text width between the borders.
    pub fn inner(&self) -> usize {
        self.inner
    }

    pub fn top(&mut self) {
        self.lines.push(fg(&format!("╭{}╮", "─".repeat(self.inner + 2)), self.border));
    }

    pub fn bottom(&mut self) {
        self.lines.push(fg(&format!("╰{}╯", "─".repeat(self.inner + 2)), self.border));
    }

    /// Divider fading from the accent colour (left) to the border colour (right).
    pub fn rule(&mut self) {
        let mut s = fg("├", self.border);
        let n = self.inner + 2;
        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            s.push_str(&fg("─", lerp(self.accent, self.border, t)));
        }
        s.push_str(&fg("┤", self.border));
        self.lines.push(s);
    }

    pub fn blank(&mut self) {
        self.row("");
    }

    pub fn row(&mut self, content: &str) {
        let pad = self.inner.saturating_sub(vis_width(content));
        let b = fg("│", self.border);
        self.lines.push(format!("{b} {content}{} {b}", " ".repeat(pad)));
    }

    pub fn split(&mut self, left: &str, right: &str) {
        if right.is_empty() {
            return self.row(left);
        }
        let gap = self.inner.saturating_sub(vis_width(left) + vis_width(right)).max(1);
        self.row(&format!("{left}{}{right}", " ".repeat(gap)));
    }
}