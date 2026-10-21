// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! A tiny pixel canvas rendered to the terminal with upper-half-block characters:
//! each terminal cell shows two vertically stacked pixels (fg = top, bg = bottom).

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
};

#[derive(Clone, Debug)]
pub struct Pixmap {
    pub w: usize,
    pub h: usize,
    data: Vec<Option<Color>>,
}

impl Pixmap {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            data: vec![None; w * h],
        }
    }

    /// Build from rows of characters; `map` turns a char into a color (None = transparent).
    pub fn from_rows(rows: &[&str], map: impl Fn(char) -> Option<Color>) -> Self {
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut p = Pixmap::new(w, rows.len());
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                p.data[y * w + x] = map(ch);
            }
        }
        p
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.w && y < self.h {
            self.data[y * self.w + x]
        } else {
            None
        }
    }

    pub fn set(&mut self, x: i32, y: i32, c: Option<Color>) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.data[y as usize * self.w + x as usize] = c;
        }
    }

    /// Copy the opaque pixels of `src` onto self at (ox, oy).
    pub fn blit(&mut self, src: &Pixmap, ox: i32, oy: i32) {
        for y in 0..src.h {
            for x in 0..src.w {
                if let Some(c) = src.get(x, y) {
                    self.set(ox + x as i32, oy + y as i32, Some(c));
                }
            }
        }
    }

    /// Resample to `tw`×`th`. Near-1:1 and upscaling use nearest neighbour (keeps pixel
    /// art crisp); real downscaling averages each source block so detail in
    /// high-resolution PNGs blends instead of dropping out.
    pub fn scaled(&self, tw: usize, th: usize) -> Pixmap {
        let mut p = Pixmap::new(tw, th);
        if self.w == 0 || self.h == 0 || tw == 0 || th == 0 {
            return p;
        }
        let shrinking = tw * 3 < self.w * 2 || th * 3 < self.h * 2;
        for y in 0..th {
            for x in 0..tw {
                p.data[y * tw + x] = if shrinking {
                    self.block_average(
                        x * self.w / tw,
                        (x + 1) * self.w / tw,
                        y * self.h / th,
                        (y + 1) * self.h / th,
                    )
                } else {
                    let sx = ((2 * x + 1) * self.w) / (2 * tw);
                    let sy = ((2 * y + 1) * self.h) / (2 * th);
                    self.get(sx, sy)
                };
            }
        }
        p
    }

    fn block_average(&self, x0: usize, x1: usize, y0: usize, y1: usize) -> Option<Color> {
        let (x1, y1) = (x1.max(x0 + 1), y1.max(y0 + 1));
        let (mut r, mut g, mut b, mut n, mut total) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for y in y0..y1.min(self.h) {
            for x in x0..x1.min(self.w) {
                total += 1;
                if let Some(Color::Rgb(cr, cg, cb)) = self.get(x, y) {
                    r += cr as u32;
                    g += cg as u32;
                    b += cb as u32;
                    n += 1;
                }
            }
        }
        // mostly transparent block stays transparent
        if n == 0 || n * 2 < total {
            return None;
        }
        Some(Color::Rgb((r / n) as u8, (g / n) as u8, (b / n) as u8))
    }

    /// From 8-bit RGBA bytes; alpha below 128 counts as transparent.
    pub fn from_rgba(w: usize, h: usize, rgba: &[u8]) -> Pixmap {
        let mut p = Pixmap::new(w, h);
        for (i, px) in rgba.chunks_exact(4).take(w * h).enumerate() {
            if px[3] >= 128 {
                p.data[i] = Some(Color::Rgb(px[0], px[1], px[2]));
            }
        }
        p
    }

    /// To 8-bit RGBA bytes, upscaled by an integer `scale` (nearest neighbour).
    pub fn to_rgba(&self, scale: usize) -> Vec<u8> {
        let scale = scale.max(1);
        let (w, h) = (self.w * scale, self.h * scale);
        let mut out = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                match self.get(x / scale, y / scale) {
                    Some(Color::Rgb(r, g, b)) => out.extend_from_slice(&[r, g, b, 255]),
                    _ => out.extend_from_slice(&[0, 0, 0, 0]),
                }
            }
        }
        out
    }

    pub fn rotate_cw(&self) -> Pixmap {
        let mut p = Pixmap::new(self.h, self.w);
        for y in 0..p.h {
            for x in 0..p.w {
                p.data[y * p.w + x] = self.get(y, self.h - 1 - x);
            }
        }
        p
    }

    pub fn rotate_180(&self) -> Pixmap {
        let mut p = self.clone();
        p.data.reverse();
        p
    }

    /// Draw at terminal cell (x0, y0). Each cell covers two pixel rows.
    /// Transparent pixels use `fallback`. Nothing is drawn outside `clip`.
    pub fn render(&self, buf: &mut Buffer, x0: u16, y0: u16, fallback: Color, clip: Rect) {
        let clip = clip.intersection(buf.area);
        let rows = self.h.div_ceil(2);
        for cy in 0..rows {
            for cx in 0..self.w {
                let (px, py) = (x0 as usize + cx, y0 as usize + cy);
                if px > u16::MAX as usize || py > u16::MAX as usize {
                    continue;
                }
                let pos = Position::new(px as u16, py as u16);
                if !clip.contains(pos) {
                    continue;
                }
                let top = self.get(cx, cy * 2).unwrap_or(fallback);
                let bot = if cy * 2 + 1 < self.h {
                    self.get(cx, cy * 2 + 1).unwrap_or(fallback)
                } else {
                    fallback
                };
                if let Some(cell) = buf.cell_mut(pos) {
                    cell.reset();
                    if top == bot {
                        cell.set_char(' ').set_bg(top);
                    } else {
                        cell.set_char('▀').set_fg(top).set_bg(bot);
                    }
                }
            }
        }
    }
}

