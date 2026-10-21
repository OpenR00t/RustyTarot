// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 The Rusty Tarot contributors

//! Retro pixel art for every card, composed from a 16-colour palette
//! (PICO-8 flavoured) at a base resolution of 16×24 pixels.
//!
//! Palette letters used in the sprite strings:
//!   K black   N navy    P plum    G forest  B brown   D slate   L silver  W white
//!   R red     O orange  Y yellow  E green   U blue    V lavender I pink   F peach
//!   `.` is transparent. In court sprites X = suit robe colour, Z = trim.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use ratatui::style::Color;

use crate::deck::{self, Kind, Suit};
use crate::pixel::Pixmap;
use crate::rng::hash2;

pub const ART_W: usize = 16;
pub const ART_H: usize = 24;

pub fn pal(c: char) -> Option<Color> {
    let (r, g, b) = match c {
        'K' => (16, 12, 24),
        'N' => (29, 43, 83),
        'P' => (126, 37, 83),
        'G' => (0, 135, 81),
        'B' => (171, 82, 54),
        'D' => (95, 87, 79),
        'L' => (194, 195, 199),
        'W' => (255, 241, 232),
        'R' => (255, 0, 77),
        'O' => (255, 163, 0),
        'Y' => (255, 236, 39),
        'E' => (0, 228, 54),
        'U' => (41, 173, 255),
        'V' => (131, 118, 156),
        'I' => (255, 119, 168),
        'F' => (255, 204, 170),
        _ => return None,
    };
    Some(Color::Rgb(r, g, b))
}

fn sprite(rows: &[&str]) -> Pixmap {
    Pixmap::from_rows(rows, pal)
}

const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// Vertical two-colour gradient with ordered dithering, optional starfield.
fn scene(top: char, bot: char, stars: bool, seed: u32) -> Pixmap {
    let (t, b) = (pal(top), pal(bot));
    let mut p = Pixmap::new(ART_W, ART_H);
    for y in 0..ART_H {
        // solid top and bottom with a dithered band through the middle
        let f = ((y as f32 / (ART_H - 1) as f32 - 0.3) / 0.4).clamp(0.0, 1.0);
        for x in 0..ART_W {
            let thr = (BAYER[y % 4][x % 4] as f32 + 0.5) / 16.0;
            p.set(x as i32, y as i32, if f > thr { b } else { t });
            if stars && y < ART_H * 2 / 3 {
                let h = hash2(x as u32, y as u32, seed);
                if h.is_multiple_of(29) {
                    p.set(
                        x as i32,
                        y as i32,
                        pal(if h.is_multiple_of(3) { 'W' } else { 'V' }),
                    );
                }
            }
        }
    }
    p
}

/// (sky top, sky bottom, starfield) for each Major Arcana card.
const MAJOR_SCENES: [(char, char, bool); 22] = [
    ('U', 'V', false), // Fool
    ('P', 'N', true),  // Magician
    ('N', 'P', true),  // High Priestess
    ('U', 'G', false), // Empress
    ('R', 'P', false), // Emperor
    ('P', 'K', true),  // Hierophant
    ('U', 'I', false), // Lovers
    ('K', 'N', true),  // Chariot
    ('E', 'G', false), // Strength
    ('N', 'K', true),  // Hermit
    ('V', 'N', true),  // Wheel
    ('V', 'P', false), // Justice
    ('V', 'N', false), // Hanged Man
    ('K', 'P', true),  // Death
    ('U', 'V', false), // Temperance
    ('K', 'P', false), // Devil
    ('K', 'N', true),  // Tower
    ('K', 'N', true),  // Star
    ('K', 'N', true),  // Moon
    ('U', 'I', false), // Sun
    ('U', 'V', false), // Judgement
    ('P', 'N', true),  // World
];

