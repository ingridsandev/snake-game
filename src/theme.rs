//! Visual themes, including holiday themes that switch on automatically by date.
//!
//! Pure data + date math: nothing here touches the terminal.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use crossterm::style::Color;

/// Something the snake can eat. `emoji` is drawn by default; `ascii` (with
/// `color`) is the `--ascii` fallback. Both must be exactly 2 columns wide —
/// one board cell.
pub struct FoodGlyph {
    pub emoji: &'static str,
    pub ascii: &'static str,
    pub color: Color,
}

pub struct Theme {
    /// Name used on the command line (`--theme <name>`).
    pub name: &'static str,
    /// Shown in the top border.
    pub title: &'static str,
    pub title_ascii: &'static str,
    /// When it switches on automatically, for `--theme list`.
    pub when: &'static str,
    pub head: Color,
    /// Body segments cycle through these, giving stripes.
    pub body: &'static [Color],
    pub border: Color,
    pub food: &'static [FoodGlyph],
    pub game_over: &'static str,
    pub win: &'static str,
}

/// Themes are identified by name: two themes are equal if their names are.
impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Theme({})", self.name)
    }
}

// 256-color palette indices: widely supported, unlike 24-bit RGB.
const ORANGE: Color = Color::AnsiValue(208);
const PURPLE: Color = Color::AnsiValue(93);
const PINK: Color = Color::AnsiValue(213);
const HOT_PINK: Color = Color::AnsiValue(198);
const GOLD: Color = Color::AnsiValue(220);
const BROWN: Color = Color::AnsiValue(130);
const KELLY: Color = Color::AnsiValue(34);
const LIME: Color = Color::AnsiValue(118);
const PASTEL_BLUE: Color = Color::AnsiValue(153);
const PASTEL_PINK: Color = Color::AnsiValue(218);
const PASTEL_YELLOW: Color = Color::AnsiValue(229);
const PASTEL_GREEN: Color = Color::AnsiValue(157);

pub const CLASSIC: Theme = Theme {
    name: "classic",
    title: " 🐍 Snake ",
    title_ascii: " Snake ",
    when: "any other day",
    head: Color::Yellow,
    body: &[Color::Green],
    border: Color::DarkGrey,
    food: &[FoodGlyph { emoji: "🍎", ascii: "◆ ", color: Color::Red }],
    game_over: "The snake bit the dust.",
    win: "Snake supreme!",
};

pub const HALLOWEEN: Theme = Theme {
    name: "halloween",
    title: " 🎃 Spooky Snake ",
    title_ascii: " Spooky Snake ",
    when: "Oct 24–31",
    head: ORANGE,
    body: &[ORANGE, PURPLE],
    border: PURPLE,
    food: &[
        FoodGlyph { emoji: "🎃", ascii: "()", color: ORANGE },
        FoodGlyph { emoji: "🍬", ascii: "><", color: PINK },
        FoodGlyph { emoji: "👻", ascii: "oo", color: Color::White },
    ],
    game_over: "The ghosts got you.",
    win: "You out-spooked the spooks!",
};

pub const CHRISTMAS: Theme = Theme {
    name: "christmas",
    title: " 🎄 Candy Cane Snake ",
    title_ascii: " Candy Cane Snake ",
    when: "Dec 1–26",
    head: Color::Red,
    body: &[Color::White, Color::Red],
    border: Color::DarkGreen,
    food: &[
        FoodGlyph { emoji: "🎁", ascii: "[]", color: Color::Red },
        FoodGlyph { emoji: "🍪", ascii: "()", color: BROWN },
        FoodGlyph { emoji: "⭐", ascii: "**", color: GOLD },
    ],
    game_over: "Coal for you this year.",
    win: "Santa's favourite snake!",
};

pub const NEW_YEAR: Theme = Theme {
    name: "new-year",
    title: " 🎆 New Year Snake ",
    title_ascii: " New Year Snake ",
    when: "Dec 31 – Jan 1",
    head: Color::White,
    body: &[GOLD, Color::White],
    border: GOLD,
    food: &[
        FoodGlyph { emoji: "🎆", ascii: "**", color: GOLD },
        FoodGlyph { emoji: "🥂", ascii: "YY", color: PASTEL_YELLOW },
        FoodGlyph { emoji: "🎉", ascii: "<>", color: Color::Magenta },
    ],
    game_over: "Better luck in the new year!",
    win: "What a way to start the year!",
};

pub const VALENTINES: Theme = Theme {
    name: "valentines",
    title: " 💘 Snake in Love ",
    title_ascii: " Snake in Love ",
    when: "Feb 14",
    head: HOT_PINK,
    body: &[PINK, Color::Red],
    border: PINK,
    food: &[
        FoodGlyph { emoji: "💖", ascii: "<3", color: HOT_PINK },
        FoodGlyph { emoji: "🍫", ascii: "[]", color: BROWN },
        FoodGlyph { emoji: "🌹", ascii: "@}", color: Color::Red },
    ],
    game_over: "Heartbroken.",
    win: "True love conquers all!",
};

