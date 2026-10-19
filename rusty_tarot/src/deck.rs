//! The 78-card Rider–Waite–Smith style deck: names, keywords, imagery and meanings.
//!
//! Card ids: 0..22 are the Major Arcana, then 14 cards per suit in the order
//! Wands, Cups, Swords, Pentacles (Ace..Ten, Page, Knight, Queen, King).

use ratatui::style::Color;

pub const DECK_SIZE: usize = 78;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Wands,
    Cups,
    Swords,
    Pentacles,
}

pub const SUITS: [Suit; 4] = [Suit::Wands, Suit::Cups, Suit::Swords, Suit::Pentacles];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Major(u8),
    /// rank 1..=14 (Ace..King)
    Minor(Suit, u8),
}

pub struct CardText {
    pub name: &'static str,
    pub up_keys: &'static [&'static str],
    pub rev_keys: &'static [&'static str],
    pub imagery: &'static str,
    pub upright: &'static str,
    pub reversed: &'static str,
}

macro_rules! card {
    ($name:expr, [$($u:expr),*], [$($r:expr),*], $img:expr, $up:expr, $rev:expr) => {
        CardText { name: $name, up_keys: &[$($u),*], rev_keys: &[$($r),*], imagery: $img, upright: $up, reversed: $rev }
    };
}

pub fn kind(id: usize) -> Kind {
    if id < 22 {
        Kind::Major(id as u8)
    } else {
        let m = id - 22;
        Kind::Minor(SUITS[m / 14], (m % 14) as u8 + 1)
    }
}

pub fn text(id: usize) -> &'static CardText {
    match kind(id) {
        Kind::Major(n) => &MAJORS[n as usize],
        Kind::Minor(s, r) => {
            let r = r as usize - 1;
            match s {
                Suit::Wands => &WANDS[r],
                Suit::Cups => &CUPS[r],
                Suit::Swords => &SWORDS[r],
                Suit::Pentacles => &PENTACLES[r],
            }
        }
    }
}

pub const ROMAN: [&str; 22] = [
    "0", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV",
    "XV", "XVI", "XVII", "XVIII", "XIX", "XX", "XXI",
];

const ASTRO: [&str; 22] = [
    "Uranus · Air",
    "Mercury",
    "The Moon",
    "Venus",
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Jupiter",
    "Libra",
    "Neptune · Water",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Mars",
    "Aquarius",
    "Pisces",
    "The Sun",
    "Pluto · Fire",
    "Saturn",
];

pub struct SuitInfo {
    pub name: &'static str,
    pub glyph: &'static str,
    pub element: &'static str,
    pub domain: &'static str,
    pub color: Color,
}

pub fn suit_info(s: Suit) -> SuitInfo {
    match s {
        Suit::Wands => SuitInfo {
            name: "Wands",
            glyph: "♣",
            element: "Fire",
            domain: "passion, creativity, ambition and action",
            color: Color::Rgb(176, 72, 28),
        },
        Suit::Cups => SuitInfo {
            name: "Cups",
            glyph: "♥",
            element: "Water",
            domain: "emotions, relationships and intuition",
            color: Color::Rgb(36, 78, 140),
        },
        Suit::Swords => SuitInfo {
            name: "Swords",
            glyph: "♠",
            element: "Air",
            domain: "thought, truth, communication and conflict",
            color: Color::Rgb(84, 84, 100),
        },
        Suit::Pentacles => SuitInfo {
            name: "Pentacles",
            glyph: "♦",
            element: "Earth",
            domain: "work, money, health and the material world",
            color: Color::Rgb(58, 108, 44),
        },
    }
}

pub const MAJOR_COLOR: Color = Color::Rgb(150, 98, 18);

pub fn accent(id: usize) -> Color {
    match kind(id) {
        Kind::Major(_) => MAJOR_COLOR,
        Kind::Minor(s, _) => suit_info(s).color,
    }
}

fn rank_short(r: u8) -> &'static str {
    match r {
        1 => "A",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "Pg",
        12 => "Kn",
        13 => "Q",
        _ => "K",
    }
}

/// Short corner label, e.g. "XIII" or "7♥".
pub fn short_label(id: usize) -> String {
    match kind(id) {
        Kind::Major(n) => ROMAN[n as usize].to_string(),
        Kind::Minor(s, r) => format!("{}{}", rank_short(r), suit_info(s).glyph),
    }
}

/// Second line under the name in the detail panel.
pub fn subtitle(id: usize) -> String {
    match kind(id) {
        Kind::Major(n) => format!(
            "Major Arcana · {} · {}",
            ROMAN[n as usize], ASTRO[n as usize]
        ),
        Kind::Minor(s, r) => {
            let si = suit_info(s);
            let court = if r > 10 { "Court · " } else { "" };
            format!(
                "Minor Arcana · {}{} {} · {}",
                court, si.name, si.glyph, si.element
            )
        }
    }
}