const MAJOR_SPRITES: [&[&str]; 22] = [
    // 0 The Fool
    &[
        "............YY..",
        "..........Y.YY.Y",
        "...........YYYY.",
        "..........YYYYYY",
        "...........YYYY.",
        "..........Y.YY.Y",
        "............YY..",
        ".......RRR......",
        "......RRRRR..B..",
        ".......FFF..BRR.",
        ".......FFF.B.RR.",
        "......OOOOBO....",
        ".....FOOOOOOF...",
        ".....W.OOOO.....",
        "....WIW.OOO.....",
        "......YYYYYY....",
        "..W...OOO.OO....",
        ".WWW..OO...OO...",
        ".W.W..KK....KK..",
        "GGGGGGGGGGGG....",
        "GGGGGGGGGGG.....",
        "GEGGGGGGGG......",
        "GGGGGEGGG.......",
        "GGGGGGGG........",
    ],
    // 1 The Magician
    &[
        "................",
        "..YYY......YYY..",
        ".Y...Y....Y...Y.",
        ".Y....Y..Y....Y.",
        ".Y.....YY.....Y.",
        ".Y....Y..Y....Y.",
        ".Y...Y....Y...Y.",
        "..YYY......YYY..",
        "................",
        ".......YY.......",
        ".......WW.......",
        ".......WW.......",
        ".......WW.......",
        ".......WW.......",
        ".......WW.......",
        ".......YY.......",
        "................",
        ".YYY...L...OOO..",
        ".UUU...L..OYYO..",
        "..U...YLY.OYYO..",
        ".UUU...B...OOO..",
        "BBBBBBBBBBBBBBBB",
        ".B............B.",
        ".B............B.",
    ],
    // 2 The High Priestess
    &[
        "................",
        "VDDDV......VWWWV",
        ".DDD...YY...WWW.",
        ".DDD..Y.....WWW.",
        ".DDD.Y......WWW.",
        ".DDD.Y......WWW.",
        ".DDD..Y.....WWW.",
        ".DDD...YY...WWW.",
        ".DDD........WWW.",
        ".DWD...UU...WDW.",
        ".DDD..UUUU..WWW.",
        ".DDD..UFFU..WWW.",
        ".DDD..UFFU..WWW.",
        ".DDD..UUUU..WWW.",
        ".DDD.UUUUUU.WWW.",
        ".DDD.UUWWUU.WWW.",
        ".DDD.UUUUUU.WWW.",
        ".DDD.UUUUUU.WWW.",
        ".DDDUUUUUUUUWWW.",
        "VDDDVUUUUUUVWWWV",
        "VVVVVVVVVVVVVVVV",
        "LVLVLVLVLVLVLVLV",
        "VVVVVVVVVVVVVVVV",
        "................",
    ],
    // 3 The Empress
    &[
        "................",
        "...Y..Y..Y..Y...",
        ".Y............Y.",
        "................",
        "......IIII......",
        ".....I....I.....",
        "....I......I....",
        "....I......I....",
        "....I......I....",
        "....I......I....",
        ".....I....I.....",
        "......IIII......",
        ".......II.......",
        ".......II.......",
        ".....IIIIII.....",
        ".......II.......",
        ".......II.......",
        "................",
        "Y.O.Y.O.Y.O.Y.O.",
        "YOYOYOYOYOYOYOYO",
        "EYEOEYEOEYEOEYEO",
        "EEEEEEEEEEEEEEEE",
        "GEGEGEGEGEGEGEGE",
        "GGGGGGGGGGGGGGGG",
    ],
    // 4 The Emperor
    &[
        "................",
        "...Y...YY...Y...",
        "...YY.YYYY.YY...",
        "...YYYYYYYYYY...",
        "...YRYYUUYYRY...",
        "...YYYYYYYYYY...",
        "................",
        "..LL........LL..",
        ".L..L......L..L.",
        ".L.LLLLLLLLLL.L.",
        "..LLDDDDDDDDLL..",
        "...LDDDDDDDDL...",
        "...LDRRRRRRDL...",
        "...LDRRRRRRDL...",
        "...LDRRYYRRDL...",
        "...LDRRRRRRDL...",
        ".LLLLLLLLLLLLLL.",
        ".L.LDDDDDDDDL.L.",
        ".L.L........L.L.",
        ".L.L........L.L.",
        "LLLLLLLLLLLLLLLL",
        "DDDDDDDDDDDDDDDD",
        "DKDDDKDDDDKDDDKD",
        "KKKKKKKKKKKKKKKK",
    ],
    // 5 The Hierophant
    &[
        ".......YY.......",
        ".....YYYYYY.....",
        ".......YY.......",
        "....YYYYYYYY....",
        ".......YY.......",
        "...YYYYYYYYYY...",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        "......RRRR......",
        ".YYY........LLL.",
        ".Y.Y........L.L.",
        ".YYY........LLL.",
        "...Y........L...",
        "....Y......L....",
        ".....Y....L.....",
        "......Y..L......",
        ".......YL.......",
        ".......LY.......",
        "......L..Y......",
        ".....LL..YY.....",
    ],
    // 6 The Lovers
    &[
        "......YYYY......",
        "....YYYYYYYY....",
        "...YYYYYYYYYY...",
        "..Y.YYYYYYYY.Y..",
        "................",
        "....RRR..RRR....",
        "...RRRRRRRRRR...",
        "...RWRRRRRRRR...",
        "...RRRRRRRRRR...",
        "....RRRRRRRR....",
        ".....RRRRRR.....",
        "......RRRR......",
        ".......RR.......",
        "................",
        "..FF........FF..",
        "..FF........FF..",
        ".EEEE......UUUU.",
        "F.EE.F....F.UU.F",
        "..EE........UU..",
        "..EE........UU..",
        "..F.F......F.F..",
        "..F.F......F.F..",
        "EEEEEEEEEEEEEEEE",
        "GGGGGGGGGGGGGGGG",
    ],
    // 7 The Chariot
    &[
        "................",
        ".UUUUUUUUUUUUUU.",
        ".UYUUYUUYUUYUUU.",
        ".UUUUUUUUUUUUUU.",
        "..L..........L..",
        "..L...LLLL...L..",
        "..L...FFFF...L..",
        "..L...FKFK...L..",
        "..L..LLLLLL..L..",
        "..L.LLLYYLLL.L..",
        "LLLLLLLLLLLLLLLL",
        "LDDDDDDDDDDDDDDL",
        "LDDDYYYYYYYYDDDL",
        "LDDDDDDYYDDDDDDL",
        "LLLLLLLLLLLLLLLL",
        ".BBB........BBB.",
        "B.B.B......B.B.B",
        "BBBBB......BBBBB",
        "B.B.B......B.B.B",
        ".BBB........BBB.",
        "................",
        ".KKKK......WWWW.",
        "KKKKKK....WWWWWW",
        "K.K.KK....WW.W.W",
    ],
    // 8 Strength
    &[
        ".....YY..YY.....",
        "....Y..YY..Y....",
        ".....YY..YY.....",
        "................",
        "....OBOOOOBO....",
        "..OOBOOOOOOBOO..",
        ".OBOOYYYYYYOOBO.",
        ".OOOYYYYYYYYOOO.",
        "OBOYYKYYYYKYYOBO",
        "OOOYYYYYYYYYYOOO",
        "OBOYYYYYYYYYYOBO",
        "OOOYYYYBBYYYYOOO",
        ".OBOYYYBBYYYOBO.",
        ".OOOYYYYYYYYOOO.",
        "..OBOYRRRRYOBO..",
        "..OOOOYYYYOOOO..",
        "....OBOOOOBO....",
        "......OOOO......",
        "................",
        "..I....Y....I...",
        ".IYI..YOY..IYI..",
        "..I....Y....I...",
        "..E....E....E...",
        "GGGGGGGGGGGGGGGG",
    ],
    // 9 The Hermit
    &[
        "................",
        ".......LL.......",
        "......L..L......",
        "......LLLL......",
        ".....LLLLLL.....",
        "....OL.YY.LO....",
        "...O.LYYYYL.O...",
        "....OLYWWYLO....",
        "...O.LYYYYL.O...",
        "....OL.YY.LO....",
        ".....LLLLLL.....",
        "......LLLL......",
        "................",
        "................",
        "................",
        ".......WW.......",
        "......WWWW......",
        ".....WWLLWW.....",
        "....WWLLLLLW....",
        "...WLLLLDLLLW...",
        "..WLLLDDLLLLLW..",
        ".LLLDDDLLDDLLLL.",
        "LLDDDDDDDDDDDDLL",
        "DDDDDDDDDDDDDDDD",
    ],
    // 10 Wheel of Fortune
    &[
        "................",
        "................",
        "................",
        "................",
        ".....OOOOOO.....",
        "...OOYYYYYYOO...",
        "..OY...YY...YO..",
        ".OY.Y..YY..Y.YO.",
        ".OY..Y.YY.Y..YO.",
        "OY....YYYY....YO",
        "OYYYYYYOOYYYYYYO",
        "OYYYYYYOOYYYYYYO",
        "OY....YYYY....YO",
        ".OY..Y.YY.Y..YO.",
        ".OY.Y..YY..Y.YO.",
        "..OY...YY...YO..",
        "...OOYYYYYYOO...",
        ".....OOOOOO.....",
        "................",
        "................",
        "..LLL.....LLLL..",
        ".LWWWLL..LWWWWL.",
        "LWWWWWWLLWWWWWWL",
        "WWWWWWWWWWWWWWWW",
    ],
    // 11 Justice
    &[
        ".......WW.......",
        ".......LL.......",
        ".......LL.......",
        ".......LL.......",
        ".......LL.......",
        ".....YYYYYY.....",
        "YYYYYYYYYYYYYYYY",
        "..Y....YY....Y..",
        ".Y.Y...YY...Y.Y.",
        "Y...Y..YY..Y...Y",
        "YYYYY..YY..YYYYY",
        ".YYY...YY...YYY.",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        ".......YY.......",
        "......YYYY......",
        ".....YYYYYY.....",
        "....RRRRRRRR....",
        "...RRRRRRRRRR...",
        "LLLLLLLLLLLLLLLL",
        "LDLLLLDLLLLDLLLL",
        "LLLLLLLLLLLLLLLL",
    ],
    // 12 The Hanged Man
    &[
        "EBBBBBBBBBBBBBBE",
        "EBBBBBBBBBBBBBBE",
        ".BB....YY....BB.",
        ".BB....YY....BB.",
        ".BB....UU....BB.",
        ".BB....UU.UU.BB.",
        ".BB....UUUUU.BB.",
        ".BB....UU....BB.",
        ".BB...RRRR...BB.",
        ".BB...RRRR...BB.",
        ".BB..FRRRRF..BB.",
        ".BB..FRRRRF..BB.",
        ".BB...RRRR...BB.",
        ".BB....FF....BB.",
        ".BB...FFFF...BB.",
        ".BB...FFFF...BB.",
        ".BB..Y.FF.Y..BB.",
        ".BB...YYYY...BB.",
        ".BB..........BB.",
        ".BB..........BB.",
        ".BB..........BB.",
        "EBBEEEEEEEEEEBBE",
        "GGGGGGGGGGGGGGGG",
        "GGGGGGGGGGGGGGGG",
    ],
    // 13 Death
    &[
        "................",
        "................",
        "....WWWWWWWW....",
        "...WWWWWWWWWW...",
        "..WWWWWWWWWWWW..",
        "..WWWWWWWWWWWW..",
        "..WWKKKWWKKKWW..",
        "..WWKKKWWKKKWW..",
        "..WWKKKWWKKKWW..",
        "...WWWWKKWWWW...",
        "....WWWKKWWW....",
        ".....WWWWWW.....",
        ".....WKWKWW.....",
        ".....WWWWWW.....",
        "................",
        "................",
        "D............D..",
        "DD..........DD..",
        "DD...OOOOO..DD..",
        "DD..OYYYYYO.DD..",
        "DD.OYYYYYYYODD..",
        "OOOOOOOOOOOOOOOO",
        "BBBBBBBBBBBBBBBB",
        "KBKBKBKBKBKBKBKB",
    ],
    // 14 Temperance
    &[
        "..............Y.",
        ".............YOY",
        "..............Y.",
        ".....RR....RR...",
        "....RRRR..RRRR..",
        "..YYYY..RRRR....",
        "..YUUY..........",
        "...YYUU.........",
        "...YY..U........",
        "..YYYY..U.......",
        ".........U......",
        "..........U.....",
        ".........YUUY...",
        ".........YUUY...",
        "..........YY....",
        "..........YY....",
        ".........YYYY...",
        "................",
        "...I.......O....",
        "..IYI.....O.....",
        "...E.....O......",
        "EEEEEEEEO.UUUUUU",
        "GGGGGGGOUUUUUUUU",
        "GGGGGGGUUUNUUUNU",
    ],
    // 15 The Devil
    &[
        "................",
        ".L............L.",
        ".LL..........LL.",
        "..LL........LL..",
        "..LLL.RRRR.LLL..",
        "...LRRRRRRRRL...",
        "...RRRRRRRRRR...",
        "..RRYYRRRRYYRR..",
        "..RRYKRRRRKYRR..",
        "..RRRRRRRRRRRR..",
        "...RRRRBBRRRR...",
        "...RRRRRRRRRR...",
        "....RKKKKKKR....",
        "....RRWRRWRR....",
        ".....RRRRRR.....",
        "......RRRR......",
        ".......RR.......",
        "................",
        "D.D.D.D..D.D.D.D",
        ".D...D....D...D.",
        "..D.D......D.D..",
        "...D........D...",
        "..DDD......DDD..",
        "..DDD......DDD..",
    ],
    // 16 The Tower
    &[
        ".Y..............",
        "..Y.......Y.Y.Y.",
        "..YY......YYYYY.",
        "...YY.....YRYRY.",
        "....Y...........",
        "...YY...........",
        ".....Y..........",
        ".....LYL.L......",
        ".....LLLLLL.....",
        ".....LLLLLL.....",
        ".....LOLLOL.....",
        "....OLRLLRLO....",
        ".....LLLLLL.....",
        ".....LLLLLL.....",
        ".....LLOLLL.....",
        "....OLLRLLL.....",
        ".....LLLLLL.....",
        ".....LLLLLL.....",
        ".F...LLKKLL...F.",
        "FF..LLLKKLLL..FF",
        "...DDDDDDDDDD...",
        "..DDDDDDDDDDDD..",
        ".DDDDDDDDDDDDDD.",
        "DDDDDDDDDDDDDDDD",
    ],
    // 17 The Star
    &[
        "................",
        ".......YY.......",
        ".......YY.......",
        "..Y....YY....Y..",
        "...Y...YY...Y...",
        "....Y.YYYY.Y....",
        ".....YYWWYY.....",
        "YYYYYYWWWWYYYYYY",
        "YYYYYYWWWWYYYYYY",
        ".....YYWWYY.....",
        "....Y.YYYY.Y....",
        "...Y...YY...Y...",
        "..Y....YY....Y..",
        ".......YY.......",
        ".......YY.......",
        "................",
        "..W.........W...",
        ".WYW.......WYW..",
        "..W....W....W...",
        "......WYW.......",
        "UUUUUUUWUUUUUUUU",
        "ULUUUUUUULUUUUUU",
        "UUUUULUUUUUUULUU",
        "NUNUNUNUNUNUNUNU",
    ],
    // 18 The Moon
    &[
        "................",
        "......LLLL......",
        "....LLLLLLLL....",
        "...YYYLLLLLLL...",
        "..YYYYYLLDLLLL..",
        "..YYYYYLLLLLLL..",
        "..YYYYKLLLLKLL..",
        "..YYYYYLLLLLLL..",
        "...YYYYLLDDLL...",
        "....YYYYLLLL....",
        "......LLLL......",
        "................",
        ".D............D.",
        "DDD..........DDD",
        "DYD..........DYD",
        "DDD..........DDD",
        "DDD....O.....DDD",
        "DDD....O.....DDD",
        "DDD.....O....DDD",
        "EEEEEEEEO.EEEEEE",
        "EEEEEEEO.EEEEEEE",
        "UUUUUUUUUUUUUUUU",
        "UUUUURRRUUUUUUUU",
        "UUUURURURUUUUUUU",
    ],
    // 19 The Sun
    &[
        "................",
        "Y......Y......Y.",
        ".Y.....Y.....Y..",
        "..Y..OOOOOO.Y...",
        "....OOYYYYOO....",
        "...OYYYYYYYYO...",
        "...OYYKYYKYYO...",
        "YYYOYYYYYYYYOYYY",
        "...OYYYYYYYYO...",
        "...OYYRYYRYYO...",
        "...OYYYRRYYYO...",
        "....OOYYYYOO....",
        "..Y..OOOOOO.Y...",
        ".Y.....Y.....Y..",
        "Y......Y......Y.",
        "................",
        "................",
        "..Y....Y....Y...",
        ".YBY..YBY..YBY..",
        "..Y....Y....Y...",
        "..E....E....E...",
        "LLLLLLLLLLLLLLLL",
        "LDLLLDLLLDLLLDLL",
        "LLLLLLLLLLLLLLLL",
    ],
    // 20 Judgement
    &[
        "................",
        "...WWW..........",
        ".LWWWWWL...WWW..",
        "LWWWWWWWL.WWWWW.",
        ".LLLLLLL...LLL..",
        "................",
        "..YY............",
        "...YY...........",
        "....YY..........",
        ".....YY.........",
        "......YYY.......",
        "......YYYY......",
        ".....WWRWWYY....",
        ".....RRRRR.YY...",
        ".....WWRWW......",
        "................",
        "................",
        ".F..F.....F..F..",
        "..FF.......FF...",
        "..FF.......FF...",
        ".DDDD.....DDDD..",
        ".DKKD.....DKKD..",
        "UUUUUUUUUUUUUUUU",
        "NUNUNUNUNUNUNUNU",
    ],
    // 21 The World
    &[
        "WW............OO",
        "WW....EEEE....OO",
        "....EE....EE....",
        "...E........E...",
        "..E....FF....E..",
        "..E....FF....E..",
        ".E....PPPP....E.",
        ".E...F.PP.F...E.",
        ".E...B.PP.B...E.",
        ".E.....PP.....E.",
        ".E....PPPP....E.",
        ".E....F..F....E.",
        ".E....F...F...E.",
        ".E....F.......E.",
        "..E..........E..",
        "..E..........E..",
        "...E........E...",
        "....EE....EE....",
        "......ERRE......",
        "......EEEE......",
        "................",
        "................",
        "YY............BB",
        "YY............BB",
    ],
];

