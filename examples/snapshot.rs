// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Dev helper: render one frame headlessly and dump the cell grid as TSV
//! (`x y symbol fg bg`) so it can be turned into a PNG screenshot.
//!
//! cargo run --example snapshot -- <cols> <rows> <spread 0-4|browse> [focus] [seed]

use ratatui::{Terminal, backend::TestBackend, style::Color};
use rusty_tarot::{
    app::{App, Mode},
    rng::Rng,
    ui,
};

fn hex(c: Color, default: &str) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("{r:02x}{g:02x}{b:02x}"),
        Color::Reset => default.to_string(),
        other => format!("{other:?}"),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let w: u16 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(160);
    let h: u16 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(48);
    let spread = a.get(3).cloned().unwrap_or("1".into());
    let focus: Option<usize> = a.get(4).and_then(|s| s.parse().ok());
    let seed: u64 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(7);

    let art_dir = std::env::var_os("RUSTY_TAROT_CARDS").map(std::path::PathBuf::from);
    let mut app = App::with_art(Rng::from_seed(seed), art_dir);
    if spread == "browse" {
        app.mode = Mode::Browse;
    } else {
        app.set_spread(spread.parse().unwrap_or(1));
    }
    if let Ok(mode) = std::env::var("SNAP_GFX") {
        // exercise the image path (output cells then hold raw escape data)
        #[allow(deprecated)]
        let mut p = ratatui_image::picker::Picker::from_fontsize((10, 20).into());
        p.set_protocol_type(match mode.as_str() {
            "kitty" => ratatui_image::picker::ProtocolType::Kitty,
            _ => ratatui_image::picker::ProtocolType::Sixel,
        });
        app.gfx = rusty_tarot::gfx::Gfx::new(Some(p));
    }
    app.settle();
    app.tick = 200;
    app.focus = focus;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    // a few frames so the per-frame image-encoding budget can finish
    for _ in 0..30 {
        term.draw(|f| ui::draw(f, &mut app)).unwrap();
    }
    let buf = term.backend().buffer();
    for y in 0..h {
        for x in 0..w {
            let c = &buf[(x, y)];
            println!(
                "{}\t{}\t{}\t{}\t{}",
                x,
                y,
                c.symbol().replace('\t', " "),
                hex(c.fg, "e8e0f5"),
                hex(c.bg, "0d0a16")
            );
        }
    }
}