/// 3×5 pixel font, just the letters the title needs.
fn glyph(c: char) -> [&'static str; 5] {
    match c {
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'S' => ["###", "#..", "###", "..#", "###"],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'O' => ["###", "#.#", "#.#", "#.#", "###"],
        _ => ["...", "...", "...", "...", "..."],
    }
}

/// Render `text` as chunky pixel letters; `color(x_frac)` gives the gradient.
/// Returns the pixmap (3 rows tall in terminal cells).
pub fn pixel_text(text: &str, color: impl Fn(f32) -> Color, shadow: Color) -> Pixmap {
    let n = text.chars().count();
    let w = n * 4 + 1;
    let mut p = Pixmap::new(w, 6);
    for (i, ch) in text.chars().enumerate() {
        let g = glyph(ch);
        for (gy, row) in g.iter().enumerate() {
            for (gx, b) in row.chars().enumerate() {
                if b == '#' {
                    let x = i * 4 + gx;
                    // drop shadow one pixel down-right, drawn first
                    if p.get(x + 1, gy + 1).is_none() {
                        p.set(x as i32 + 1, gy as i32 + 1, Some(shadow));
                    }
                }
            }
        }
    }
    for (i, ch) in text.chars().enumerate() {
        let g = glyph(ch);
        for (gy, row) in g.iter().enumerate() {
            for (gx, b) in row.chars().enumerate() {
                if b == '#' {
                    let x = i * 4 + gx;
                    p.set(x as i32, gy as i32, Some(color(x as f32 / w as f32)));
                }
            }
        }
    }
    p
}

// ---------------------------------------------------------------------------
// Gothic (blackletter-ish) title font. Capitals are 7 px wide on rows 0–7;
// lowercase sit on rows 2–7 with descenders on row 8. 9 px tall in total.

fn gothic_glyph(c: char) -> (&'static [&'static str], usize) {
    // (rows, first row)
    match c {
        'R' => (
            &[
                "#####..", ".##.##.", ".##..##", ".##.##.", ".####..", ".##.##.", ".##..##",
                "###..##",
            ],
            0,
        ),
        'T' => (
            &[
                "#######", "#..##.#", "...##..", "...##..", "...##..", "...##..", "...##..",
                "..####.",
            ],
            0,
        ),
        'u' => (
            &["##..##", "##..##", "##..##", "##..##", "##.###", ".##.##"],
            2,
        ),
        's' => (
            &[".####.", "##..#.", ".###..", "..###.", ".#..##", ".####."],
            2,
        ),
        't' => (
            &[
                "..#...", ".##...", "#####.", ".##...", ".##...", ".##...", ".##..#", "..###.",
            ],
            0,
        ),
        'y' => (
            &[
                "##..##", "##..##", "##..##", ".#####", "....##", "#..##.", ".###..",
            ],
            2,
        ),
        'a' => (
            &[".###..", "...##.", ".####.", "##.##.", "##.##.", ".##.##"],
            2,
        ),
        'r' => (
            &["##.##.", "###..#", "##....", "##....", "##....", "###..."],
            2,
        ),
        'o' => (
            &[".###..", "##.##.", "##..##", "##..##", ".##.##", "..###."],
            2,
        ),
        _ => (&[], 0),
    }
}