// --- suit pips (5×5) -----------------------------------------------------

const PIP_WANDS: &[&str] = &["...EE", "..OE.", "..O..", ".O...", "O...."];
const PIP_CUPS: &[&str] = &["YYYYY", "YUUUY", ".YYY.", "..Y..", ".YYY."];
const PIP_SWORDS: &[&str] = &["..W..", "..L..", "..L..", "YYYYY", "..B.."];
const PIP_PENTACLES: &[&str] = &[".YYY.", "YOYOY", "YYOYY", "YOYOY", ".YYY."];

fn pip(s: Suit) -> Pixmap {
    sprite(match s {
        Suit::Wands => PIP_WANDS,
        Suit::Cups => PIP_CUPS,
        Suit::Swords => PIP_SWORDS,
        Suit::Pentacles => PIP_PENTACLES,
    })
}

const L: i32 = 1;
const C: i32 = 5;
const R: i32 = 10;

fn pip_layout(rank: u8) -> &'static [(i32, i32)] {
    match rank {
        2 => &[(C, 2), (C, 17)],
        3 => &[(C, 1), (C, 10), (C, 19)],
        4 => &[(L, 1), (R, 1), (L, 19), (R, 19)],
        5 => &[(L, 1), (R, 1), (C, 10), (L, 19), (R, 19)],
        6 => &[(L, 1), (R, 1), (L, 10), (R, 10), (L, 19), (R, 19)],
        7 => &[(L, 1), (R, 1), (C, 5), (L, 10), (R, 10), (L, 19), (R, 19)],
        8 => &[
            (L, 1),
            (R, 1),
            (C, 5),
            (L, 10),
            (R, 10),
            (C, 15),
            (L, 19),
            (R, 19),
        ],
        9 => &[
            (L, 1),
            (R, 1),
            (L, 7),
            (R, 7),
            (C, 10),
            (L, 13),
            (R, 13),
            (L, 19),
            (R, 19),
        ],
        _ => &[
            (L, 1),
            (R, 1),
            (C, 4),
            (L, 7),
            (R, 7),
            (L, 13),
            (R, 13),
            (C, 16),
            (L, 19),
            (R, 19),
        ],
    }
}

