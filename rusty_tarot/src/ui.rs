// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Rendering, styled after a medieval-skinned pocket sampler: a khaki chassis,
//! a dark green name-plate with a speaker grille, key-cap tabs with Latin
//! labels, a dark display the cards are dealt onto, and a parchment page for
//! the reading — rubricated headings and illuminated initials.

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
};

use crate::app::{App, Mode, Tab};
use crate::art::{ArtCache, BACK};
use crate::deck::{self, Kind, ROMAN, SUITS, suit_info};
use crate::gfx::Gfx;
use crate::pixel::{Pixmap, gothic_text, illuminated};
use crate::spreads::{SPREADS, Slot, Spread};

// chassis & controls
const CHASSIS: Color = Color::Rgb(178, 166, 108);
const ENGRAVE: Color = Color::Rgb(66, 54, 30);
const PLATE: Color = Color::Rgb(36, 50, 42);
const PLATE_TEXT: Color = Color::Rgb(232, 224, 200);
const PLATE_DIM: Color = Color::Rgb(150, 160, 140);
const GRILLE: Color = Color::Rgb(70, 38, 28);
const GRILLE_HOLE: Color = Color::Rgb(28, 14, 10);
const KEY_BROWN: Color = Color::Rgb(84, 38, 30);
const KEY_CREAM: Color = Color::Rgb(234, 226, 204);
const KEY_RED: Color = Color::Rgb(204, 50, 32);
const KEY_GREEN: Color = Color::Rgb(32, 50, 40);
const KEY_LIGHT: Color = Color::Rgb(242, 234, 214);
const KEY_DARK: Color = Color::Rgb(52, 34, 24);
// display (the table)
const DISPLAY: Color = Color::Rgb(34, 22, 19);
const DISPLAY_DOT: Color = Color::Rgb(58, 40, 32);
const LED: Color = Color::Rgb(255, 78, 44);
const LED_DIM: Color = Color::Rgb(120, 52, 36);
const CARD_BG: Color = Color::Rgb(26, 17, 14);
const SLOT_EDGE: Color = Color::Rgb(86, 60, 46);
// parchment page
const PARCH: Color = Color::Rgb(238, 228, 200);
const PARCH_EDGE: Color = Color::Rgb(150, 118, 70);
const CHIP: Color = Color::Rgb(222, 206, 168);
const INK: Color = Color::Rgb(48, 32, 22);
const INK_DIM: Color = Color::Rgb(112, 90, 66);
const RUBRIC: Color = Color::Rgb(176, 36, 24);
const GOLD: Color = Color::Rgb(196, 150, 50);
const VINE: Color = Color::Rgb(74, 120, 60);

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let buf = f.buffer_mut();
    fill(buf, area, CHASSIS);

    let [strip, plate, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    app.hitboxes.clear();
    app.tab_boxes.clear();
    app.gfx.begin_frame();
    app.gfx.paused = app.show_help;

    draw_strip(buf, strip, app);
    draw_plate(buf, plate, app);

    let body = body.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let show_panel = body.width >= 72;
    let (table, panel) = if show_panel {
        let panel_w = (body.width * 2 / 5).clamp(36, 58);
        let [t, _, p] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(panel_w),
        ])
        .areas(body);
        (t, p)
    } else {
        (body, Rect::default())
    };
    app.panel_rect = panel;

    match app.mode {
        Mode::Reading => draw_table(buf, table, app),
        Mode::Browse => draw_browse(buf, table, app),
    }
    if show_panel {
        draw_panel(buf, panel, app);
    }
    draw_footer(buf, footer, app);
    if app.show_help {
        draw_help(buf, area);
    }
    app.gfx.end_frame();
}

fn fill(buf: &mut Buffer, area: Rect, bg: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                c.reset();
                c.set_bg(bg);
            }
        }
    }
}

fn roman(n: usize) -> &'static str {
    ROMAN.get(n).copied().unwrap_or("?")
}

// ---------------------------------------------------------------------------
// top strip, name-plate, footer

/// A key cap: ` label ` on a coloured cap.
fn keycap(buf: &mut Buffer, x: u16, y: u16, label: &str, cap: Color, text: Color) -> u16 {
    let s = format!(" {label} ");
    buf.set_string(x, y, &s, Style::new().fg(text).bg(cap));
    s.chars().count() as u16
}

fn draw_strip(buf: &mut Buffer, area: Rect, app: &mut App) {
    let y = area.y;
    let mut x = area.x + 2;
    for (i, s) in SPREADS.iter().enumerate() {
        let active = app.mode == Mode::Reading && app.spread_idx == i;
        let label = format!("{} {}", roman(i + 1), s.tab);
        let w = label.chars().count() as u16 + 2;
        if x + w > area.right() {
            break;
        }
        let cap = if active { KEY_RED } else { KEY_BROWN };
        keycap(buf, x, y, &label, cap, KEY_LIGHT);
        app.tab_boxes.push((Rect::new(x, y, w, 1), Tab::Spread(i)));
        x += w + 1;
    }
    x += 1;
    let codex = "codex";
    let w = codex.len() as u16 + 2;
    if x + w <= area.right() {
        let cap = if app.mode == Mode::Browse {
            KEY_RED
        } else {
            KEY_GREEN
        };
        keycap(buf, x, y, codex, cap, KEY_LIGHT);
        app.tab_boxes.push((Rect::new(x, y, w, 1), Tab::Browse));
        x += w + 1;
    }
    // "nova lectio" on the right, like the ignition switch
    let nova = "nova lectio";
    let w = nova.len() as u16 + 2;
    let nx = area.right().saturating_sub(w + 3);
    if app.mode == Mode::Reading && nx > x {
        keycap(buf, nx, y, nova, KEY_CREAM, KEY_DARK);
        buf.set_string(nx + w + 1, y, "▀", Style::new().fg(KEY_RED).bg(CHASSIS));
        app.tab_boxes
            .push((Rect::new(nx, y, w, 1), Tab::NewReading));
    }
}

