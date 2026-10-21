// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Application state and input handling.

use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};
use std::path::PathBuf;

use crate::art::ArtCache;
use crate::deck::DECK_SIZE;
use crate::gfx::Gfx;
use crate::rng::Rng;
use crate::spreads::SPREADS;

/// Ticks a flip animation lasts (~33 ms per tick).
pub const FLIP_TICKS: u64 = 8;

#[derive(Clone, Copy, Debug)]
pub struct Drawn {
    pub card: usize,
    pub reversed: bool,
    /// Tick when the card lands on the table (face down).
    pub appear_at: u64,
    /// Tick when it starts turning over; None = still face down.
    pub flip_at: Option<u64>,
}

impl Drawn {
    pub fn visible(&self, t: u64) -> bool {
        t >= self.appear_at
    }

    pub fn face_up(&self, t: u64) -> bool {
        matches!(self.flip_at, Some(f) if t >= f + FLIP_TICKS)
    }

    /// (width factor 0..1, showing face?) for the flip animation.
    pub fn flip_state(&self, t: u64) -> (f32, bool) {
        match self.flip_at {
            Some(f) if t >= f => {
                let p = ((t - f) as f32 / FLIP_TICKS as f32).min(1.0);
                let w = (p * std::f32::consts::PI).cos().abs();
                (w.max(0.08), p >= 0.5)
            }
            _ => (1.0, false),
        }
    }