// --- court busts (12 wide) ------------------------------------------------

const COURT: [&[&str]; 4] = [
    // Page
    &[
        "....XXXXX...",
        "...XXXXXXXZ.",
        "...ZZZZZZ.Z.",
        "...BFFFFB...",
        "...FKFFKF...",
        "...FFFFFF...",
        "....FRRF....",
        ".....FF.....",
        "...XXXXXX...",
        "..XXXZZXXX..",
        ".XXXXZZXXXX.",
        ".XXXXZZXXXX.",
        ".XXXXZZXXXX.",
        ".XXXXZZXXXX.",
    ],
    // Knight
    &[
        ".....RR.....",
        "....LLLL....",
        "...LLLLLL...",
        "...LLLLLL...",
        "...LKKKKL...",
        "...LLLLLL...",
        "...LLLLLL...",
        "....LLLL....",
        "..LLLLLLLL..",
        ".LXXXZZXXXL.",
        ".LXXXZZXXXL.",
        ".XXXXZZXXXX.",
        ".XXXXZZXXXX.",
        ".XXXXZZXXXX.",
    ],
    // Queen
    &[
        "....Y..Y....",
        "...YYYYYY...",
        "..BBBBBBBB..",
        "..BFFFFFFB..",
        "..BFKFFKFB..",
        "..BFFFFFFB..",
        "..BBFRRFBB..",
        "..BB.FF.BB..",
        "..BXXXXXXB..",
        ".XXXZZZZXXX.",
        ".XXXXZZXXXX.",
        "XXXXXZZXXXXX",
        "XXXXXZZXXXXX",
        "XXXXXZZXXXXX",
    ],
    // King
    &[
        "...Y.YY.Y...",
        "...YYYYYY...",
        "...YRYYRY...",
        "...FFFFFF...",
        "...FKFFKF...",
        "...FFFFFF...",
        "...LLLLLL...",
        "....LRRL....",
        ".....LL.....",
        "..XXXXXXXX..",
        ".XXXZZZZXXX.",
        ".XXXXZZXXXX.",
        "XXXXXZZXXXXX",
        "XXXXXZZXXXXX",
    ],
];