fn grille(w: usize, h: usize) -> Pixmap {
    let mut p = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let hole = x % 2 == 1 && y % 2 == 1 && x + 1 < w && y + 1 < h;
            p.set(
                x as i32,
                y as i32,
                Some(if hole { GRILLE_HOLE } else { GRILLE }),
            );
        }
    }
    p
}

fn draw_plate(buf: &mut Buffer, area: Rect, app: &App) {
    let area = area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });
    if area.width < 10 {
        return;
    }
    // speaker grille on the right
    let gw = (area.width / 6).clamp(8, 24);
    let plate = Rect::new(area.x, area.y, area.width - gw, area.height);
    let grille_r = Rect::new(plate.right(), area.y, gw, area.height);
    fill(buf, plate, PLATE);
    grille(gw as usize, area.height as usize * 2)
        .render(buf, grille_r.x, grille_r.y, GRILLE, grille_r);
    // plate screws
    for (x, y) in [
        (plate.x, plate.y),
        (plate.right() - 1, plate.y),
        (plate.x, plate.bottom() - 1),
        (plate.right() - 1, plate.bottom() - 1),
    ] {
        buf.set_string(x, y, "▪", Style::new().fg(Color::Rgb(22, 30, 26)).bg(PLATE));
    }

    let title = gothic_text("Rusty Tarot", KEY_RED, PLATE_TEXT, Color::Rgb(16, 24, 20));
    let mut text_x = plate.x + 2;
    if plate.width as usize > title.w + 30 {
        title.render(buf, plate.x + 2, plate.y, PLATE, plate);
        text_x = plate.x + 2 + title.w as u16 + 3;
    } else {
        buf.set_string(
            plate.x + 2,
            plate.y + 1,
            "Rusty Tarot",
            Style::new()
                .fg(PLATE_TEXT)
                .bg(PLATE)
                .add_modifier(Modifier::BOLD),
        );
        text_x += 14;
    }
    let info = Rect::new(
        text_x,
        plate.y,
        plate.right().saturating_sub(text_x + 1),
        plate.height,
    );
    if info.width < 8 {
        return;
    }
    let (name, blurb) = match app.mode {
        Mode::Reading => {
            let s = &SPREADS[app.spread_idx];
            (s.name, s.blurb)
        }
        Mode::Browse => (
            "Codex",
            "All 78 cards — 22 Major Arcana and 56 Minor. Hover any card to read it.",
        ),
    };
    let lines = vec![
        Line::from(Span::styled(
            name,
            Style::new().fg(PLATE_TEXT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(blurb, Style::new().fg(PLATE_DIM))),
    ];
    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .render(Rect::new(info.x, info.y + 1, info.width, 3), buf);
    buf.set_string(
        info.x,
        plate.bottom() - 1,
        "instrumentum divinationis",
        Style::new().fg(PLATE_TEXT).bg(PLATE),
    );
}

fn draw_footer(buf: &mut Buffer, area: Rect, app: &App) {
    let y = area.y;
    let mut x = area.x + 2;
    let items: [(&str, &str); 8] = [
        ("⏎", "verte"),
        ("f", "omnes"),
        ("n", "nova"),
        ("1-5", "dispositio"),
        ("b", "codex"),
        ("g", "imago"),
        ("?", "auxilium"),
        ("q", "exi"),
    ];
    for (k, label) in items {
        let need = k.chars().count() as u16 + 3 + label.len() as u16 + 2;
        if x + need > area.right().saturating_sub(26) {
            break;
        }
        x += keycap(buf, x, y, k, KEY_BROWN, KEY_LIGHT);
        buf.set_string(x + 1, y, label, Style::new().fg(ENGRAVE).bg(CHASSIS));
        x += label.len() as u16 + 3;
    }
    // indicator lamps, like the little LEDs under the device's keys
    let led = |on: bool| if on { LED } else { Color::Rgb(120, 104, 64) };
    let mut rx = area.right().saturating_sub(24);
    if rx > x {
        for (label, on) in [
            ("inversae", app.allow_reversed),
            ("sponte", app.auto_reveal),
        ] {
            buf.set_string(rx, y, "●", Style::new().fg(led(on)).bg(CHASSIS));
            buf.set_string(rx + 2, y, label, Style::new().fg(ENGRAVE).bg(CHASSIS));
            rx += label.len() as u16 + 4;
        }
    }
}

// ---------------------------------------------------------------------------
// the display

fn display_bg(buf: &mut Buffer, area: Rect) {
    fill(buf, area, DISPLAY);
    for y in area.top()..area.bottom() {
        for x in (area.left()..area.right()).step_by(2) {
            buf.set_string(x, y, "·", Style::new().fg(DISPLAY_DOT).bg(DISPLAY));
        }
    }
}

// ---------------------------------------------------------------------------
// cards

/// Card proportions: art height÷width, and terminal cell width÷height.
#[derive(Clone, Copy)]
pub struct Shape {
    pub ar: f32,
    pub cr: f32,
}

impl Shape {
    pub fn of(app: &App) -> Self {
        Shape {
            ar: app.art.aspect,
            cr: app.gfx.cell_ratio(),
        }
    }
}

/// Outer size (incl. border) of an upright card whose art is `aw` columns wide.
pub fn card_dims(aw: u16, sh: Shape) -> (u16, u16) {
    (aw + 2, (aw as f32 * sh.ar * sh.cr).round() as u16 + 2)
}

/// Outer size of a card lying on its side.
fn rotated_dims(aw: u16, sh: Shape) -> (u16, u16) {
    (
        (aw as f32 * sh.ar).round() as u16 + 2,
        (aw as f32 * sh.cr).round() as u16 + 2,
    )
}

pub struct CardLook {
    pub id: usize,
    pub reversed: bool,
    pub face: bool,
    pub rotated: bool,
    pub focused: bool,
    pub width_factor: f32,
    pub slot_no: Option<usize>,
}

#[allow(clippy::collapsible_if)]
/// Draw a card. `track`: count its image toward overlap detection (off for the
/// detail panel, whose art is swapped in place).
pub fn draw_card(
    buf: &mut Buffer,
    rect: Rect,
    look: &CardLook,
    art: &ArtCache,
    gfx: &mut Gfx,
    clip: Rect,
    track: bool,
) {
    let mut r = rect;
    if look.width_factor < 0.999 {
        let w = ((rect.width as f32) * look.width_factor).round().max(2.0) as u16;
        r = Rect::new(rect.x + (rect.width - w) / 2, rect.y, w, rect.height);
    }
    let rc = r.intersection(clip);
    if rc.is_empty() {
        return;
    }

    let (border, bt) = if look.focused {
        (KEY_RED, BorderType::Thick)
    } else if look.face {
        (KEY_CREAM, BorderType::Plain)
    } else {
        (GOLD, BorderType::Plain)
    };
    let mut block = Block::bordered()
        .border_type(bt)
        .border_style(Style::new().fg(border).bg(CARD_BG))
        .style(Style::new().bg(CARD_BG));

    if r.width >= 6 && look.width_factor > 0.95 {
        let mut tr = vec![];
        if look.face && look.reversed {
            tr.push(Span::styled(
                "▼",
                Style::new().fg(LED).add_modifier(Modifier::BOLD),
            ));
        }
        if let Some(n) = look.slot_no {
            tr.push(Span::styled(roman(n + 1), Style::new().fg(GOLD)));
        }
        if !tr.is_empty() {
            block = block.title_top(Line::from(tr).right_aligned());
        }
        if look.face {
            block = block.title_top(
                Line::from(Span::styled(
                    deck::short_label(look.id),
                    Style::new().fg(KEY_CREAM).add_modifier(Modifier::BOLD),
                ))
                .left_aligned(),
            );
            let name = deck::text(look.id).name;
            let max = r.width.saturating_sub(2) as usize;
            let shown: String = if name.chars().count() <= max {
                name.to_string()
            } else {
                // "Nine of Wands" -> "Nine", "The Hierophant" -> "Hierophant"
                let short = name.trim_start_matches("The ");
                let short = short.split(" of ").next().unwrap_or(short);
                short.chars().take(max).collect()
            };
            block = block.title_bottom(
                Line::from(Span::styled(shown, Style::new().fg(KEY_CREAM))).centered(),
            );
        }
    }

    let inner = block.inner(r);
    block.render(rc, buf);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let idx = if look.face { look.id } else { BACK };
    let turn: u8 = match (look.rotated, look.face && look.reversed) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };
    // real image when the terminal supports it and the card is fully shown
    if look.width_factor >= 0.999 && inner.intersection(clip) == inner {
        if let Some(src) = art.full(idx) {
            if gfx.draw(buf, inner, idx, src, turn) {
                let ident = if track {
                    (idx * 4 + turn as usize + 1) as u64
                } else {
                    0
                };
                gfx.mark(inner, ident);
                return;
            }
        }
    }
    let img = art.scaled(idx, inner.width as usize, inner.height as usize * 2, turn);
    img.render(buf, inner.x, inner.y, CARD_BG, clip.intersection(inner));
}