pub const ST_PATRICKS: Theme = Theme {
    name: "st-patricks",
    title: " 🍀 Lucky Snake ",
    title_ascii: " Lucky Snake ",
    when: "Mar 17",
    head: GOLD,
    body: &[KELLY, LIME],
    border: KELLY,
    food: &[
        FoodGlyph { emoji: "🍀", ascii: "%%", color: LIME },
        FoodGlyph { emoji: "🪙", ascii: "()", color: GOLD },
        FoodGlyph { emoji: "🌈", ascii: "~~", color: Color::Magenta },
    ],
    game_over: "Out of luck.",
    win: "You found the pot of gold!",
};

pub const EASTER: Theme = Theme {
    name: "easter",
    title: " 🐣 Easter Snake ",
    title_ascii: " Easter Snake ",
    when: "Good Friday – Easter Monday",
    head: PASTEL_YELLOW,
    body: &[PASTEL_BLUE, PASTEL_PINK, PASTEL_GREEN],
    border: PASTEL_PINK,
    food: &[
        FoodGlyph { emoji: "🥚", ascii: "()", color: PASTEL_BLUE },
        FoodGlyph { emoji: "🐣", ascii: "^^", color: PASTEL_YELLOW },
        FoodGlyph { emoji: "🥕", ascii: "V ", color: ORANGE },
    ],
    game_over: "The bunny wins.",
    win: "Every egg found!",
};

pub const THANKSGIVING: Theme = Theme {
    name: "thanksgiving",
    title: " 🦃 Gobble Snake ",
    title_ascii: " Gobble Snake ",
    when: "4th Thursday of November",
    head: ORANGE,
    body: &[BROWN, ORANGE],
    border: BROWN,
    food: &[
        FoodGlyph { emoji: "🥧", ascii: "()", color: ORANGE },
        FoodGlyph { emoji: "🌽", ascii: "||", color: GOLD },
        FoodGlyph { emoji: "🦃", ascii: "Q>", color: BROWN },
    ],
    game_over: "Stuffed.",
    win: "Thankful for this run!",
};

pub const CARNAVAL: Theme = Theme {
    name: "carnaval",
    title: " 🎭 Cobra do Carnaval ",
    title_ascii: " Cobra do Carnaval ",
    when: "Saturday – Ash Wednesday before Lent",
    head: Color::White,
    body: &[GOLD, KELLY, Color::Magenta, Color::Cyan, ORANGE, PURPLE],
    border: GOLD,
    food: &[
        FoodGlyph { emoji: "🎭", ascii: "=)", color: GOLD },
        FoodGlyph { emoji: "🥁", ascii: "[]", color: Color::Red },
        FoodGlyph { emoji: "🎉", ascii: "<>", color: Color::Magenta },
        FoodGlyph { emoji: "🍹", ascii: "Y ", color: ORANGE },
    ],
    game_over: "A festa acabou!",
    win: "É campeã da Sapucaí!",
};

/// Every theme, classic first. Order is what `--theme list` shows.
pub const ALL: &[&Theme] = &[
    &CLASSIC,
    &CARNAVAL,
    &VALENTINES,
    &ST_PATRICKS,
    &EASTER,
    &HALLOWEEN,
    &THANKSGIVING,
    &CHRISTMAS,
    &NEW_YEAR,
];

pub fn by_name(name: &str) -> Option<&'static Theme> {
    ALL.iter().copied().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// The holiday theme for `date`, if it falls inside a holiday window.
///
/// Windows (inclusive):
/// - Carnaval: Saturday → Ash Wednesday (Easter − 50 … − 46 days)
/// - Valentine's: Feb 14
/// - St Patrick's: Mar 17
/// - Easter: Good Friday → Easter Monday
/// - Halloween: Oct 24 → 31
/// - Thanksgiving (US): 4th Thursday of November
/// - Christmas: Dec 1 → 26
/// - New Year: Dec 31 → Jan 1
///
/// Carnaval is checked first: it can land on Feb 14 (e.g. 2026) and wins.
pub fn holiday_for(date: NaiveDate) -> Option<&'static Theme> {
    let (y, m, d) = (date.year(), date.month(), date.day());
    let easter = easter_sunday(y);

    let carnaval_start = easter - Duration::days(50);
    let ash_wednesday = easter - Duration::days(46);
    if (carnaval_start..=ash_wednesday).contains(&date) {
        return Some(&CARNAVAL);
    }
    if (easter - Duration::days(2)..=easter + Duration::days(1)).contains(&date) {
        return Some(&EASTER);
    }
    if date == thanksgiving(y) {
        return Some(&THANKSGIVING);
    }
    match (m, d) {
        (2, 14) => Some(&VALENTINES),
        (3, 17) => Some(&ST_PATRICKS),
        (10, 24..=31) => Some(&HALLOWEEN),
        (12, 1..=26) => Some(&CHRISTMAS),
        (12, 31) | (1, 1) => Some(&NEW_YEAR),
        _ => None,
    }
}

