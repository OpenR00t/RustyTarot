// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Real image output (Sixel / Kitty / iTerm2) via `ratatui-image`, with the
//! half-block renderer as the fallback.
//!
//! Graphics protocols draw pixels on top of the text grid, so two rules keep the
//! screen clean:
//! * every image placed in a frame is recorded in a signature; when the
//!   signature changes (cards dealt, flipped, scrolled…) the terminal is fully
//!   cleared and redrawn so no stale pixels linger, and overlapping cards are
//!   re-sent in the right order;
//! * encoding is rate-limited per frame so opening the 78-card browser doesn't
//!   stall — cards not yet encoded show their half-block version meanwhile.

use std::collections::HashMap;

use image::imageops::{self, FilterType};
use image::{DynamicImage, RgbaImage};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::widgets::Widget;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};

/// How many new images may be encoded per frame.
const ENCODE_BUDGET: usize = 4;

pub struct Gfx {
    picker: Option<Picker>,
    /// User toggle (`g`): use images when available.
    pub enabled: bool,
    /// Temporarily draw without images (e.g. while an overlay covers the table).
    pub paused: bool,
    cache: HashMap<(usize, u16, u16, u8), Protocol>,
    budget: usize,
    sig: Vec<(Rect, u64)>,
    last_sig: Vec<(Rect, u64)>,
    /// Regions whose images must be re-sent next frame (overlaps), drawn once
    /// without images first so their cells change and get re-emitted.
    repaint: Vec<Rect>,
    /// While set, images in these regions are drawn as half-blocks instead.
    suppress: Vec<Rect>,
    /// Full-screen repaint requested (Ctrl+L).
    pub needs_clear: bool,
}

impl Gfx {
    /// Half-block rendering only.
    pub fn none() -> Self {
        Self::new(None)
    }

    pub fn new(picker: Option<Picker>) -> Self {
        Self {
            picker,
            enabled: true,
            paused: false,
            cache: HashMap::new(),
            budget: ENCODE_BUDGET,
            sig: vec![],
            last_sig: vec![],
            repaint: vec![],
            suppress: vec![],
            needs_clear: false,
        }
    }

    pub fn active(&self) -> bool {
        self.enabled && self.picker.is_some()
    }

    pub fn available(&self) -> bool {
        self.picker.is_some()
    }

    /// Terminal cell width ÷ height in pixels (0.5 assumed for half-blocks).
    pub fn cell_ratio(&self) -> f32 {
        match (&self.picker, self.enabled) {
            (Some(p), true) => {
                let f = p.font_size();
                if f.width == 0 || f.height == 0 {
                    0.5
                } else {
                    f.width as f32 / f.height as f32
                }
            }
            _ => 0.5,
        }
    }

    pub fn describe(&self) -> String {
        match &self.picker {
            None => "half-block pixels (no image protocol detected)".into(),
            Some(p) => {
                let proto = match p.protocol_type() {
                    ProtocolType::Sixel => "Sixel",
                    ProtocolType::Kitty => "Kitty",
                    ProtocolType::Iterm2 => "iTerm2",
                    ProtocolType::Halfblocks => "half-blocks",
                };
                let f = p.font_size();
                let state = if self.enabled {
                    ""
                } else {
                    " (off — press g)"
                };
                format!("{proto} images, cell {}×{} px{state}", f.width, f.height)
            }
        }
    }

    pub fn toggle(&mut self) {
        if self.picker.is_some() {
            self.enabled = !self.enabled;
        }
    }

    /// Call at the start of every frame.
    pub fn begin_frame(&mut self) {
        self.budget = ENCODE_BUDGET;
        self.sig.clear();
    }

    /// Call at the end of every frame.
    ///
    /// Images that disappear or move need no special care: the cells they
    /// covered are rewritten with text, which erases them. The one case that
    /// does: a newly placed image overlapping another (the Celtic Cross), since
    /// whichever is sent last ends up on top. Those regions are queued for a
    /// quick two-pass repaint so they're re-sent in the right order.
    pub fn end_frame(&mut self) {
        if self.sig != self.last_sig {
            let mut dirty = vec![];
            for (i, new) in self.sig.iter().enumerate() {
                if self.last_sig.contains(new) {
                    continue;
                }
                for (j, other) in self.sig.iter().enumerate() {
                    if i != j && !new.0.intersection(other.0).is_empty() {
                        dirty.push(new.0);
                        dirty.push(other.0);
                    }
                }
            }
            self.repaint = dirty;
            self.last_sig = std::mem::take(&mut self.sig);
        }
    }

    /// Regions to repaint in two passes (see [`Gfx::end_frame`]); empty if none.
    pub fn take_repaint(&mut self) -> Vec<Rect> {
        std::mem::take(&mut self.repaint)
    }

    pub fn set_suppress(&mut self, rects: Vec<Rect>) {
        self.suppress = rects;
    }

    /// True once after a full repaint was requested.
    pub fn take_clear(&mut self) -> bool {
        std::mem::take(&mut self.needs_clear)
    }