fn draw_slot_placeholder(buf: &mut Buffer, rect: Rect, idx: usize, slot: &Slot, clip: Rect) {
    let rc = rect.intersection(clip);
    if rc.is_empty() {
        return;
    }
    let block = Block::bordered()
        .border_type(BorderType::Plain)
        .border_style(Style::new().fg(SLOT_EDGE).bg(DISPLAY))
        .style(Style::new().bg(DISPLAY));
    let inner = block.inner(rect).intersection(clip);
    block.render(rc, buf);
    let lines = vec![
        Line::from(Span::styled(
            roman(idx + 1),
            Style::new().fg(LED_DIM).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(slot.name, Style::new().fg(SLOT_EDGE))),
    ];
    let top = inner.y + inner.height.saturating_sub(2) / 2;
    let r = Rect::new(
        inner.x,
        top,
        inner.width,
        inner.bottom().saturating_sub(top),
    );
    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .render(r, buf);
}

// ---------------------------------------------------------------------------
// spread table

struct Geo {
    fits: bool,
    sh: Shape,
    aw: u16,
    cw: u16,
    ch: u16,
    uw: f32,
    uh: f32,
    ox: f32,
    oy: f32,
}

const GAP_X: u16 = 3;

fn fit(spread: &Spread, area: Rect, sh: Shape) -> Geo {
    // biggest card size that fits; squeeze out the vertical gap before shrinking
    let mut chosen = (6, 0);
    let mut fits = false;
    'search: for aw in (6u16..=64).rev() {
        for gy in [1u16, 0] {
            let (cw, ch) = card_dims(aw, sh);
            let tw = spread.w * (cw + GAP_X) as f32 - GAP_X as f32;
            let th = spread.h * (ch + gy) as f32; // gap mode keeps a row for the hover lift
            if tw <= area.width as f32 && th <= area.height as f32 {
                chosen = (aw, gy);
                fits = true;
                break 'search;
            }
        }
    }
    let (aw, gy) = chosen;
    let (cw, ch) = card_dims(aw, sh);
    let uw = (cw + GAP_X) as f32;
    let uh = (ch + gy) as f32;
    let tw = spread.w * uw - GAP_X as f32;
    let th = spread.h * uh - gy as f32;
    Geo {
        fits,
        sh,
        aw,
        cw,
        ch,
        uw,
        uh,
        ox: area.x as f32 + ((area.width as f32 - tw) / 2.0).max(0.0).floor(),
        oy: area.y as f32 + ((area.height as f32 - th) / 2.0).max(0.0).floor(),
    }
}