/// Easter Sunday (Gregorian), via the anonymous Gregorian computus
/// (Meeus/Jones/Butcher).
pub fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("computus yields a valid date")
}

/// US Thanksgiving: the 4th Thursday of November.
pub fn thanksgiving(year: i32) -> NaiveDate {
    NaiveDate::from_weekday_of_month_opt(year, 11, Weekday::Thu, 4)
        .expect("November always has 4 Thursdays")
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn holiday(y: i32, m: u32, d: u32) -> Option<&'static str> {
        holiday_for(date(y, m, d)).map(|t| t.name)
    }

    #[test]
    fn easter_known_dates() {
        assert_eq!(easter_sunday(2000), date(2000, 4, 23));
        assert_eq!(easter_sunday(2019), date(2019, 4, 21));
        assert_eq!(easter_sunday(2024), date(2024, 3, 31));
        assert_eq!(easter_sunday(2025), date(2025, 4, 20));
        assert_eq!(easter_sunday(2026), date(2026, 4, 5));
        assert_eq!(easter_sunday(2027), date(2027, 3, 28));
        assert_eq!(easter_sunday(2038), date(2038, 4, 25)); // latest possible
        assert_eq!(easter_sunday(2285), date(2285, 3, 22)); // earliest possible
    }

    #[test]
    fn thanksgiving_known_dates() {
        assert_eq!(thanksgiving(2024), date(2024, 11, 28));
        assert_eq!(thanksgiving(2025), date(2025, 11, 27));
        assert_eq!(thanksgiving(2026), date(2026, 11, 26));
    }

    #[test]
    fn carnaval_window() {
        // 2025: Carnaval Tuesday is Mar 4.
        assert_eq!(holiday(2025, 2, 28), None);
        assert_eq!(holiday(2025, 3, 1), Some("carnaval")); // Saturday
        assert_eq!(holiday(2025, 3, 4), Some("carnaval")); // Tuesday
        assert_eq!(holiday(2025, 3, 5), Some("carnaval")); // Ash Wednesday
        assert_eq!(holiday(2025, 3, 6), None);
    }

    #[test]
    fn carnaval_beats_valentines_when_they_collide() {
        // 2026: Carnaval Saturday is Feb 14.
        assert_eq!(holiday(2026, 2, 14), Some("carnaval"));
        assert_eq!(holiday(2025, 2, 14), Some("valentines"));
    }

    #[test]
    fn easter_window() {
        // 2026: Easter Apr 5 → Good Friday Apr 3, Easter Monday Apr 6.
        assert_eq!(holiday(2026, 4, 2), None);
        assert_eq!(holiday(2026, 4, 3), Some("easter"));
        assert_eq!(holiday(2026, 4, 6), Some("easter"));
        assert_eq!(holiday(2026, 4, 7), None);
    }

    #[test]
    fn fixed_date_windows() {
        assert_eq!(holiday(2026, 3, 17), Some("st-patricks"));
        assert_eq!(holiday(2026, 3, 18), None);
        assert_eq!(holiday(2026, 10, 23), None);
        assert_eq!(holiday(2026, 10, 24), Some("halloween"));
        assert_eq!(holiday(2026, 10, 31), Some("halloween"));
        assert_eq!(holiday(2026, 11, 1), None);
        assert_eq!(holiday(2026, 11, 30), None);
        assert_eq!(holiday(2026, 12, 1), Some("christmas"));
        assert_eq!(holiday(2026, 12, 26), Some("christmas"));
        assert_eq!(holiday(2026, 12, 27), None);
        assert_eq!(holiday(2026, 12, 31), Some("new-year"));
        assert_eq!(holiday(2027, 1, 1), Some("new-year"));
        assert_eq!(holiday(2027, 1, 2), None);
    }

    #[test]
    fn thanksgiving_is_one_day() {
        assert_eq!(holiday(2026, 11, 25), None);
        assert_eq!(holiday(2026, 11, 26), Some("thanksgiving"));
        assert_eq!(holiday(2026, 11, 27), None);
    }

    #[test]
    fn ordinary_day_has_no_holiday() {
        assert_eq!(holiday(2026, 11, 14), None);
        assert_eq!(holiday(2026, 7, 4), None);
    }

    #[test]
    fn every_food_glyph_fills_exactly_one_cell() {
        for theme in ALL {
            for f in theme.food {
                assert_eq!(f.emoji.width(), 2, "{} emoji {:?}", theme.name, f.emoji);
                assert_eq!(f.ascii.width(), 2, "{} ascii {:?}", theme.name, f.ascii);
            }
        }
    }

    #[test]
    fn themes_are_findable_and_well_formed() {
        for theme in ALL {
            assert_eq!(by_name(theme.name), Some(*theme));
            assert!(!theme.body.is_empty() && !theme.food.is_empty());
        }
        assert!(by_name("CHRISTMAS").is_some());
        assert!(by_name("nope").is_none());
    }
}
