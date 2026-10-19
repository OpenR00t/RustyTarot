//! Spread layouts. Slot positions are card *centers* measured in layout units,
//! where one unit is a card's footprint plus its gap.

pub struct Slot {
    pub name: &'static str,
    pub meaning: &'static str,
    pub x: f32,
    pub y: f32,
    /// Drawn sideways across the previous card (Celtic Cross "crossing" card).
    pub rotated: bool,
}

pub struct Spread {
    pub name: &'static str,
    pub tab: &'static str,
    pub blurb: &'static str,
    pub w: f32,
    pub h: f32,
    pub slots: &'static [Slot],
}

const fn s(name: &'static str, meaning: &'static str, x: f32, y: f32) -> Slot {
    Slot {
        name,
        meaning,
        x,
        y,
        rotated: false,
    }
}

pub static SPREADS: [Spread; 5] = [
    Spread {
        name: "Single Card",
        tab: "unus",
        blurb: "One card for the day, or a quick answer to a focused question.",
        w: 1.0,
        h: 1.0,
        slots: &[s(
            "The Card",
            "The energy or message you most need to hear right now.",
            0.5,
            0.5,
        )],
    },
    Spread {
        name: "Past · Present · Future",
        tab: "tres",
        blurb: "The classic three-card line: where you've been, where you are, where it's heading.",
        w: 3.0,
        h: 1.0,
        slots: &[
            s(
                "Past",
                "Influences and events that led to the current situation.",
                0.5,
                0.5,
            ),
            s(
                "Present",
                "Where things stand right now and the energy around you.",
                1.5,
                0.5,
            ),
            s(
                "Future",
                "Where things are heading if the current path continues.",
                2.5,
                0.5,
            ),
        ],
    },
    Spread {
        name: "Five-Card Cross",
        tab: "crux",
        blurb: "A compact cross that weighs the situation against its roots and its potential.",
        w: 3.0,
        h: 3.0,
        slots: &[
            s(
                "Present",
                "The heart of the matter as it stands today.",
                1.5,
                1.5,
            ),
            s(
                "Past",
                "What has shaped the situation and is now receding.",
                0.5,
                1.5,
            ),
            s("Future", "What is approaching in the near term.", 2.5, 1.5),
            s(
                "Root",
                "The underlying reason or hidden foundation of the question.",
                1.5,
                2.5,
            ),
            s(
                "Potential",
                "The best outcome you can reach toward.",
                1.5,
                0.5,
            ),
        ],
    },
    Spread {
        name: "Horseshoe",
        tab: "solea",
        blurb: "Seven cards in an arch, tracing a situation from its past through advice to the likely outcome.",
        w: 5.0,
        h: 3.0,
        slots: &[
            s(
                "Past",
                "Past events still affecting the question.",
                0.5,
                2.5,
            ),
            s("Present", "Your current circumstances.", 1.0, 1.5),
            s(
                "Hidden Influences",
                "Factors you may not be aware of.",
                1.5,
                0.5,
            ),
            s("Obstacles", "What stands in the way.", 2.5, 0.5),
            s(
                "External Influences",
                "The people and environment around you.",
                3.5,
                0.5,
            ),
            s("Advice", "The recommended approach.", 4.0, 1.5),
            s(
                "Outcome",
                "The most likely result if you follow the advice.",
                4.5,
                2.5,
            ),
        ],
    },
    Spread {
        name: "Celtic Cross",
        tab: "celtica",
        blurb: "The great ten-card spread: a cross for the situation, a staff for its unfolding.",
        w: 5.35,
        h: 4.0,
        slots: &[
            s(
                "Present",
                "The situation at the heart of the reading.",
                1.9,
                2.0,
            ),
            Slot {
                name: "Challenge",
                meaning: "What crosses you — the immediate obstacle or tension.",
                x: 1.9,
                y: 2.0,
                rotated: true,
            },
            s(
                "Foundation",
                "The root of the matter; the deep past beneath it.",
                1.9,
                3.0,
            ),
            s(
                "Recent Past",
                "What is just now passing out of your life.",
                0.5,
                2.0,
            ),
            s(
                "Crown",
                "Your conscious goal — the best that can be achieved.",
                1.9,
                1.0,
            ),
            s("Near Future", "What is about to come into play.", 3.3, 2.0),
            s(
                "Self",
                "Your attitude and the way you see yourself in this.",
                4.85,
                3.5,
            ),
            s(
                "Environment",
                "Outside influences: people, places, circumstances.",
                4.85,
                2.5,
            ),
            s(
                "Hopes & Fears",
                "What you hope for — and what you secretly dread.",
                4.85,
                1.5,
            ),
            s("Outcome", "Where all of this is heading.", 4.85, 0.5),
        ],
    },
];