fn slot_rect(g: &Geo, slot: &Slot) -> Rect {
    let left = g.ox + (slot.x - 0.5) * g.uw;
    let top = g.oy + (slot.y - 0.5) * g.uh;
    if slot.rotated {
        let (rw, rh) = rotated_dims(g.aw, g.sh);
        let cx = left + g.cw as f32 / 2.0;
        let cy = top + g.ch as f32 / 2.0;
        Rect::new(
            (cx - rw as f32 / 2.0).round().max(0.0) as u16,
            (cy - rh as f32 / 2.0).round().max(0.0) as u16,
            rw,
            rh,
        )
    } else {
        Rect::new(
            left.round().max(0.0) as u16,
            top.round().max(0.0) as u16,
            g.cw,
            g.ch,
        )
    }
}

/// The display's readout line: a recording-style lamp and a count of turned cards.
fn draw_readout(buf: &mut Buffer, area: Rect, app: &App) {
    let t = app.tick;
    let n = app.drawn.len();
    let up = app.drawn.iter().filter(|d| d.face_up(t)).count();
    let waiting = up < n;
    let lamp_on = waiting && (t / 15).is_multiple_of(2);
    let x = area.x + 1;
    buf.set_string(
        x,
        area.y,
        "●",
        Style::new()
            .fg(if lamp_on { LED } else { LED_DIM })
            .bg(DISPLAY),
    );
    buf.set_string(
        x + 2,
        area.y,
        "lectio",
        Style::new().fg(LED_DIM).bg(DISPLAY),
    );
    let shown_up = if up == 0 { "–" } else { roman(up) };
    let count = format!("{shown_up} / {}", roman(n));
    buf.set_string(
        x + 9,
        area.y,
        &count,
        Style::new()
            .fg(LED)
            .bg(DISPLAY)
            .add_modifier(Modifier::BOLD),
    );
    let hint = if waiting { "⏎ verte   " } else { "" };
    let right = format!("{hint}{}", SPREADS[app.spread_idx].tab);
    let rx = area
        .right()
        .saturating_sub(right.chars().count() as u16 + 1);
    if rx > x + 20 {
        buf.set_string(rx, area.y, &right, Style::new().fg(LED_DIM).bg(DISPLAY));
    }
}