fn suit_scene(s: Suit) -> (char, char, char) {
    // (sky top, sky bottom, robe)
    match s {
        Suit::Wands => ('B', 'K', 'R'),
        Suit::Cups => ('N', 'K', 'U'),
        Suit::Swords => ('D', 'K', 'V'),
        Suit::Pentacles => ('G', 'K', 'E'),
    }
}

fn minor_art(s: Suit, rank: u8, id: usize) -> Pixmap {
    let (top, bot, robe) = suit_scene(s);
    let mut p = scene(top, bot, false, id as u32);
    let pp = pip(s);
    match rank {
        1 => {
            // Ace: a big glowing pip with sparkles
            let big = pp.scaled(10, 10);
            for (x, y) in [
                (2, 3),
                (13, 4),
                (1, 17),
                (14, 18),
                (7, 1),
                (8, 21),
                (3, 11),
                (12, 12),
            ] {
                p.set(x, y, pal('Y'));
            }
            for (x, y) in [(1, 6), (14, 9), (4, 20), (11, 2)] {
                p.set(x, y, pal('W'));
            }
            p.blit(&big, 3, 7);
        }
        2..=10 => {
            for &(x, y) in pip_layout(rank) {
                p.blit(&pp, x, y);
            }
        }
        _ => {
            let rows = COURT[(rank - 11) as usize];
            let bust = Pixmap::from_rows(rows, |c| match c {
                'X' => pal(robe),
                'Z' => pal('Y'),
                other => pal(other),
            });
            p.blit(&bust, 2, 2);
            p.blit(&pp, 5, 18);
        }
    }
    p
}