pub static MAJORS: [CardText; 22] = [
    card!(
        "The Fool",
        ["beginnings", "innocence", "spontaneity", "leap of faith"],
        ["recklessness", "naivety", "hesitation", "careless risk"],
        "A young traveler steps toward a cliff's edge under a bright sun, a white rose in one hand, a small bundle over the shoulder and a dog yapping at their heels.",
        "A new chapter is opening, and the card asks you to step into it with curiosity rather than a fully worked-out plan. Trust that you'll learn the path by walking it.",
        "Enthusiasm is outrunning judgment — or fear is keeping you frozen at the edge. Look before you leap, but don't let caution stop you entirely."
    ),
    card!(
        "The Magician",
        ["willpower", "skill", "manifestation", "resourcefulness"],
        [
            "manipulation",
            "untapped talent",
            "scattered energy",
            "trickery"
        ],
        "A figure raises a wand to the sky and points to the earth, the infinity sign above their head and the four suit tools laid out on the table before them.",
        "You have every tool you need. Focus your intent, align thought and action, and you can turn an idea into something real.",
        "Talent is being wasted or misdirected. Watch for smoke and mirrors — in others, or in the stories you tell yourself."
    ),
    card!(
        "The High Priestess",
        ["intuition", "mystery", "inner knowing", "the subconscious"],
        [
            "secrets",
            "ignored intuition",
            "withdrawal",
            "surface knowledge"
        ],
        "A veiled priestess sits between a black pillar and a white one, a crescent moon at her feet and a scroll of hidden law half-tucked in her robe.",
        "The answer isn't out there yet — it's in you. Get quiet, pay attention to dreams and hunches, and let things reveal themselves in their own time.",
        "You're tuning out your inner voice, or something is being kept hidden. Pause the noise and listen."
    ),
    card!(
        "The Empress",
        [
            "abundance",
            "nurturing",
            "fertility",
            "creativity",
            "nature"
        ],
        ["creative block", "dependence", "smothering", "self-neglect"],
        "Crowned with twelve stars, the Empress lounges on cushions in a ripening wheat field, a shield bearing the symbol of Venus at her side.",
        "A season of growth and comfort. Create, nurture, enjoy your senses, and let things flourish without forcing them.",
        "Growth has stalled — perhaps because you're pouring care into everyone but yourself, or holding on too tightly."
    ),
    card!(
        "The Emperor",
        ["authority", "structure", "stability", "leadership"],
        [
            "rigidity",
            "domination",
            "lack of discipline",
            "control issues"
        ],
        "A stern ruler sits on a stone throne carved with ram heads, holding an ankh scepter and an orb, barren mountains rising behind him.",
        "Bring order to the chaos. Set clear boundaries, make a plan and take responsibility — structure is what lets things last.",
        "Control has tipped into tyranny, or there's no structure at all. Ask whether the rules are serving you or caging you."
    ),
    card!(
        "The Hierophant",
        ["tradition", "teaching", "institutions", "shared belief"],
        [
            "rebellion",
            "nonconformity",
            "personal beliefs",
            "questioning dogma"
        ],
        "A religious figure in red robes and a triple crown raises a hand in blessing; two acolytes kneel before him and crossed keys lie at his feet.",
        "There's wisdom in established paths. Seek a mentor, learn the fundamentals, or lean on a community that shares your values.",
        "The old rules no longer fit. You may need to forge your own way and question inherited beliefs."
    ),
    card!(
        "The Lovers",
        ["love", "harmony", "choices", "aligned values"],
        [
            "imbalance",
            "disharmony",
            "misalignment",
            "avoiding a choice"
        ],
        "A man and a woman stand beneath a radiant angel; behind her grows a serpent-wrapped tree of knowledge, behind him a tree of flames.",
        "A meaningful bond or a values-driven decision. Choose what is genuinely aligned with who you are, not just what is easy.",
        "Something is out of sync — in a relationship or within yourself. Revisit your values before committing."
    ),
    card!(
        "The Chariot",
        ["determination", "victory", "control", "momentum"],
        [
            "no direction",
            "aggression",
            "scattered effort",
            "loss of control"
        ],
        "An armored driver rides a canopied chariot pulled by a black sphinx and a white one, steering them by will alone with no reins.",
        "Victory through focus. Harness opposing forces, commit to a direction and drive forward with confidence.",
        "You're pulled in two directions or pushing too hard. Regain control of the reins before charging ahead."
    ),
    card!(
        "Strength",
        ["courage", "compassion", "inner strength", "patience"],
        ["self-doubt", "weakness", "insecurity", "raw emotion"],
        "A woman crowned with flowers gently closes the jaws of a lion, the infinity sign floating above her head.",
        "True strength is gentle. Meet challenges with calm courage and compassion — including toward your own wilder impulses.",
        "Doubt or frustration has the upper hand. Reconnect with your resilience; you're stronger than you feel."
    ),
    card!(
        "The Hermit",
        ["introspection", "solitude", "inner guidance", "wisdom"],
        ["isolation", "loneliness", "withdrawal", "lost way"],
        "A cloaked old man stands alone on a snowy peak, holding up a lantern that contains a six-pointed star.",
        "Step back from the crowd to find your own light. Time alone, reflection or study will show you the next step.",
        "Solitude has become isolation — or you're avoiding necessary reflection. Reach out, or turn inward, whichever you've been resisting."
    ),
    card!(
        "Wheel of Fortune",
        ["cycles", "change", "fate", "turning point", "luck"],
        ["bad luck", "resisting change", "repeating cycles"],
        "A great wheel inscribed with letters and alchemical symbols turns in the sky, a sphinx atop it and winged creatures reading books in the four corners.",
        "The wheel is turning. Change is coming and it may well be in your favor — ride the momentum and stay adaptable.",
        "A downturn or a stubborn loop. What goes down comes up again; look for the pattern you keep repeating."
    ),
    card!(
        "Justice",
        ["fairness", "truth", "cause and effect", "law"],
        ["unfairness", "dishonesty", "no accountability"],
        "A crowned figure sits between two pillars holding an upright double-edged sword in one hand and balanced scales in the other.",
        "Actions have consequences. Be honest, weigh things fairly, and a just outcome follows. Legal matters lean toward the truth.",
        "Something is unbalanced or someone is dodging responsibility — possibly you. Own your part."
    ),
    card!(
        "The Hanged Man",
        ["surrender", "pause", "new perspective", "letting go"],
        ["stalling", "resistance", "indecision", "needless sacrifice"],
        "A man hangs upside down by one foot from a living T-shaped tree, hands behind his back, his face calm and a halo glowing around his head.",
        "Stop pushing. A pause or a voluntary sacrifice lets you see the situation from a completely new angle.",
        "You're stuck in limbo, resisting the very surrender that would free you. Decide, or let go."
    ),
    card!(
        "Death",
        ["endings", "transformation", "transition", "clearing away"],
        ["resisting change", "stagnation", "fear of endings"],
        "A skeleton in black armor rides a white horse beneath a banner of a white rose; a king lies fallen, a bishop pleads, and the sun rises between two towers.",
        "Rarely literal — this is an ending that makes room for something new. Let the old chapter close cleanly.",
        "Clinging to what's already over is keeping you stuck. The change is coming anyway; meet it."
    ),
    card!(
        "Temperance",
        ["balance", "moderation", "patience", "blending"],
        ["imbalance", "excess", "impatience", "misalignment"],
        "A winged angel with one foot on land and one in water pours liquid between two cups, a path winding toward a distant crown of light.",
        "Find the middle way. Blend opposites patiently and things come together into something better than either part.",
        "You're overdoing something, or rushing. Recalibrate and restore balance."
    ),
    card!(
        "The Devil",
        ["attachment", "addiction", "shadow self", "materialism"],
        ["release", "breaking free", "reclaiming power", "detachment"],
        "A horned, bat-winged figure perches on a pedestal; below, a naked man and woman are chained to it — but the chains are loose enough to lift off.",
        "Something has a hold on you — a habit, a relationship, a belief. Notice that the chains are looser than they seem.",
        "You're recognizing the trap and beginning to free yourself. Keep going."
    ),
    card!(
        "The Tower",
        ["sudden upheaval", "revelation", "chaos", "awakening"],
        ["averting disaster", "fear of change", "delayed collapse"],
        "Lightning strikes a tall tower on a rocky peak and blasts off its crown; flames burst from the windows and two figures fall headlong.",
        "A sudden shake-up topples whatever was built on false foundations. It's jarring, but the truth it reveals is liberating.",
        "You may be resisting an inevitable change or narrowly avoiding one. Better to rebuild deliberately than wait for the lightning."
    ),
    card!(
        "The Star",
        ["hope", "renewal", "inspiration", "serenity"],
        ["despair", "disconnection", "lost faith", "discouragement"],
        "A woman kneels at a pool beneath a great eight-pointed star, pouring water onto the land and back into the pool, seven smaller stars around her.",
        "After the storm, healing. Hope returns and you can trust the future again — keep your vision bright.",
        "You've lost sight of your light. Small acts of self-care and faith can reignite it."
    ),
    card!(
        "The Moon",
        ["illusion", "intuition", "dreams", "uncertainty"],
        ["fear released", "clarity emerging", "confusion lifting"],
        "A full moon with a solemn face hangs between two towers; a dog and a wolf howl at it while a crayfish crawls from a pool onto a winding path.",
        "Not everything is as it seems. Move carefully through the uncertainty, trust your intuition, and don't let fear invent monsters.",
        "The fog is lifting. Confusion or anxiety is easing and the truth is surfacing."
    ),
    card!(
        "The Sun",
        ["joy", "success", "vitality", "clarity"],
        ["temporary setback", "dimmed enthusiasm", "overconfidence"],
        "A radiant sun with a face shines on a joyful child riding a white horse, a red banner in hand and sunflowers blooming over a garden wall.",
        "One of the brightest cards in the deck. Success, warmth and clarity — enjoy it and share it.",
        "The sun is still there, just behind clouds. Adjust expectations a little, or check that optimism isn't blinding you."
    ),
    card!(
        "Judgement",
        ["reckoning", "rebirth", "inner calling", "absolution"],
        ["self-doubt", "harsh self-judgment", "ignoring the call"],
        "An angel blows a trumpet from the clouds; below, men, women and children rise from open coffins with their arms outstretched.",
        "A moment of awakening. Review the past honestly, forgive it, and answer the calling you've been hearing.",
        "You're judging yourself too harshly, or ignoring a clear call to change. Listen."
    ),
    card!(
        "The World",
        ["completion", "integration", "accomplishment", "wholeness"],
        ["incompletion", "no closure", "shortcuts", "delays"],
        "A dancing figure wrapped in a sash floats inside a laurel wreath holding two wands, with an angel, an eagle, a bull and a lion in the four corners.",
        "A cycle reaches its fulfilling end. Celebrate what you've achieved — the next journey starts from this wholeness.",
        "You're nearly there but something is unfinished. Tie up the loose ends before moving on."
    ),
];