fn draw_table(buf: &mut Buffer, area: Rect, app: &mut App) {
    display_bg(buf, area);
    if area.height < 4 {
        return;
    }
    draw_readout(buf, area, app);
    let spread = &SPREADS[app.spread_idx];
    let inner = Rect::new(
        area.x + 1,
        area.y + 2,
        area.width.saturating_sub(2),
        area.height - 2,
    );
    let g = fit(spread, inner, Shape::of(app));
    let t = app.tick;

    let rects: Vec<Rect> = spread.slots.iter().map(|s| slot_rect(&g, s)).collect();
    for (i, slot) in spread.slots.iter().enumerate() {
        if !slot.rotated {
            draw_slot_placeholder(buf, rects[i], i, slot, area);
        }
    }

    for (i, slot) in spread.slots.iter().enumerate() {
        let r = rects[i];
        app.hitboxes.push((r, i));
        let Some(d) = app.drawn.get(i).copied() else {
            continue;
        };
        if !d.visible(t) {
            continue;
        }
        let focused = app.focus == Some(i);
        let (wf, face) = d.flip_state(t);
        let mut rr = r;
        // lift the hovered card, unless it shares space with another (Celtic Cross)
        let overlaps = rects
            .iter()
            .enumerate()
            .any(|(j, o)| j != i && !o.intersection(r).is_empty());
        if focused && !slot.rotated && !overlaps && rr.y > inner.y {
            rr.y -= 1;
        }
        let look = CardLook {
            id: d.card,
            reversed: d.reversed,
            face,
            rotated: slot.rotated,
            focused,
            width_factor: wf,
            slot_no: Some(i),
        };
        draw_card(buf, rr, &look, &app.art, &mut app.gfx, area, true);
    }

    if !g.fits {
        let msg = " fenestram auge — enlarge the terminal ";
        let x = area.x + area.width.saturating_sub(msg.chars().count() as u16) / 2;
        buf.set_string(
            x,
            area.bottom().saturating_sub(1),
            msg,
            Style::new().fg(LED_DIM).bg(DISPLAY),
        );
    }
}

// ---------------------------------------------------------------------------
// browse grid

fn draw_browse(buf: &mut Buffer, area: Rect, app: &mut App) {
    display_bg(buf, area);
    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let sh = Shape::of(app);
    let aw = ((inner.width + 1) / 7).saturating_sub(3).clamp(6, 14);
    let (cw, ch) = card_dims(aw, sh);
    let (gx, gy) = (1u16, 1u16);
    let cols = ((inner.width + gx) / (cw + gx)).max(1) as usize;
    let rows_total = deck::DECK_SIZE.div_ceil(cols);
    let vis = ((inner.height + gy) / (ch + gy)).max(1) as usize;
    app.browse_cols = cols;
    app.browse_rows_visible = vis;
    app.browse_max_scroll = rows_total.saturating_sub(vis) as u16;
    app.browse_scroll = app.browse_scroll.min(app.browse_max_scroll);

    let grid_w = cols as u16 * (cw + gx) - gx;
    let ox = inner.x + inner.width.saturating_sub(grid_w) / 2;
    for vr in 0..vis {
        let row = app.browse_scroll as usize + vr;
        for c in 0..cols {
            let id = row * cols + c;
            if id >= deck::DECK_SIZE {
                break;
            }
            let r = Rect::new(
                ox + c as u16 * (cw + gx),
                inner.y + vr as u16 * (ch + gy),
                cw,
                ch,
            );
            let look = CardLook {
                id,
                reversed: false,
                face: true,
                rotated: false,
                focused: app.focus == Some(id),
                width_factor: 1.0,
                slot_no: None,
            };
            draw_card(buf, r, &look, &app.art, &mut app.gfx, area, true);
            app.hitboxes.push((r, id));
        }
    }

    if app.browse_max_scroll > 0 {
        let mut st = ScrollbarState::new(app.browse_max_scroll as usize + 1)
            .position(app.browse_scroll as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_style(Style::new().fg(LED_DIM))
            .track_style(Style::new().fg(DISPLAY_DOT))
            .render(area, buf, &mut st);
    }
}

// ---------------------------------------------------------------------------
// parchment page

fn rubric(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("✠ ", Style::new().fg(RUBRIC)),
        Span::styled(
            title.to_uppercase(),
            Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn chips(keys: &[&str], bg: Color, fg: Color) -> Line<'static> {
    let mut spans = vec![];
    for k in keys {
        spans.push(Span::styled(format!(" {k} "), Style::new().fg(fg).bg(bg)));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

fn para(text: &str, color: Color) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), Style::new().fg(color)))
}

fn orientation_badge(reversed: bool) -> Line<'static> {
    let (label, cap, english) = if reversed {
        (" ▼ inversa ", KEY_RED, " reversed")
    } else {
        (" ▲ erecta ", KEY_GREEN, " upright")
    };
    Line::from(vec![
        Span::styled(
            label,
            Style::new()
                .fg(KEY_LIGHT)
                .bg(cap)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(english, Style::new().fg(INK_DIM)),
    ])
}

fn card_head_lines(id: usize, reversed: Option<bool>) -> Vec<Line<'static>> {
    let t = deck::text(id);
    let mut v = vec![
        Line::from(Span::styled(
            t.name.to_string(),
            Style::new().fg(INK).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            deck::subtitle(id),
            Style::new().fg(deck::accent(id)),
        )),
        Line::raw(""),
    ];
    if let Some(r) = reversed {
        v.push(orientation_badge(r));
        v.push(Line::raw(""));
        let keys = if r { t.rev_keys } else { t.up_keys };
        v.push(chips(keys, CHIP, INK));
    }
    v
}

