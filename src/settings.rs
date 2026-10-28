//! The settings page's options. Each is a [`Choice`] its button cycles
//! through; the mod's storage keeps the chosen value under the choice's key,
//! so the page, saving and loading are the same code for all of them.

/// An option: its storage key, its label on the page and its values
/// (storage value, label), the first being the default.
pub struct Choice {
    pub key: &'static str,
    pub label: &'static str,
    pub values: &'static [(&'static str, &'static str)],
}

const ON: (&str, &str) = ("true", "ON");
const OFF: (&str, &str) = ("false", "OFF");

/// How far it looks, in blocks. The default is a creative player's reach,
/// a little over survival's 4.5; the raycast goes no further than 1024.
pub const RANGE: Choice = Choice {
    key: "range",
    label: "Range",
    values: &[
        ("5", "Reach (5 blocks)"),
        ("16", "16 blocks"),
        ("64", "64 blocks"),
        ("1024", "1024 blocks"),
    ],
};
/// Where the box sits, at the top of the screen.
pub const POSITION: Choice = Choice {
    key: "position",
    label: "Position",
    values: &[
        ("top", "Top"),
        ("top_left", "Top left"),
        ("top_right", "Top right"),
    ],
};
/// The box's background and frame, in percent; the text stays opaque.
pub const OPACITY: Choice = Choice {
    key: "opacity",
    label: "Background opacity",
    values: &[("100", "100%"), ("75", "75%"), ("50", "50%"), ("25", "25%")],
};
pub const ICON: Choice = Choice {
    key: "icon",
    label: "Item icon",
    values: &[ON, OFF],
};
pub const SOURCE: Choice = Choice {
    key: "source",
    label: "Where it comes from",
    values: &[ON, OFF],
};
pub const DETAILS: Choice = Choice {
    key: "details",
    label: "Position and distance",
    values: &[OFF, ON],
};

/// The page's options, in its order. A button's id is its index here.
pub const CHOICES: [&Choice; 6] = [&RANGE, &POSITION, &OPACITY, &ICON, &SOURCE, &DETAILS];

/// Where the box sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Top,
    TopLeft,
    TopRight,
}

/// The chosen value of each of [`CHOICES`], as an index in its values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings([usize; CHOICES.len()]);

impl Settings {
    /// From stored values, read by `get(key)`; a missing or unknown one is
    /// the default.
    pub fn load(get: impl Fn(&str) -> Option<String>) -> Self {
        Settings(CHOICES.map(|c| {
            get(c.key)
                .and_then(|v| c.values.iter().position(|(value, _)| *value == v.trim()))
                .unwrap_or(0)
        }))
    }

    /// Every (key, value) to store.
    pub fn stored(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        CHOICES
            .iter()
            .zip(self.0)
            .map(|(c, i)| (c.key, c.values[i].0))
    }

    /// The page's buttons: (id, label).
    pub fn buttons(&self) -> impl Iterator<Item = (u32, String)> + '_ {
        CHOICES
            .iter()
            .zip(self.0)
            .enumerate()
            .map(|(id, (c, i))| (id as u32, format!("{}: {}", c.label, c.values[i].1)))
    }

    /// Moves the option behind button `id` to its next value; false for an
    /// id that is no button.
    pub fn cycle(&mut self, id: u32) -> bool {
        let Some(c) = CHOICES.get(id as usize) else {
            return false;
        };
        let i = &mut self.0[id as usize];
        *i = (*i + 1) % c.values.len();
        true
    }

    fn value(&self, choice: &Choice) -> &'static str {
        // By key: a `const` has no single address to compare.
        let index = CHOICES
            .iter()
            .position(|c| c.key == choice.key)
            .expect("every choice is on the page");
        choice.values[self.0[index]].0
    }

    fn on(&self, choice: &Choice) -> bool {
        self.value(choice) == ON.0
    }

    pub fn range(&self) -> f32 {
        self.value(&RANGE).parse().expect("ranges are numbers")
    }

    pub fn position(&self) -> Position {
        match self.value(&POSITION) {
            "top_left" => Position::TopLeft,
            "top_right" => Position::TopRight,
            _ => Position::Top,
        }
    }

    /// 0.0 to 1.0.
    pub fn opacity(&self) -> f32 {
        self.value(&OPACITY)
            .parse::<f32>()
            .expect("opacities are numbers")
            / 100.0
    }

    pub fn icon(&self) -> bool {
        self.on(&ICON)
    }

    pub fn source(&self) -> bool {
        self.on(&SOURCE)
    }

    pub fn details(&self) -> bool {
        self.on(&DETAILS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn load(pairs: &[(&str, &str)]) -> Settings {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|&(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Settings::load(|key| map.get(key).cloned())
    }

    #[test]
    fn the_defaults_are_a_vanilla_tooltip_within_reach() {
        let s = Settings::default();
        assert_eq!(s.range(), 5.0);
        assert_eq!(s.position(), Position::Top);
        assert_eq!(s.opacity(), 1.0);
        assert!(s.icon() && s.source() && !s.details());
        assert_eq!(load(&[]), s);
    }

    #[test]
    fn stored_values_are_read_back_and_unknown_ones_are_the_default() {
        let s = load(&[
            ("range", "1024\n"),
            ("position", "top_right"),
            ("opacity", "50"),
            ("icon", "false"),
            ("details", "true"),
            ("source", "maybe"),
        ]);
        assert_eq!(s.range(), 1024.0);
        assert_eq!(s.position(), Position::TopRight);
        assert_eq!(s.opacity(), 0.5);
        assert!(!s.icon() && s.details() && s.source());
        let again = load(&s.stored().collect::<Vec<_>>());
        assert_eq!(again, s);
    }

    #[test]
    fn a_button_cycles_its_option_and_wraps_around() {
        let mut s = Settings::default();
        let range = CHOICES.iter().position(|c| c.key == "range").unwrap() as u32;
        for blocks in [16.0, 64.0, 1024.0, 5.0] {
            assert!(s.cycle(range));
            assert_eq!(s.range(), blocks);
        }
        assert!(!s.cycle(CHOICES.len() as u32));
    }

    #[test]
    fn the_page_labels_every_option_with_its_value() {
        let labels: Vec<String> = Settings::default().buttons().map(|(_, l)| l).collect();
        assert_eq!(
            labels,
            [
                "Range: Reach (5 blocks)",
                "Position: Top",
                "Background opacity: 100%",
                "Item icon: ON",
                "Where it comes from: ON",
                "Position and distance: OFF",
            ]
        );
    }

    #[test]
    fn keys_and_values_are_unique() {
        for (i, c) in CHOICES.iter().enumerate() {
            assert!(CHOICES[..i].iter().all(|o| o.key != c.key), "{}", c.key);
            for (j, (v, _)) in c.values.iter().enumerate() {
                assert!(c.values[..j].iter().all(|(o, _)| o != v), "{}: {v}", c.key);
            }
        }
    }
}