fn major_art(n: usize) -> Pixmap {
    let (t, b, stars) = MAJOR_SCENES[n];
    let mut p = scene(t, b, stars, n as u32 * 7 + 3);
    let s = sprite(MAJOR_SPRITES[n]);
    let ox = (ART_W as i32 - s.w as i32) / 2;
    let oy = (ART_H as i32 - s.h as i32) / 2;
    p.blit(&s, ox, oy);
    p
}

fn back_art() -> Pixmap {
    let mut p = Pixmap::new(ART_W, ART_H);
    for y in 0..ART_H as i32 {
        for x in 0..ART_W as i32 {
            let edge = x == 0 || y == 0 || x == ART_W as i32 - 1 || y == ART_H as i32 - 1;
            let lattice = (x + y) % 4 == 0 || (x - y).rem_euclid(4) == 0;
            let c = if edge {
                'Y'
            } else if x == 1 || y == 1 || x == ART_W as i32 - 2 || y == ART_H as i32 - 2 {
                'K'
            } else if lattice {
                'V'
            } else {
                'P'
            };
            p.set(x, y, pal(c));
        }
    }
    let emblem = sprite(&[
        "...YYYY...",
        "..YKKKKY..",
        ".YKKYYKKY.",
        "YKKYYWYKKY",
        "YKYYWWWYKY",
        "YKKYYWYKKY",
        ".YKKYYKKY.",
        "..YKKKKY..",
        "...YYYY...",
    ]);
    p.blit(&emblem, 3, 7);
    p
}