    /// Record a region that holds an image for change detection. `ident` should
    /// change whenever a *different* image would appear there and the old one could
    /// be left partly visible (pass 0 for slots where a new image fully covers the old).
    pub fn mark(&mut self, area: Rect, ident: u64) {
        self.sig.push((area, ident));
    }

    /// Draw image `idx` (rotated by `turn` quarter turns clockwise) filling `area`.
    /// Returns false if it couldn't be drawn this frame (caller draws a fallback).
    pub fn draw(
        &mut self,
        buf: &mut Buffer,
        area: Rect,
        idx: usize,
        src: &RgbaImage,
        turn: u8,
    ) -> bool {
        let Some(picker) = self
            .picker
            .as_ref()
            .filter(|_| self.enabled && !self.paused)
        else {
            return false;
        };
        if area.width == 0 || area.height == 0 || area.intersection(buf.area) != area {
            return false;
        }
        if self
            .suppress
            .iter()
            .any(|r| !r.intersection(area).is_empty())
        {
            return false;
        }
        let key = (idx, area.width, area.height, turn);
        if !self.cache.contains_key(&key) {
            if self.budget == 0 {
                return false;
            }
            self.budget -= 1;
            let font = picker.font_size();
            let (pw, ph) = (
                area.width as u32 * font.width as u32,
                area.height as u32 * font.height as u32,
            );
            let img = match turn {
                1 => imageops::rotate90(src),
                2 => imageops::rotate180(src),
                3 => imageops::rotate270(src),
                _ => src.clone(),
            };
            let img = scale_pixel_art(&img, pw.max(1), ph.max(1));
            match picker.new_protocol(
                DynamicImage::ImageRgba8(img),
                Size::new(area.width, area.height),
                Resize::Fit(None),
            ) {
                Ok(p) => {
                    if self.cache.len() > 300 {
                        self.cache.clear();
                    }
                    self.cache.insert(key, p);
                }
                Err(_) => return false,
            }
        }
        Image::new(&self.cache[&key]).render(area, buf);
        true
    }
}

/// Build a picker according to the `--graphics` choice:
/// `auto` (default), `sixel`, `kitty`, `iterm2`, or `blocks`.
/// `font` overrides the detected cell size in pixels.
///
/// Must be called after entering the alternate screen, before reading events.
pub fn detect(choice: &str, font: Option<(u16, u16)>) -> Option<Picker> {
    let forced = match choice {
        "blocks" | "halfblocks" | "none" | "off" => return None,
        "sixel" => Some(ProtocolType::Sixel),
        "kitty" => Some(ProtocolType::Kitty),
        "iterm" | "iterm2" => Some(ProtocolType::Iterm2),
        _ => None,
    };
    // Windows Terminal: ConPTY often swallows the reply to the capability query,
    // and the query thread would then keep eating keystrokes. WT (1.22+) speaks
    // Sixel with a fixed 10×20 px cell, so skip the query and use that.
    let windows_terminal = std::env::var_os("WT_SESSION").is_some();
    if windows_terminal && choice == "auto" {
        #[allow(deprecated)]
        let mut p = Picker::from_fontsize(font.unwrap_or((10, 20)).into());
        p.set_protocol_type(ProtocolType::Sixel);
        p.set_background_color(Some(image::Rgba([30u8, 22, 46, 255])));
        return Some(p);
    }
    let queried = if !windows_terminal && (font.is_none() || forced.is_none()) {
        Picker::from_query_stdio().ok()
    } else {
        None
    };
    let mut picker = match font {
        #[allow(deprecated)]
        Some(f) => {
            let mut p = Picker::from_fontsize(f.into());
            if let Some(q) = &queried {
                p.set_protocol_type(q.protocol_type());
            }
            p
        }
        None => queried.unwrap_or_else(Picker::halfblocks),
    };
    match forced {
        Some(p) => picker.set_protocol_type(p),
        None => {
            // Windows Terminal (1.22+) speaks Sixel, but ConPTY often swallows the
            // reply to the capability query, so detection comes back empty.
            if picker.protocol_type() == ProtocolType::Halfblocks
                && std::env::var_os("WT_SESSION").is_some()
            {
                picker.set_protocol_type(ProtocolType::Sixel);
            }
        }
    }
    picker.set_background_color(Some(image::Rgba([30u8, 22, 46, 255])));
    (picker.protocol_type() != ProtocolType::Halfblocks).then_some(picker)
}

/// Scale pixel art to exactly `w`×`h`: enlarge by the largest whole factor with
/// nearest-neighbour (crisp pixels), then a gentle bilinear step for the small
/// remainder — "sharp bilinear", so pixels stay square-edged at any size.
pub fn scale_pixel_art(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let (sw, sh) = (img.width().max(1), img.height().max(1));
    let k = (w / sw).min(h / sh);
    if k >= 1 {
        let up = if k > 1 {
            imageops::resize(img, sw * k, sh * k, FilterType::Nearest)
        } else {
            img.clone()
        };
        if up.width() == w && up.height() == h {
            return up;
        }
        imageops::resize(&up, w, h, FilterType::Triangle)
    } else {
        imageops::resize(img, w, h, FilterType::Triangle)
    }
}