fn card_body_lines(
    id: usize,
    reversed: Option<bool>,
    slot: Option<(usize, &Slot)>,
    include_head: bool,
) -> Vec<Line<'static>> {
    let t = deck::text(id);
    let mut v = vec![];
    if include_head {
        v.extend(card_head_lines(id, reversed));
        v.push(Line::raw(""));
    }
    if let Some((i, s)) = slot {
        v.push(rubric(&format!("Locus {} · {}", roman(i + 1), s.name)));
        v.push(Line::from(Span::styled(
            s.meaning.to_string(),
            Style::new().fg(INK_DIM).add_modifier(Modifier::ITALIC),
        )));
        v.push(Line::raw(""));
    }
    match reversed {
        Some(rev) => {
            v.push(rubric(if slot.is_some() {
                "In this position"
            } else {
                "Meaning"
            }));
            v.push(para(if rev { t.reversed } else { t.upright }, INK));
            v.push(Line::raw(""));
            v.push(rubric(if rev {
                "Were it upright"
            } else {
                "Were it reversed"
            }));
            v.push(chips(
                if rev { t.up_keys } else { t.rev_keys },
                CHIP,
                INK_DIM,
            ));
            v.push(para(if rev { t.upright } else { t.reversed }, INK_DIM));
        }
        None => {
            v.push(orientation_badge(false));
            v.push(chips(t.up_keys, CHIP, INK));
            v.push(para(t.upright, INK));
            v.push(Line::raw(""));
            v.push(orientation_badge(true));
            v.push(chips(t.rev_keys, CHIP, INK));
            v.push(para(t.reversed, INK));
        }
    }
    v.push(Line::raw(""));
    v.push(rubric("Pictura"));
    v.push(Line::from(Span::styled(
        t.imagery.to_string(),
        Style::new().fg(INK_DIM).add_modifier(Modifier::ITALIC),
    )));
    if let Kind::Minor(s, _) = deck::kind(id) {
        let si = suit_info(s);
        v.push(Line::raw(""));
        v.push(Line::from(vec![
            Span::styled(
                format!("{} {} ", si.glyph, si.name),
                Style::new().fg(si.color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({}) — {}.", si.element, si.domain),
                Style::new().fg(INK_DIM),
            ),
        ]));
    }
    v
}

fn overview_lines(app: &App) -> Vec<Line<'static>> {
    let spread = &SPREADS[app.spread_idx];
    let t = app.tick;
    let mut v = vec![
        Line::from(Span::styled(
            spread.name.to_string(),
            Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            spread.blurb.to_string(),
            Style::new().fg(INK_DIM).add_modifier(Modifier::ITALIC),
        )),
        Line::raw(""),
    ];
    let name_w = spread
        .slots
        .iter()
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(8)
        + 2;
    for (i, slot) in spread.slots.iter().enumerate() {
        let d = app.drawn[i];
        let mut spans = vec![
            Span::styled(
                format!("{:>5} ", roman(i + 1)),
                Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<name_w$}", slot.name), Style::new().fg(INK_DIM)),
        ];
        if d.face_up(t) {
            spans.push(Span::styled(
                deck::text(d.card).name.to_string(),
                Style::new()
                    .fg(deck::accent(d.card))
                    .add_modifier(Modifier::BOLD),
            ));
            if d.reversed {
                spans.push(Span::styled(" ▼", Style::new().fg(RUBRIC)));
            }
        } else {
            spans.push(Span::styled("· · ·", Style::new().fg(PARCH_EDGE)));
        }
        v.push(Line::from(spans));
    }
    v.push(Line::raw(""));
    if app.all_revealed() {
        v.push(rubric("Summa"));
        let n = app.drawn.len();
        let majors = app
            .drawn
            .iter()
            .filter(|d| matches!(deck::kind(d.card), Kind::Major(_)))
            .count();
        let revs = app.drawn.iter().filter(|d| d.reversed).count();
        let mut counts = [0usize; 4];
        for d in &app.drawn {
            if let Kind::Minor(s, _) = deck::kind(d.card) {
                counts[SUITS.iter().position(|x| *x == s).unwrap()] += 1;
            }
        }
        let maj_note = if n > 1 && majors * 2 >= n {
            "— major forces and life lessons are at work."
        } else if majors == 0 {
            "— this is about everyday matters within your control."
        } else {
            ""
        };
        v.push(Line::from(vec![
            Span::styled("Major Arcana ", Style::new().fg(deck::MAJOR_COLOR)),
            Span::styled(format!("{majors} of {n} "), Style::new().fg(INK)),
            Span::styled(maj_note, Style::new().fg(INK_DIM)),
        ]));
        let best = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .map(|(i, c)| (i, *c));
        if let Some((i, c)) = best {
            let tie = counts.iter().filter(|x| **x == c).count() > 1;
            if c >= 2 && !tie {
                let si = suit_info(SUITS[i]);
                v.push(Line::from(vec![
                    Span::styled(
                        format!("{} {} lead ", si.glyph, si.name),
                        Style::new().fg(si.color),
                    ),
                    Span::styled(
                        format!("— a focus on {}.", si.domain),
                        Style::new().fg(INK_DIM),
                    ),
                ]));
            }
        }
        if n > 1 {
            let rev_note = if revs * 2 > n {
                "— energy is blocked or turned inward; look at what you're resisting."
            } else if revs == 0 {
                "— energy flows freely."
            } else {
                ""
            };
            v.push(Line::from(vec![
                Span::styled("Reversed ", Style::new().fg(RUBRIC)),
                Span::styled(format!("{revs} of {n} "), Style::new().fg(INK)),
                Span::styled(rev_note, Style::new().fg(INK_DIM)),
            ]));
        }
        v.push(Line::raw(""));
        v.push(para(
            "Hover over a card to read it in its position.",
            INK_DIM,
        ));
    } else {
        v.push(para(
            "Press ⏎ to turn the next card, click any face-down card, or press f to turn them all.",
            INK_DIM,
        ));
    }
    v
}