/// Custom art is also kept as a pixel map for the half-block renderer; anything
/// bigger than this is averaged down once at load time (the terminal can't show
/// more than this with half-blocks anyway).
pub const MAX_ART_W: usize = 128;
pub const MAX_ART_H: usize = 256;

/// Index of the card back in per-image tables (after the 78 cards).
pub const BACK: usize = deck::DECK_SIZE;

/// Art for every card plus the card back — built-in pixel art, with any PNGs
/// found in the art folder layered over the top.
pub struct ArtCache {
    /// Pixel maps for the half-block renderer (78 cards + back).
    pix: Vec<Pixmap>,
    /// Full-resolution images from the cards folder, for terminal graphics protocols.
    full: Vec<Option<image::RgbaImage>>,
    /// Height ÷ width of the card art (1.5 for the built-in 16×24 art).
    pub aspect: f32,
    /// Cache of resized pixel maps: (image, w, h, transform) → pixmap.
    scaled: RefCell<HashMap<(usize, usize, usize, u8), Pixmap>>,
}

impl ArtCache {
    /// Built-in art only.
    pub fn new() -> Self {
        let mut pix: Vec<Pixmap> = (0..deck::DECK_SIZE)
            .map(|id| match deck::kind(id) {
                Kind::Major(n) => major_art(n as usize),
                Kind::Minor(s, r) => minor_art(s, r, id),
            })
            .collect();
        pix.push(back_art());
        Self {
            pix,
            full: vec![None; BACK + 1],
            aspect: ART_H as f32 / ART_W as f32,
            scaled: RefCell::new(HashMap::new()),
        }
    }

    /// Built-in art, overridden by any matching PNGs in `dir`.
    /// Returns the cache and a one-line status for the UI.
    pub fn load(dir: Option<&Path>) -> (Self, String) {
        let mut cache = Self::new();
        let Some(dir) = dir else {
            return (cache, "built-in art (no cards folder found)".into());
        };
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => return (cache, format!("built-in art ({}: {e})", dir.display())),
        };
        let (mut loaded, mut failed) = (0, Vec::new());
        let mut aspects = vec![];
        for entry in entries.flatten() {
            let path = entry.path();
            let is_png = path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("png"));
            let Some(target) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(parse_art_name)
            else {
                continue;
            };
            if !is_png {
                continue;
            }
            match load_png(&path) {
                Ok((pix, full)) => {
                    let idx = match target {
                        ArtTarget::Card(id) => id,
                        ArtTarget::Back => BACK,
                    };
                    aspects.push(full.height() as f32 / full.width() as f32);
                    cache.pix[idx] = pix;
                    cache.full[idx] = Some(full);
                    loaded += 1;
                }
                Err(e) => failed.push(format!(
                    "{}: {e}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )),
            }
        }
        if !aspects.is_empty() {
            aspects.sort_by(|a, b| a.total_cmp(b));
            cache.aspect = aspects[aspects.len() / 2];
        }
        let mut status = format!("{loaded} of 79 images from {}", dir.display());
        if !failed.is_empty() {
            status.push_str(&format!(
                " — {} failed: {}",
                failed.len(),
                failed.join("; ")
            ));
        }
        (cache, status)
    }

    pub fn card(&self, id: usize) -> &Pixmap {
        &self.pix[id]
    }

    pub fn back(&self) -> &Pixmap {
        &self.pix[BACK]
    }

    /// Full-resolution image for card `idx` (or [`BACK`]), if one was loaded from disk.
    pub fn full(&self, idx: usize) -> Option<&image::RgbaImage> {
        self.full.get(idx).and_then(|f| f.as_ref())
    }

    /// Pixel map for `idx` resized to `w`×`h`, after rotating (1 = 90° cw, 2 = 180°, 3 = 270°).
    /// Results are cached, so redrawing a frame is cheap even for big source images.
    pub fn scaled(&self, idx: usize, w: usize, h: usize, turn: u8) -> Pixmap {
        let key = (idx, w, h, turn);
        if let Some(p) = self.scaled.borrow().get(&key) {
            return p.clone();
        }
        let mut img = self.pix[idx].clone();
        match turn {
            1 => img = img.rotate_cw(),
            2 => img = img.rotate_180(),
            3 => img = img.rotate_cw().rotate_180(),
            _ => {}
        }
        let out = img.scaled(w, h);
        let mut cache = self.scaled.borrow_mut();
        if cache.len() > 600 {
            cache.clear();
        }
        cache.insert(key, out.clone());
        out
    }
}

impl Default for ArtCache {
    fn default() -> Self {
        Self::new()
    }
}

enum ArtTarget {
    Card(usize),
    Back,
}

