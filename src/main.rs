// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{Clear, ClearType};

use rusty_tarot::{app::App, art, gfx, rng::Rng, ui};

const FRAME: Duration = Duration::from_millis(33);

const USAGE: &str = "\
rusty_tarot — retro pixel-art tarot in your terminal

USAGE:
    rusty_tarot [--cards <DIR>] [--graphics <MODE>] [--font-size <WxH>]
    rusty_tarot --export-art <DIR> [--scale <N>]

OPTIONS:
    --cards <DIR>        folder of card PNGs (default: $RUSTY_TAROT_CARDS, ./cards,
                         or a cards folder next to the executable)
    --graphics <MODE>    auto (default), sixel, kitty, iterm2, or blocks
    --font-size <WxH>    terminal cell size in pixels, e.g. 10x20, if images
                         come out the wrong size (auto-detected otherwise)
    --export-art <DIR>   write the built-in art as PNG templates and exit
    --scale <N>          pixel scale for --export-art (default 3 → 48×72)
    -h, --help           show this help
";

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return Ok(());
    }
    if let Some(dir) = flag("--export-art") {
        let scale = flag("--scale").and_then(|s| s.parse().ok()).unwrap_or(3);
        match art::export_builtin(Path::new(&dir), scale) {
            Ok(n) => println!(
                "wrote {n} PNGs ({}×{} each) to {dir}",
                16 * scale,
                24 * scale
            ),
            Err(e) => {
                eprintln!("export failed: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let art_dir = flag("--cards").map(PathBuf::from).or_else(find_cards_dir);

    let graphics = flag("--graphics")
        .or_else(|| std::env::var("RUSTY_TAROT_GRAPHICS").ok())
        .unwrap_or_else(|| "auto".into());
    let font = flag("--font-size").and_then(|s| {
        let (w, h) = s.split_once(['x', 'X'])?;
        Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
    });

    let mut terminal = ratatui::init();
    // ask the terminal about image support before any input is read
    let picker = gfx::detect(&graphics, font);
    execute!(stdout(), EnableMouseCapture)?;

    // make sure mouse capture is released even if we panic
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        prev(info);
    }));

    let result = run(&mut terminal, art_dir, picker);

    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// $RUSTY_TAROT_CARDS, then ./cards, then `cards` beside the executable
/// (or up from target/release when run through cargo).
fn find_cards_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("RUSTY_TAROT_CARDS") {
        return Some(PathBuf::from(p));
    }
    let mut candidates = vec![PathBuf::from("cards")];
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors().skip(1).take(4) {
            candidates.push(anc.join("cards"));
        }
    }
    candidates.into_iter().find(|p| p.is_dir())
}

fn run(
    terminal: &mut DefaultTerminal,
    art_dir: Option<PathBuf>,
    picker: Option<ratatui_image::picker::Picker>,
) -> io::Result<()> {
    let mut app = App::with_art(Rng::seeded(), art_dir);
    app.gfx = gfx::Gfx::new(picker);
    let mut last_tick = Instant::now();
    while !app.quit {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        let repaint = app.gfx.take_repaint();
        if !repaint.is_empty() {
            // overlapping images: draw those regions once without images so the
            // cells change, then again with images so they're re-sent in order
            app.gfx.set_suppress(repaint);
            terminal.draw(|f| ui::draw(f, &mut app))?;
            app.gfx.set_suppress(vec![]);
            terminal.draw(|f| ui::draw(f, &mut app))?;
            app.gfx.take_repaint();
        }
        if app.gfx.take_clear() {
            // Ctrl+L: repaint everything from scratch
            execute!(stdout(), Clear(ClearType::All))?;
            terminal.swap_buffers();
            terminal.draw(|f| ui::draw(f, &mut app))?;
        }

        let timeout = FRAME.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            // drain everything that's queued (mouse moves arrive in bursts)
            loop {
                match event::read()? {
                    Event::Key(k) if k.kind == KeyEventKind::Press => app.on_key(k),
                    Event::Mouse(m) => app.on_mouse(m),
                    _ => {}
                }
                if app.quit || !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= FRAME {
            app.tick += 1;
            last_tick = Instant::now();
        }
    }
    Ok(())
}