fn browse_intro_lines(app: &App) -> Vec<Line<'static>> {
    let mut v = vec![
        Line::from(Span::styled(
            "Codex",
            Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "78 cards in the Rider–Waite–Smith tradition.",
            Style::new().fg(INK_DIM).add_modifier(Modifier::ITALIC),
        )),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "22 Major Arcana ",
                Style::new()
                    .fg(deck::MAJOR_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— the Fool's journey through life's great archetypes, from The Fool to The World.",
                Style::new().fg(INK),
            ),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "56 Minor Arcana ",
                Style::new().fg(INK).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— everyday life, in four suits of Ace–Ten plus four court cards:",
                Style::new().fg(INK),
            ),
        ]),
        Line::raw(""),
    ];
    for s in SUITS {
        let si = suit_info(s);
        v.push(Line::from(vec![
            Span::styled(
                format!(" {} {:<10}", si.glyph, si.name),
                Style::new().fg(si.color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<6} ", si.element), Style::new().fg(INK_DIM)),
            Span::styled(si.domain.to_string(), Style::new().fg(INK)),
        ]));
    }
    v.push(Line::raw(""));
    v.push(para(
        "Hover any card, or use the arrow keys, to read its upright and reversed meanings.",
        INK_DIM,
    ));
    v.push(Line::raw(""));
    v.push(rubric("Imagines"));
    v.push(para(&app.art_status, INK_DIM));
    v.push(para(
        &format!("Drawing with {}.", app.gfx.describe()),
        INK_DIM,
    ));
    v.push(para(
        "Edit the PNGs and press F5 to reload. Ctrl+L repaints the screen.",
        INK_DIM,
    ));
    v
}

/// The illuminated initial for a card name ("The Fool" → F).
fn initial_of(name: &str) -> char {
    name.trim_start_matches("The ")
        .chars()
        .next()
        .unwrap_or('?')
}

fn draw_panel(buf: &mut Buffer, area: Rect, app: &mut App) {
    if app.mode == Mode::Reading && app.focus.is_some_and(|i| i >= app.drawn.len()) {
        app.focus = None;
    }
    let title = match (app.mode, app.focus) {
        (Mode::Reading, Some(i)) => format!(
            " ✠ {} · {} ",
            roman(i + 1),
            SPREADS[app.spread_idx].slots[i].name
        ),
        (Mode::Reading, None) => " ✠ Lectio ".to_string(),
        (Mode::Browse, Some(id)) => format!(" ✠ {} ", deck::short_label(id)),
        (Mode::Browse, None) => " ✠ Codex ".to_string(),
    };
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(PARCH_EDGE).bg(PARCH))
        .style(Style::new().bg(PARCH))
        .title(Line::from(Span::styled(
            title,
            Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
        )));
    let inner = block.inner(area).inner(Margin {
        horizontal: 1,
        vertical: 0,
    });
    block.render(area, buf);
    if inner.width < 4 || inner.height < 2 {
        return;
    }

    // Which card (if any) and in what orientation?
    let t = app.tick;
    let mut card: Option<(usize, Option<bool>)> = None;
    let mut slot_info: Option<(usize, &Slot)> = None;
    let mut face_down = false;
    match (app.mode, app.focus) {
        (Mode::Reading, Some(i)) => {
            let slot = &SPREADS[app.spread_idx].slots[i];
            slot_info = Some((i, slot));
            let d = app.drawn[i];
            if d.face_up(t) {
                card = Some((d.card, Some(d.reversed)));
            } else {
                face_down = true;
            }
        }
        (Mode::Browse, Some(id)) => card = Some((id, None)),
        _ => {}
    }

    let mut body_area = inner;
    let lines: Vec<Line> = if let Some((id, rev)) = card {
        // as big as fits beside the title column, using up to ~60% of the height
        let sh = Shape::of(app);
        let by_h = ((inner.height as f32 * 0.6 - 2.0) / (sh.ar * sh.cr)).max(0.0) as u16;
        let aw = (inner.width / 2).saturating_sub(2).min(by_h).min(40);
        let (acw, ach) = card_dims(aw, sh);
        let with_art = aw >= 8 && inner.height >= ach + 10 && inner.width >= acw + 16;
        if with_art {
            let art_rect = Rect::new(inner.x, inner.y + 1, acw, ach);
            let look = CardLook {
                id,
                reversed: rev.unwrap_or(false),
                face: true,
                rotated: false,
                focused: false,
                width_factor: 1.0,
                slot_no: None,
            };
            draw_card(buf, art_rect, &look, &app.art, &mut app.gfx, inner, false);
            let ix = inner.x + acw + 2;
            let iw = inner.width - acw - 2;
            // illuminated initial above the name
            let scale = if iw >= 26 && ach >= 22 { 2 } else { 1 };
            let cap = illuminated(
                initial_of(deck::text(id).name),
                scale,
                RUBRIC,
                KEY_CREAM,
                GOLD,
                VINE,
            );
            let cap_rows = cap.h.div_ceil(2) as u16;
            let head_y = if (cap.w as u16) < iw && cap_rows + 6 < ach {
                cap.render(buf, ix, inner.y + 1, PARCH, inner);
                inner.y + 1 + cap_rows + 1
            } else {
                inner.y + 1
            };
            let info = Rect::new(ix, head_y, iw, (inner.y + 1 + ach).saturating_sub(head_y));
            Paragraph::new(card_head_lines(id, rev))
                .wrap(Wrap { trim: false })
                .render(info, buf);
            body_area = Rect::new(
                inner.x,
                inner.y + ach + 2,
                inner.width,
                inner.height - ach - 2,
            );
        }
        card_body_lines(id, rev, slot_info, !with_art)
    } else if face_down {
        let (i, s) = slot_info.unwrap();
        vec![
            Line::raw(""),
            Line::from(Span::styled(
                "Charta tecta — face down",
                Style::new().fg(INK).add_modifier(Modifier::BOLD),
            )),
            para("Click the card, or press ⏎, to turn it over.", INK_DIM),
            Line::raw(""),
            rubric(&format!("Locus {} · {}", roman(i + 1), s.name)),
            Line::from(Span::styled(
                s.meaning.to_string(),
                Style::new().fg(INK_DIM).add_modifier(Modifier::ITALIC),
            )),
        ]
    } else if app.mode == Mode::Reading {
        let mut v = vec![Line::raw("")];
        v.extend(overview_lines(app));
        v
    } else {
        let mut v = vec![Line::raw("")];
        v.extend(browse_intro_lines(app));
        v
    };

    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    let total = p.line_count(body_area.width) as u16;
    app.panel_max_scroll = total.saturating_sub(body_area.height);
    app.panel_scroll = app.panel_scroll.min(app.panel_max_scroll);
    p.scroll((app.panel_scroll, 0)).render(body_area, buf);

    if app.panel_max_scroll > 0 {
        let mut st = ScrollbarState::new(app.panel_max_scroll as usize + 1)
            .position(app.panel_scroll as usize);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_style(Style::new().fg(RUBRIC))
            .track_style(Style::new().fg(PARCH_EDGE))
            .render(
                Rect::new(area.right() - 1, body_area.y, 1, body_area.height),
                buf,
                &mut st,
            );
    }
}