fn load_png(path: &Path) -> Result<(Pixmap, image::RgbaImage), String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w == 0 || h == 0 {
        return Err("empty image".into());
    }
    let mut p = Pixmap::from_rgba(w, h, img.as_raw());
    // A blocky upscale of pixel art (e.g. a 16×24 sprite saved at 3× = 48×72)
    // goes back to its true grid, so it stays crisp at every card size.
    let k = block_factor(&p);
    let full = if k > 1 {
        p = p.scaled(w / k, h / k);
        image::imageops::resize(
            &img,
            (w / k) as u32,
            (h / k) as u32,
            image::imageops::FilterType::Nearest,
        )
    } else {
        img
    };
    let (w, h) = (p.w, p.h);
    if w > MAX_ART_W || h > MAX_ART_H {
        let s = (MAX_ART_W as f32 / w as f32).min(MAX_ART_H as f32 / h as f32);
        let (tw, th) = (
            ((w as f32 * s).round() as usize).max(1),
            ((h as f32 * s).round() as usize).max(1),
        );
        p = p.scaled(tw, th);
    }
    Ok((p, full))
}

/// Largest k such that the image is made of uniform k×k blocks.
fn block_factor(p: &Pixmap) -> usize {
    let g = gcd(p.w, p.h);
    (2..=g)
        .rev()
        .filter(|k| g.is_multiple_of(*k))
        .find(|&k| (0..p.h).all(|y| (0..p.w).all(|x| p.get(x, y) == p.get(x - x % k, y - y % k))))
        .unwrap_or(1)
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

const SUIT_PREFIX: [&str; 4] = ["wands", "cups", "swords", "pentacles"];
const RANK_WORD: [&str; 14] = [
    "ace", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "page",
    "knight", "queen", "king",
];

/// File name (without extension) for a card, e.g. `major_13_death`, `cups_12_knight`.
pub fn art_file_stem(id: usize) -> String {
    match deck::kind(id) {
        Kind::Major(n) => {
            let slug: String = deck::text(id)
                .name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            format!("major_{n:02}_{slug}")
        }
        Kind::Minor(s, r) => {
            let si = deck::SUITS.iter().position(|x| *x == s).unwrap();
            format!("{}_{r:02}_{}", SUIT_PREFIX[si], RANK_WORD[r as usize - 1])
        }
    }
}

/// Accepts `major_13`, `major_13_death`, `cups_01_whatever`, `back`… (case-insensitive).
fn parse_art_name(stem: &str) -> Option<ArtTarget> {
    let stem = stem.to_lowercase();
    let mut parts = stem.split(['_', '-', ' ']);
    let kind = parts.next()?;
    if kind == "back" {
        return Some(ArtTarget::Back);
    }
    let n: usize = parts.next()?.parse().ok()?;
    if kind == "major" {
        return (n < 22).then_some(ArtTarget::Card(n));
    }
    let si = SUIT_PREFIX.iter().position(|p| *p == kind)?;
    (1..=14)
        .contains(&n)
        .then(|| ArtTarget::Card(22 + si * 14 + n - 1))
}

/// Write the built-in art for all 78 cards plus the back as PNGs, upscaled by `scale`.
pub fn export_builtin(dir: &Path, scale: usize) -> Result<usize, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let cache = ArtCache::new();
    let mut jobs: Vec<(String, &Pixmap)> = (0..deck::DECK_SIZE)
        .map(|id| (art_file_stem(id), cache.card(id)))
        .collect();
    jobs.push(("back".into(), cache.back()));
    for (stem, p) in &jobs {
        let path = dir.join(format!("{stem}.png"));
        let rgba = p.to_rgba(scale);
        image::save_buffer(
            &path,
            &rgba,
            (p.w * scale) as u32,
            (p.h * scale) as u32,
            image::ColorType::Rgba8,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(jobs.len())
}

/// Sprite rows that aren't 16 wide (used by tests to catch typos).
pub fn sprite_width_problems() -> Vec<(usize, usize, usize)> {
    let mut out = vec![];
    for (n, rows) in MAJOR_SPRITES.iter().enumerate() {
        for (r, row) in rows.iter().enumerate() {
            let w = row.chars().count();
            if w != ART_W {
                out.push((n, r, w));
            }
        }
        if rows.len() > ART_H {
            out.push((n, 999, rows.len()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn art_names_round_trip() {
        for id in 0..crate::deck::DECK_SIZE {
            match super::parse_art_name(&super::art_file_stem(id)) {
                Some(super::ArtTarget::Card(got)) => assert_eq!(got, id),
                _ => panic!("bad name for {id}"),
            }
        }
        assert!(matches!(
            super::parse_art_name("Back"),
            Some(super::ArtTarget::Back)
        ));
        assert!(matches!(
            super::parse_art_name("major_13"),
            Some(super::ArtTarget::Card(13))
        ));
    }

    #[test]
    fn sprites_are_16_wide() {
        assert_eq!(super::sprite_width_problems(), vec![]);
    }
}