/// Render `text` in the gothic pixel face. Capitals use `initial`, the rest `body`.
pub fn gothic_text(text: &str, initial: Color, body: Color, shadow: Color) -> Pixmap {
    let mut x = 0usize;
    let mut placed = vec![];
    for c in text.chars() {
        if c == ' ' {
            x += 4;
            continue;
        }
        let (rows, top) = gothic_glyph(c);
        let w = rows.first().map(|r| r.len()).unwrap_or(4);
        placed.push((x, top, rows, c.is_uppercase()));
        x += w + 1;
    }
    let mut p = Pixmap::new(x + 1, 10);
    for pass in 0..2 {
        for &(gx, top, rows, cap) in &placed {
            for (ry, row) in rows.iter().enumerate() {
                for (rx, b) in row.chars().enumerate() {
                    if b != '#' {
                        continue;
                    }
                    let (px, py) = ((gx + rx) as i32, (top + ry) as i32);
                    if pass == 0 {
                        p.set(px + 1, py + 1, Some(shadow));
                    } else {
                        p.set(px, py, Some(if cap { initial } else { body }));
                    }
                }
            }
        }
    }
    p
}

// ---------------------------------------------------------------------------
// 5×7 capitals for illuminated initials.

fn cap_glyph(c: char) -> [&'static str; 7] {
    match c.to_ascii_uppercase() {
        'A' => [
            ".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'B' => [
            "####.", "#...#", "#...#", "####.", "#...#", "#...#", "####.",
        ],
        'C' => [
            ".###.", "#...#", "#....", "#....", "#....", "#...#", ".###.",
        ],
        'D' => [
            "####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####.",
        ],
        'E' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#####",
        ],
        'F' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#....",
        ],
        'G' => [
            ".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####",
        ],
        'H' => [
            "#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'I' => [
            ".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###.",
        ],
        'J' => [
            "..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##..",
        ],
        'K' => [
            "#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#",
        ],
        'L' => [
            "#....", "#....", "#....", "#....", "#....", "#....", "#####",
        ],
        'M' => [
            "#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#",
        ],
        'N' => [
            "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#",
        ],
        'O' => [
            ".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'P' => [
            "####.", "#...#", "#...#", "####.", "#....", "#....", "#....",
        ],
        'Q' => [
            ".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#",
        ],
        'R' => [
            "####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#",
        ],
        'S' => [
            ".####", "#....", "#....", ".###.", "....#", "....#", "####.",
        ],
        'T' => [
            "#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#..",
        ],
        'U' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'V' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#..",
        ],
        'W' => [
            "#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#.",
        ],
        'X' => [
            "#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#",
        ],
        'Y' => [
            "#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#..",
        ],
        'Z' => [
            "#####", "....#", "...#.", "..#..", ".#...", "#....", "#####",
        ],
        _ => ["....."; 7],
    }
}

/// An illuminated initial: a red capital (scaled ×`scale`) on a cream tile with
/// a gold frame and corner studs — like the A/B/C/D keys of a medieval sampler.
pub fn illuminated(
    c: char,
    scale: usize,
    red: Color,
    cream: Color,
    gold: Color,
    vine: Color,
) -> Pixmap {
    let scale = scale.max(1);
    let (gw, gh) = (5 * scale, 7 * scale);
    let pad = 2;
    let (w, h) = (gw + 2 * pad + 2, gh + 2 * pad + 2);
    let mut p = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            p.set(x as i32, y as i32, Some(if edge { gold } else { cream }));
        }
    }
    // little vine flourishes in the margins
    for i in (2..h - 2).step_by(3) {
        p.set(1, i as i32, Some(vine));
        p.set((w - 2) as i32, (h - 1 - i) as i32, Some(vine));
    }
    for (x, y) in [(1, 1), (w - 2, 1), (1, h - 2), (w - 2, h - 2)] {
        p.set(x as i32, y as i32, Some(gold));
    }
    let g = cap_glyph(c);
    for (ry, row) in g.iter().enumerate() {
        for (rx, b) in row.chars().enumerate() {
            if b == '#' {
                for sy in 0..scale {
                    for sx in 0..scale {
                        p.set(
                            (1 + pad + rx * scale + sx) as i32,
                            (1 + pad + ry * scale + sy) as i32,
                            Some(red),
                        );
                    }
                }
            }
        }
    }
    p
}