pub static WANDS: [CardText; 14] = [
    card!(
        "Ace of Wands",
        ["inspiration", "new venture", "potential", "spark"],
        ["delays", "low motivation", "false start"],
        "A hand emerges from a cloud gripping a sprouting wand, leaves drifting down, a castle on a distant hill.",
        "A spark of inspiration or a burst of energy — a new project, passion or opportunity is ready to be lit.",
        "The fire won't catch. Energy is blocked or the timing isn't right yet."
    ),
    card!(
        "Two of Wands",
        ["planning", "future vision", "decisions", "discovery"],
        ["fear of the unknown", "poor planning", "playing safe"],
        "A man on a castle rampart holds a small globe and a wand, looking out over the sea while a second wand stands fixed to the wall.",
        "You've taken the first step; now plan the bigger move. The world is wider than your current walls.",
        "Comfort is holding you back from expanding, or the plan lacks substance."
    ),
    card!(
        "Three of Wands",
        ["expansion", "foresight", "progress", "returns"],
        ["obstacles", "delays", "frustration", "limited vision"],
        "A figure stands on a cliff beside three planted wands, watching ships sail across a golden sea.",
        "Your efforts are paying off and new horizons are opening. Think long-term.",
        "Delays or setbacks; the plan may need adjusting."
    ),
    card!(
        "Four of Wands",
        ["celebration", "harmony", "homecoming", "milestone"],
        ["transition", "lack of support", "tension at home"],
        "Four wands hung with a garland of flowers frame two figures raising bouquets in front of a castle.",
        "A joyful milestone — celebrate it with the people who matter.",
        "The celebration is muted, or home life feels unsettled."
    ),
    card!(
        "Five of Wands",
        ["competition", "conflict", "rivalry", "tension"],
        ["avoiding conflict", "resolution", "inner conflict"],
        "Five young men brandish wands in a chaotic scuffle, no one clearly winning.",
        "Clashing egos or friendly competition. Channel the friction into growth.",
        "Conflict is winding down — or being dodged when it really needs to be faced."
    ),
    card!(
        "Six of Wands",
        ["victory", "recognition", "success", "confidence"],
        ["ego", "fall from grace", "lack of recognition"],
        "A rider crowned with laurel parades on a white horse through a cheering crowd, a wreathed wand held high.",
        "Public recognition and success. You've earned it — enjoy the moment.",
        "Validation is missing, or pride is setting up a fall."
    ),
    card!(
        "Seven of Wands",
        ["perseverance", "defense", "standing firm", "challenge"],
        ["overwhelm", "giving up", "exhaustion"],
        "A man on high ground fends off six wands rising from below, wearing mismatched shoes.",
        "Hold your position. You have the advantage — defend what you believe in.",
        "You're worn down and tempted to surrender. Pick your battles."
    ),
    card!(
        "Eight of Wands",
        ["speed", "movement", "swift action", "news"],
        ["delays", "frustration", "waiting", "slowing down"],
        "Eight wands fly through a clear sky over a river, all heading the same way.",
        "Things are moving fast. Messages arrive and plans accelerate — act quickly.",
        "Momentum stalls, or hurry leads to mistakes."
    ),
    card!(
        "Nine of Wands",
        ["resilience", "persistence", "last stand", "boundaries"],
        ["paranoia", "exhaustion", "defensiveness"],
        "A bandaged man leans on a wand, warily eyeing the eight wands standing behind him like a fence.",
        "You're battered but nearly there. Keep going — one final push.",
        "You're so guarded it's becoming exhausting. Rest, or ask for help."
    ),
    card!(
        "Ten of Wands",
        ["burden", "responsibility", "overload", "hard work"],
        ["letting go", "delegating", "burnout"],
        "A man staggers toward a town carrying ten heavy wands bundled in his arms, his view blocked.",
        "You've taken on too much. Success has come with heavy responsibility.",
        "Time to put some of it down — delegate, or drop what isn't yours to carry."
    ),
    card!(
        "Page of Wands",
        ["enthusiasm", "exploration", "curiosity", "free spirit"],
        ["haste", "no direction", "procrastination"],
        "A young person in a salamander-patterned tunic studies a sprouting wand in a desert landscape.",
        "Exciting news or a new idea that fires your curiosity. Explore it.",
        "Lots of ideas, little follow-through."
    ),
    card!(
        "Knight of Wands",
        ["energy", "passion", "adventure", "impulsiveness"],
        ["recklessness", "haste", "frustration"],
        "A knight on a rearing horse charges across the desert, wand raised and plume flaming.",
        "Bold action and adventure. Go after it with gusto.",
        "Impulsiveness burns out fast — slow down and aim."
    ),
    card!(
        "Queen of Wands",
        ["confidence", "warmth", "determination", "charisma"],
        ["jealousy", "insecurity", "demanding"],
        "A queen sits on a lion-carved throne holding a wand and a sunflower, a black cat at her feet.",
        "Radiate confidence and warmth. You inspire others simply by being yourself.",
        "Self-doubt or jealousy is dimming your fire."
    ),
    card!(
        "King of Wands",
        ["leadership", "vision", "entrepreneurship", "honor"],
        ["impulsiveness", "overbearing", "impossible standards"],
        "A king on a throne decorated with lions and salamanders holds a blossoming wand, gazing into the distance.",
        "Visionary leadership. Take the lead with a bold plan and inspire others to follow.",
        "Ruthless or impatient leadership; expectations no one can meet."
    ),
];

