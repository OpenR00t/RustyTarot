# Rusty Tarot

*instrumentum divinationis*

A retro pixel-art tarot reader for the terminal, built with [ratatui](https://ratatui.rs).

Every card is drawn from a PNG in `cards/`, as a real image in terminals that
support Sixel, Kitty or iTerm2 graphics (Windows Terminal included), or as
half-block pixels elsewhere. Cards are dealt face down —
press Enter to turn them over one at a time — and hovering over any card shows its
meaning in its position.

The interface is styled after a medieval-skinned pocket sampler (think
teenage engineering's EP-1320 "Medieval"): a khaki chassis, a dark green
name-plate with a gothic pixel title and speaker grille, key-cap tabs with
Latin labels (*unus, tres, crux, solea, celtica, codex*), a dark display the
cards are dealt onto with an LED readout, and a parchment page for the reading
with rubricated headings and illuminated initials. `?` lists every key with its
Latin label and English meaning.

## Run

```sh
cargo run --release
```

Use a terminal with true-colour and mouse support (Windows Terminal, WezTerm, Alacritty,
kitty, iTerm2…). Bigger windows get bigger cards, and the single-card spread draws its
card at 2× scale.

## Spreads

| Key | Spread | Cards |
|-----|--------|-------|
| `1` | *unus* — Single Card | 1 |
| `2` | *tres* — Past · Present · Future | 3 |
| `3` | *crux* — Five-Card Cross | 5 |
| `4` | *solea* — Horseshoe | 7 |
| `5` | *celtica* — Celtic Cross (crossing card laid sideways) | 10 |
| `b` | *codex* — browse all 78 cards | — |

## Controls

| Input | Action |
|-------|--------|
| mouse hover | show the card's meaning in the side panel |
| click | turn over a face-down card; click the tabs to switch spreads |
| `←` `→` `↑` `↓` / `Tab` | move between cards |
| `Enter` / `Space` | turn the selected card, or the next face-down one |
| `f` | turn every card over |
| `Esc` | back to the reading overview |
| `n` | shuffle and deal a new reading |
| `r` | toggle reversed cards (applies to the next deal) |
| `a` | toggle auto-flip while dealing (off by default) |
| `F5` | reload card PNGs from disk |
| `g` | toggle real images / half-block pixels |
| `Ctrl+L` | repaint the whole screen |
| `PgUp` / `PgDn` / wheel | scroll the detail panel |
| `?` | help |
| `q` | quit |

## Card art

The cards come from the PNGs in `cards/` — one per card plus `back.png`:

```
major_00_the_fool.png … major_21_the_world.png
wands_01_ace.png … wands_14_king.png      (same for cups_, swords_, pentacles_)
back.png
```

Only the prefix and number matter (`major_13.png` works too). A card without a
PNG falls back to the built-in pixel art. Press `F5` to reload after editing.
Card proportions follow the art (about 1:1.75 for this deck).

The current set was cut from scanned sheets of a printed pixel-art deck with
`tools/extract_cards.py`. It finds each card, straightens it, crops to the
black frame, then **recovers the printed pixel grid** (84×147 true pixels) so the
app can enlarge the art with crisp whole-pixel scaling instead of blurring a scan:

```sh
pip install opencv-python numpy
python tools/extract_cards.py <folder of sheet scans> cards
```

## Graphics

Cards are drawn as real images when the terminal supports a graphics protocol,
and as half-block pixels otherwise.

| Terminal | Mode |
|----------|------|
| Windows Terminal 1.22+ | Sixel (assumed automatically when `WT_SESSION` is set) |
| WezTerm, iTerm2 | iTerm2 inline images |
| kitty, Ghostty | Kitty graphics |
| foot, mlterm, xterm `-ti vt340` | Sixel |
| anything else | half-blocks |

`g` toggles between images and half-blocks while running; the browse screen (`b`)
shows which mode is active. Options if the auto-detection gets it wrong:

```sh
rusty_tarot --graphics sixel          # or kitty, iterm2, blocks
rusty_tarot --font-size 10x20         # cell size in pixels, if images are mis-sized
```

(`RUSTY_TAROT_GRAPHICS=sixel` works as an environment variable too.)

Images are enlarged by the largest whole-pixel factor with nearest-neighbour,
then a small bilinear step for the remainder, so the pixel art stays sharp at
any card size. Flips animate with pixel blocks and the image appears when the
card lands. The screen is never fully cleared during play — only overlapping
cards (the Celtic Cross) get a quick local repaint. `Ctrl+L` forces a full
repaint if a terminal ever leaves stray pixels behind.

To regenerate PNGs of the built-in pixel art (as templates):

```sh
cargo run --release -- --export-art built-in-cards --scale 3
```

## Code layout

| File | What's in it |
|------|--------------|
| `src/deck.rs` | names, keywords, imagery, upright and reversed meanings for all 78 cards |
| `src/art.rs` | built-in sprites and palette, PNG loading/naming, `--export-art` |
| `src/gfx.rs` | Sixel/Kitty/iTerm2 output via `ratatui-image`, protocol detection, repaint tracking |
| `src/pixel.rs` | `Pixmap`, the half-block renderer, scaling/rotation, the pixel title font |
| `src/spreads.rs` | spread layouts (positions in card units) |
| `src/app.rs` | state, dealing/flip animation timing, keyboard and mouse handling |
| `src/ui.rs` | header, table, browse grid, detail panel, footer and help overlay |
| `examples/snapshot.rs` | dev helper that renders one frame headlessly and dumps the cells |
| `tools/extract_cards.py` | cuts and straightens cards from scanned sheets |