// ---------------------------------------------------------------------------
// help

fn draw_help(buf: &mut Buffer, area: Rect) {
    let w = 64.min(area.width);
    let h = 25.min(area.height);
    let r = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    Clear.render(r, buf);
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(PARCH_EDGE).bg(PARCH))
        .style(Style::new().bg(PARCH))
        .title(
            Line::from(Span::styled(
                " ✠ Regulae — how to read ✠ ",
                Style::new().fg(RUBRIC).add_modifier(Modifier::BOLD),
            ))
            .centered(),
        );
    let k = |key: &'static str, latin: &'static str, d: &'static str| {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {key} "), Style::new().fg(KEY_LIGHT).bg(KEY_BROWN)),
            Span::raw(" ".repeat(11usize.saturating_sub(key.chars().count() + 2))),
            Span::styled(
                format!("{latin:<11}"),
                Style::new().fg(RUBRIC).add_modifier(Modifier::ITALIC),
            ),
            Span::styled(d, Style::new().fg(INK)),
        ])
    };
    let lines = vec![
        Line::raw(""),
        k("mouse", "manus", "hover to read a card, click to turn it"),
        k("← → tab", "move", "select the previous / next card"),
        k("⏎ space", "verte", "turn the selected card, or the next"),
        k("f", "omnes", "turn every card over"),
        k("esc", "redi", "back to the reading overview"),
        k("n", "nova", "shuffle and deal a new reading"),
        k(
            "1 – 5",
            "dispositio",
            "unus · tres · crux · solea · celtica",
        ),
        k("b", "codex", "browse all 78 cards"),
        k("r", "inversae", "allow reversed cards (next deal)"),
        k("a", "sponte", "turn cards automatically as dealt"),
        k("g", "imago", "real images / pixel blocks"),
        k("F5", "renova", "reload card PNGs from disk"),
        k("ctrl+l", "purga", "repaint the whole screen"),
        k("pgup/dn", "volve", "scroll the page (or mouse wheel)"),
        k("q", "exi", "quit"),
        Line::raw(""),
        Line::from(Span::styled(
            "  Reversed cards (▼) lie upside down: their energy is",
            Style::new().fg(INK_DIM),
        )),
        Line::from(Span::styled(
            "  blocked, delayed or turned inward.",
            Style::new().fg(INK_DIM),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "press any key to close",
            Style::new().fg(PARCH_EDGE),
        ))
        .centered(),
    ];
    Paragraph::new(lines).block(block).render(r, buf);
}