pub static CUPS: [CardText; 14] = [
    card!(
        "Ace of Cups",
        [
            "new love",
            "compassion",
            "emotional awakening",
            "creativity"
        ],
        ["blocked emotions", "repressed feelings", "emptiness"],
        "A hand from a cloud offers an overflowing cup; a dove descends with a wafer and five streams pour into a lotus pond.",
        "Your heart opens — new love, deep feeling or creative flow.",
        "Feelings are bottled up, or the cup feels empty."
    ),
    card!(
        "Two of Cups",
        ["partnership", "unity", "attraction", "mutual respect"],
        ["imbalance", "broken communication", "tension"],
        "Two figures exchange cups beneath a winged lion's head rising from a caduceus.",
        "A meaningful connection — romance, friendship or partnership — built on mutual respect.",
        "A relationship is out of balance; reconnect or renegotiate."
    ),
    card!(
        "Three of Cups",
        ["friendship", "celebration", "community", "joy"],
        ["overindulgence", "gossip", "isolation"],
        "Three women raise their cups in a dance among a harvest of fruit and flowers.",
        "Gather your people and celebrate. Community lifts you up.",
        "Too much partying, or friction within the friend group."
    ),
    card!(
        "Four of Cups",
        ["apathy", "contemplation", "reevaluation", "missed offer"],
        ["renewed interest", "motivation", "acceptance"],
        "A young man sits under a tree, arms crossed, staring at three cups while a hand from a cloud offers him a fourth.",
        "Discontent is making you blind to what's being offered. Look up.",
        "Coming out of a funk and ready to engage again."
    ),
    card!(
        "Five of Cups",
        ["loss", "grief", "regret", "disappointment"],
        ["acceptance", "moving on", "forgiveness"],
        "A cloaked figure mourns three spilled cups, not yet noticing the two still standing behind them.",
        "Grief is real — allow it. But not everything is lost.",
        "Healing begins; you're ready to turn toward what remains."
    ),
    card!(
        "Six of Cups",
        ["nostalgia", "childhood", "innocence", "reunion"],
        ["stuck in the past", "rose-tinted memories"],
        "Children exchange cups filled with white flowers in a peaceful old village courtyard.",
        "Sweet memories, old friends, or simple kindness.",
        "Living in the past is keeping you from the present."
    ),
    card!(
        "Seven of Cups",
        ["choices", "illusion", "fantasy", "wishful thinking"],
        ["clarity", "decisiveness", "reality check"],
        "A figure faces seven cups floating in clouds, each holding a different vision — jewels, a castle, a snake, a dragon, a veiled head.",
        "Many options, some of them mirages. Separate fantasy from real possibility.",
        "The fog clears and you can choose with clarity."
    ),
    card!(
        "Eight of Cups",
        ["walking away", "disillusionment", "seeking meaning"],
        ["fear of change", "avoidance", "aimless drifting"],
        "A cloaked traveler walks away from eight stacked cups toward rugged mountains under an eclipsed moon.",
        "Something no longer fulfills you. It's okay to leave and look for more.",
        "You know you should go but fear keeps you put — or you keep leaving without a direction."
    ),
    card!(
        "Nine of Cups",
        ["contentment", "wish fulfilled", "satisfaction", "gratitude"],
        ["smugness", "dissatisfaction", "materialism"],
        "A well-fed man sits contentedly, arms crossed, in front of a curved table of nine cups.",
        "The \"wish card\" — what you hoped for comes true. Enjoy it.",
        "Getting what you wanted didn't satisfy; look deeper."
    ),
    card!(
        "Ten of Cups",
        ["harmony", "family", "happiness", "fulfillment"],
        ["broken home", "misaligned values", "disconnection"],
        "A couple embraces beneath a rainbow of ten cups while their children dance beside a cozy home.",
        "Lasting happiness and harmony with the people you love.",
        "The picture-perfect ideal is cracking; tend to what really matters."
    ),
    card!(
        "Page of Cups",
        ["creative opportunity", "curiosity", "intuitive message"],
        ["emotional immaturity", "creative block", "insecurity"],
        "A young person by the sea holds up a cup from which a small fish peeks out.",
        "A sweet surprise, an intuitive nudge or a creative idea arrives. Stay open.",
        "Feelings are getting the better of you; mind your inner child."
    ),
    card!(
        "Knight of Cups",
        ["romance", "charm", "imagination", "following the heart"],
        ["moodiness", "unrealistic", "jealousy"],
        "A knight on a calm white horse holds a cup before him like an offering as he crosses a gentle stream.",
        "An offer of love or inspiration — follow your heart with grace.",
        "Charm without substance; emotions running hot and cold."
    ),
    card!(
        "Queen of Cups",
        ["compassion", "calm", "intuition", "emotional security"],
        ["codependence", "insecurity", "martyrdom"],
        "A queen sits on a throne at the water's edge, gazing into an ornate, closed cup.",
        "Lead with empathy and emotional intelligence. Trust your intuition.",
        "Caring for everyone else is draining you; set some boundaries."
    ),
    card!(
        "King of Cups",
        ["emotional balance", "diplomacy", "wisdom", "generosity"],
        ["manipulation", "moodiness", "coldness"],
        "A king on a throne floating on a turbulent sea remains perfectly calm, holding a cup and a scepter.",
        "Steady and compassionate — feeling deeply while staying in control.",
        "Emotions are being suppressed, or used to manipulate."
    ),
];