    pub fn flipping_or_up(&self, t: u64) -> bool {
        matches!(self.flip_at, Some(f) if t >= f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Reading,
    Browse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Spread(usize),
    Browse,
    NewReading,
}

pub struct App {
    pub tick: u64,
    pub rng: Rng,
    pub art: ArtCache,
    pub gfx: Gfx,
    pub art_dir: Option<PathBuf>,
    /// One-line summary of where the card art came from (shown in browse mode).
    pub art_status: String,
    pub mode: Mode,
    pub spread_idx: usize,
    pub drawn: Vec<Drawn>,
    pub allow_reversed: bool,
    pub auto_reveal: bool,
    /// Slot index in Reading mode, card id in Browse mode.
    pub focus: Option<usize>,
    pub mouse: Option<(u16, u16)>,
    /// Filled by the renderer each frame: (rect, slot/card), later entries on top.
    pub hitboxes: Vec<(Rect, usize)>,
    pub tab_boxes: Vec<(Rect, Tab)>,
    pub panel_rect: Rect,
    pub panel_scroll: u16,
    pub panel_max_scroll: u16,
    pub browse_scroll: u16,
    pub browse_max_scroll: u16,
    pub browse_cols: usize,
    pub browse_rows_visible: usize,
    pub show_help: bool,
    pub quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_rng(Rng::seeded())
    }

    pub fn with_rng(rng: Rng) -> Self {
        Self::with_art(rng, None)
    }

    /// `art_dir`: folder of card PNGs; missing cards fall back to the built-in art.
    pub fn with_art(rng: Rng, art_dir: Option<PathBuf>) -> Self {
        let (art, art_status) = ArtCache::load(art_dir.as_deref());
        let mut app = App {
            tick: 0,
            rng,
            art,
            gfx: Gfx::none(),
            art_dir,
            art_status,
            mode: Mode::Reading,
            spread_idx: 1,
            drawn: vec![],
            allow_reversed: true,
            auto_reveal: false,
            focus: None,
            mouse: None,
            hitboxes: vec![],
            tab_boxes: vec![],
            panel_rect: Rect::default(),
            panel_scroll: 0,
            panel_max_scroll: 0,
            browse_scroll: 0,
            browse_max_scroll: 0,
            browse_cols: 1,
            browse_rows_visible: 1,
            show_help: false,
            quit: false,
        };
        app.new_reading();
        app
    }

    pub fn new_reading(&mut self) {
        let spread = &SPREADS[self.spread_idx];
        let mut deck: Vec<usize> = (0..DECK_SIZE).collect();
        self.rng.shuffle(&mut deck);
        let start = self.tick + 2;
        self.drawn = deck
            .into_iter()
            .take(spread.slots.len())
            .enumerate()
            .map(|(i, card)| {
                let reversed = self.allow_reversed && self.rng.below(100) < 30;
                let appear_at = start + i as u64 * 3;
                let flip_at = self.auto_reveal.then(|| start + 10 + i as u64 * 7);
                Drawn {
                    card,
                    reversed,
                    appear_at,
                    flip_at,
                }
            })
            .collect();
        self.focus = None;
        self.panel_scroll = 0;
        self.mode = Mode::Reading;
        self.hitboxes.clear();
    }

    pub fn set_spread(&mut self, i: usize) {
        if i < SPREADS.len() {
            self.spread_idx = i;
            self.new_reading();
        }
    }

    /// Instantly finish dealing, as if all timers had elapsed (used for snapshots).
    pub fn settle(&mut self) {
        for d in &mut self.drawn {
            d.appear_at = 0;
            d.flip_at = Some(0);
        }
        self.tick = self.tick.max(FLIP_TICKS + 1);
    }

    pub fn reveal_all(&mut self) {
        let t = self.tick;
        let mut k = 0;
        for d in &mut self.drawn {
            if d.flip_at.is_none_or(|f| f > t + k * 4) {
                d.appear_at = d.appear_at.min(t);
                d.flip_at = Some(t + k * 4);
                k += 1;
            }
        }
    }

    pub fn flip(&mut self, slot: usize) {
        let t = self.tick;
        match self.drawn.get_mut(slot) {
            Some(d) if !d.flipping_or_up(t) => {
                d.appear_at = d.appear_at.min(t);
                d.flip_at = Some(t);
            }
            _ => {}
        }
    }

    pub fn all_revealed(&self) -> bool {
        self.drawn.iter().all(|d| d.face_up(self.tick))
    }

    fn set_focus(&mut self, f: Option<usize>) {
        // ignore stale targets (e.g. a click queued before the layout changed)
        let f = f.filter(|&i| i < self.item_count());
        if f != self.focus {
            self.focus = f;
            self.panel_scroll = 0;
        }
    }

    fn hit(&self, x: u16, y: u16) -> Option<usize> {
        let p = Position::new(x, y);
        self.hitboxes
            .iter()
            .rev()
            .find(|(r, _)| r.contains(p))
            .map(|(_, i)| *i)
    }

    fn item_count(&self) -> usize {
        match self.mode {
            Mode::Reading => self.drawn.len(),
            Mode::Browse => DECK_SIZE,
        }
    }

    fn cycle(&mut self, delta: isize) {
        let n = self.item_count() as isize;
        if n == 0 {
            return;
        }
        let next = match self.focus {
            None if delta >= 0 => 0,
            None => n - 1,
            Some(f) => (f as isize + delta).rem_euclid(n),
        };
        self.set_focus(Some(next as usize));
        self.ensure_browse_visible();
    }

    fn browse_move(&mut self, delta: isize) {
        let n = DECK_SIZE as isize;
        let next = match self.focus {
            None => 0,
            Some(f) => (f as isize + delta).clamp(0, n - 1),
        };
        self.set_focus(Some(next as usize));
        self.ensure_browse_visible();
    }

    fn ensure_browse_visible(&mut self) {
        if self.mode != Mode::Browse {
            return;
        }
        if let Some(f) = self.focus {
            let row = (f / self.browse_cols.max(1)) as u16;
            let vis = self.browse_rows_visible.max(1) as u16;
            if row < self.browse_scroll {
                self.browse_scroll = row;
            } else if row >= self.browse_scroll + vis {
                self.browse_scroll = row + 1 - vis;
            }
        }
    }

    /// Enter/Space: flip the selected card if it's face down, otherwise flip
    /// the next face-down card in spread order and select it.
    fn activate(&mut self) {
        if self.mode != Mode::Reading {
            return;
        }
        let t = self.tick;
        match self.focus {
            Some(slot) if self.drawn.get(slot).is_some_and(|d| !d.flipping_or_up(t)) => {
                self.flip(slot);
                return;
            }
            _ => {}
        }
        if let Some(next) = self.drawn.iter().position(|d| !d.flipping_or_up(t)) {
            self.flip(next);
            self.set_focus(Some(next));
        }
    }

    /// Reload card PNGs from disk (F5) so art edits show up without restarting.
    pub fn reload_art(&mut self) {
        let (art, status) = ArtCache::load(self.art_dir.as_deref());
        self.art = art;
        self.art_status = status;
    }

    fn scroll_panel(&mut self, delta: i32) {
        let v = (self.panel_scroll as i32 + delta).clamp(0, self.panel_max_scroll as i32);
        self.panel_scroll = v as u16;
    }

    pub fn on_key(&mut self, k: KeyEvent) {
        if self.show_help {
            self.show_help = false;
            if matches!(
                k.code,
                KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('h')
            ) {
                return;
            }
        }
        match k.code {
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => self.quit = true,
            KeyCode::Char('l') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                self.gfx.needs_clear = true
            }
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Esc => self.set_focus(None),
            KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
            KeyCode::Char('n') | KeyCode::Char('N') => self.new_reading(),
            KeyCode::Char(c @ '1'..='5') => self.set_spread(c as usize - '1' as usize),
            KeyCode::Char('b') | KeyCode::Char('B') => self.toggle_browse(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.allow_reversed = !self.allow_reversed,
            KeyCode::Char('a') | KeyCode::Char('A') => self.auto_reveal = !self.auto_reveal,
            KeyCode::Char(' ') | KeyCode::Enter => self.activate(),
            KeyCode::Char('f') | KeyCode::Char('F') => self.reveal_all(),
            KeyCode::F(5) => self.reload_art(),
            KeyCode::Char('g') | KeyCode::Char('G') => self.gfx.toggle(),
            KeyCode::Tab => self.cycle(1),
            KeyCode::BackTab => self.cycle(-1),
            KeyCode::Right | KeyCode::Char('l') => match self.mode {
                Mode::Reading => self.cycle(1),
                Mode::Browse => self.browse_move(1),
            },
            KeyCode::Left => match self.mode {
                Mode::Reading => self.cycle(-1),
                Mode::Browse => self.browse_move(-1),
            },
            KeyCode::Down | KeyCode::Char('j') => match self.mode {
                Mode::Reading => self.cycle(1),
                Mode::Browse => self.browse_move(self.browse_cols as isize),
            },
            KeyCode::Up | KeyCode::Char('k') => match self.mode {
                Mode::Reading => self.cycle(-1),
                Mode::Browse => self.browse_move(-(self.browse_cols as isize)),
            },
            KeyCode::PageDown => self.scroll_panel(5),
            KeyCode::PageUp => self.scroll_panel(-5),
            _ => {}
        }
    }

    fn toggle_browse(&mut self) {
        self.mode = match self.mode {
            Mode::Reading => Mode::Browse,
            Mode::Browse => Mode::Reading,
        };
        self.focus = None;
        self.panel_scroll = 0;
        self.hitboxes.clear();
    }

    pub fn on_mouse(&mut self, m: MouseEvent) {
        self.mouse = Some((m.column, m.row));
        let p = Position::new(m.column, m.row);
        match m.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                if self.panel_rect.contains(p) {
                    return; // keep showing the card while reading its details
                }
                // hovering empty table keeps the last card; Esc returns to the overview
                if let Some(h) = self.hit(m.column, m.row) {
                    self.set_focus(Some(h));
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.show_help {
                    self.show_help = false;
                    return;
                }
                if let Some((_, tab)) = self.tab_boxes.iter().find(|(r, _)| r.contains(p)).copied()
                {
                    match tab {
                        Tab::Spread(i) => self.set_spread(i),
                        Tab::Browse => self.toggle_browse(),
                        Tab::NewReading => self.new_reading(),
                    }
                    return;
                }
                if let Some(i) = self.hit(m.column, m.row) {
                    self.set_focus(Some(i));
                    if self.mode == Mode::Reading {
                        self.flip(i);
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if self.panel_rect.contains(p) {
                    self.scroll_panel(2);
                } else if self.mode == Mode::Browse {
                    self.browse_scroll = (self.browse_scroll + 1).min(self.browse_max_scroll);
                }
            }
            MouseEventKind::ScrollUp => {
                if self.panel_rect.contains(p) {
                    self.scroll_panel(-2);
                } else if self.mode == Mode::Browse {
                    self.browse_scroll = self.browse_scroll.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
}