pub static SWORDS: [CardText; 14] = [
    card!(
        "Ace of Swords",
        ["clarity", "breakthrough", "truth", "new idea"],
        ["confusion", "chaos", "misinformation"],
        "A hand from a cloud grips an upright sword crowned with a wreath, above jagged mountains.",
        "A breakthrough — sharp clarity cuts through confusion. Speak the truth.",
        "Muddled thinking or misused words; get your facts straight."
    ),
    card!(
        "Two of Swords",
        ["indecision", "stalemate", "avoidance", "hard choice"],
        ["information overload", "confusion", "lesser of two evils"],
        "A blindfolded woman sits by the sea holding two crossed swords over her heart.",
        "You're blocking out information to avoid a hard choice. Take off the blindfold.",
        "Pressure to decide is mounting and anxiety muddles everything."
    ),
    card!(
        "Three of Swords",
        ["heartbreak", "grief", "emotional pain", "sorrow"],
        ["recovery", "forgiveness", "releasing pain"],
        "Three swords pierce a red heart against a gray, rainy sky.",
        "Painful truth or heartbreak. Let yourself feel it — clarity often follows hurt.",
        "Healing begins; you're releasing old pain."
    ),
    card!(
        "Four of Swords",
        ["rest", "recovery", "contemplation", "retreat"],
        ["restlessness", "burnout", "stagnation"],
        "A knight's effigy lies in prayer on a tomb, one sword beneath him and three hanging on the wall above.",
        "Rest and recuperate. Withdraw for a while before the next challenge.",
        "You need rest but you're fighting it — or you've been withdrawn too long."
    ),
    card!(
        "Five of Swords",
        ["conflict", "defeat", "winning at all costs", "tension"],
        ["reconciliation", "making amends", "old resentment"],
        "A smirking man gathers up swords while two defeated figures walk away under a jagged sky.",
        "A hollow victory — is winning worth the damage?",
        "Ready to make peace and let resentment go."
    ),
    card!(
        "Six of Swords",
        [
            "transition",
            "moving on",
            "leaving behind",
            "rite of passage"
        ],
        ["emotional baggage", "resistance", "unfinished business"],
        "A ferryman poles a boat carrying a cloaked woman and child toward calmer waters, six swords standing in the bow.",
        "Leaving troubled waters for calmer shores. The change is helping.",
        "Baggage follows you; resolve the past before moving on."
    ),
    card!(
        "Seven of Swords",
        ["deception", "strategy", "sneaking away", "cunning"],
        ["confession", "conscience", "getting caught"],
        "A man tiptoes away from a camp carrying five swords, glancing back at the two he left behind.",
        "Someone is being sneaky — maybe you. Or a clever strategy is called for.",
        "Secrets come to light; honesty is the better path."
    ),
    card!(
        "Eight of Swords",
        ["restriction", "self-imposed limits", "feeling trapped"],
        ["release", "new perspective", "freedom"],
        "A bound and blindfolded woman stands among eight swords stuck in marshy ground — but the way out is open.",
        "You feel trapped, but the limits are largely in your mind.",
        "You're seeing the exit and starting to walk free."
    ),
    card!(
        "Nine of Swords",
        ["anxiety", "worry", "nightmares", "fear"],
        ["hope", "reaching out", "releasing worry"],
        "A figure sits up in bed with face in hands, nine swords hanging on the dark wall behind them.",
        "Worry is keeping you up at night; often the fear is bigger than the reality.",
        "The worst has passed. Talk it out."
    ),
    card!(
        "Ten of Swords",
        ["painful ending", "rock bottom", "betrayal", "crisis"],
        ["recovery", "regeneration", "resisting the end"],
        "A man lies face down pierced by ten swords as a golden dawn breaks over a calm sea.",
        "A painful ending — but it is the end. The only way from here is up.",
        "Recovery begins; don't reopen the wound."
    ),
    card!(
        "Page of Swords",
        ["curiosity", "new ideas", "communication", "vigilance"],
        ["gossip", "deception", "all talk"],
        "A young person stands on windy high ground, sword raised, watching the clouds.",
        "Hungry for knowledge and truth. Stay alert and ask questions.",
        "Sharp words or idle chatter cause trouble."
    ),
    card!(
        "Knight of Swords",
        ["ambition", "action", "fast thinking", "drive"],
        ["impulsive", "aggressive", "no direction"],
        "A knight charges into the wind on a galloping horse, sword drawn, storm clouds behind.",
        "Charge ahead with conviction — fast decisions and strong opinions.",
        "Rushing in without thinking; mind the collateral damage."
    ),
    card!(
        "Queen of Swords",
        ["clarity", "independence", "honest words", "perception"],
        ["coldness", "bitterness", "cruelty"],
        "A queen on a high throne holds an upright sword and extends an open hand, clouds gathered below her.",
        "See clearly and speak plainly. Independent thinking and honest boundaries.",
        "Sharpness turning into bitterness or harshness."
    ),
    card!(
        "King of Swords",
        ["intellect", "authority", "truth", "ethics"],
        ["abuse of power", "manipulation", "cold logic"],
        "A king sits facing forward with an upright sword, butterflies carved into his throne.",
        "Lead with reason, fairness and clear principles.",
        "Intellect used to control, or rules applied without compassion."
    ),
];

pub static PENTACLES: [CardText; 14] = [
    card!(
        "Ace of Pentacles",
        ["opportunity", "prosperity", "new venture", "manifestation"],
        ["lost opportunity", "poor planning", "scarcity mindset"],
        "A hand from a cloud offers a golden pentacle above a lush garden with a hedge archway.",
        "A new financial or material opportunity — plant the seed.",
        "An opportunity slips away; check your plans and finances."
    ),
    card!(
        "Two of Pentacles",
        ["balance", "adaptability", "juggling priorities"],
        ["overwhelm", "disorganization", "overcommitment"],
        "A young man juggles two pentacles linked by an infinity loop as ships ride big waves behind him.",
        "Keeping many balls in the air — stay flexible.",
        "Too much at once; drop something before everything falls."
    ),
    card!(
        "Three of Pentacles",
        ["teamwork", "collaboration", "skill", "learning"],
        ["disharmony", "poor teamwork", "misalignment"],
        "A stonemason works on a cathedral arch while a monk and a nobleman consult the plans.",
        "Collaboration and craft — your skills are recognized as part of a team.",
        "Team friction or sloppy work; regroup."
    ),
    card!(
        "Four of Pentacles",
        ["security", "control", "saving", "conservatism"],
        ["greed", "materialism", "letting go"],
        "A crowned man clutches a pentacle to his chest, with one on his crown and one under each foot.",
        "Holding on to what you have — security is good, but don't hoard.",
        "Loosen your grip; scarcity thinking is costing you."
    ),
    card!(
        "Five of Pentacles",
        ["hardship", "loss", "isolation", "poverty"],
        ["recovery", "help received", "improvement"],
        "Two ragged figures trudge through the snow past a glowing stained-glass church window.",
        "A hard time, materially or emotionally — but help is closer than you think.",
        "The worst is behind you; recovery is underway."
    ),
    card!(
        "Six of Pentacles",
        ["generosity", "charity", "giving and receiving"],
        ["strings attached", "debt", "inequality"],
        "A merchant holds balanced scales while giving coins to two kneeling beggars.",
        "Give and receive with an open hand — balance in resources.",
        "Generosity with strings attached, or a lopsided exchange."
    ),
    card!(
        "Seven of Pentacles",
        ["patience", "long-term view", "investment", "assessment"],
        ["impatience", "poor returns", "wasted effort"],
        "A farmer leans on his hoe and gazes at seven pentacles growing on a lush vine.",
        "Your hard work is growing. Assess your progress and keep investing.",
        "Frustration at slow results — or the investment isn't paying off."
    ),
    card!(
        "Eight of Pentacles",
        ["diligence", "craftsmanship", "mastery", "practice"],
        ["perfectionism", "lack of focus", "uninspired work"],
        "A craftsman sits at his bench carving pentacles, finished ones displayed on a post beside him.",
        "Dedicated practice — keep honing your craft.",
        "Mindless repetition or perfectionism; reconnect with your purpose."
    ),
    card!(
        "Nine of Pentacles",
        ["abundance", "luxury", "self-sufficiency", "independence"],
        ["overworking", "financial setback", "superficiality"],
        "An elegant woman stands in a vineyard with a hooded falcon on her gloved hand.",
        "You've earned this comfort through your own effort — enjoy it.",
        "Success built on overwork or appearances."
    ),
    card!(
        "Ten of Pentacles",
        ["wealth", "legacy", "family", "long-term success"],
        ["financial failure", "loss", "family disputes"],
        "An old man sits with his dogs beneath an archway as a family gathers, ten pentacles arranged in a tree of life.",
        "Lasting wealth and legacy — what you build now can last generations.",
        "Family or financial foundations are shaky."
    ),
    card!(
        "Page of Pentacles",
        [
            "ambition",
            "desire to learn",
            "new opportunity",
            "diligence"
        ],
        ["no progress", "procrastination", "missed chance"],
        "A young person stands in a flowering field gazing intently at a pentacle held aloft.",
        "A practical new beginning — study, an apprenticeship, a new job.",
        "All plans and no action."
    ),
    card!(
        "Knight of Pentacles",
        ["hard work", "routine", "reliability", "patience"],
        ["laziness", "boredom", "stuck in a rut"],
        "A knight sits motionless on a heavy black horse, holding a pentacle and surveying a plowed field.",
        "Slow and steady wins. Reliability and consistent effort.",
        "Routine has turned into a rut."
    ),
    card!(
        "Queen of Pentacles",
        ["nurturing", "practicality", "abundance", "down-to-earth"],
        ["self-neglect", "work–home imbalance", "smothering"],
        "A queen sits in a flower-covered garden cradling a pentacle in her lap, a rabbit at her feet.",
        "Practical care — a comfortable home, a healthy body, resources used wisely.",
        "Taking care of everything except yourself."
    ),
    card!(
        "King of Pentacles",
        ["wealth", "security", "discipline", "abundance"],
        ["greed", "stubbornness", "materialism"],
        "A king in a robe embroidered with grapes sits on a bull-carved throne, a pentacle on his knee and his castle behind him.",
        "Material mastery and a steady hand — build wealth responsibly.",
        "Obsession with status, or controlling money too tightly."
    ),
];
