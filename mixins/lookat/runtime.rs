// LookAt: what is under the crosshair, at the top of the screen, in a box
// drawn as vanilla draws a tooltip.
//
// This file is both the standalone, unit-tested Cargo library and the
// injected `crate::game::lookat` module. It has no Horizon dependency: the
// game hands it what the crosshair hits (a `Target`) and item names (an
// `ItemNames`), and draws the `Draw`s it returns; `native.rs` is that adapter.
//
// - Names: a block is named after its item in the player's language (no
//   table names blocks), or after the item vanilla's pick block gives.
// - Settings: one `Choice` per button of the settings page, each kept in its
//   own file under the mod's folder, as the WebAssembly LookAt kept them, so
//   a player's settings carry over.
// - Layout: the box, its frame drawn with fills as 0.1.0 drew it, the item
//   icon and the lines.

use std::collections::HashMap;

/// The mod's id: its row in Options > Mods and its settings folder
/// (`horizon-mods/.horizon-mod-data/lookat/`), the WebAssembly mod's too.
pub const ID: &str = "lookat";
pub const VERSION: &str = "0.2.0";
pub const DESCRIPTION: &str = "What is under your crosshair at the top of the screen, as a vanilla tooltip: \
    a block's icon, name, harvest tool and where it comes from, a mob's name and health.";

// ---- Look -------------------------------------------------------------------

/// Vanilla's tooltip (TooltipRenderUtil): a near-black purple fill and a
/// frame fading from violet at the top to a darker violet at the bottom,
/// 0xAARRGGBB.
pub const BACKGROUND: u32 = 0xF010_0010;
pub const FRAME_TOP: u32 = 0x5050_00FF;
pub const FRAME_BOTTOM: u32 = 0x5028_007F;
/// Text colours (0xRRGGBB), as vanilla's chat formatting codes: the name
/// white (`f`), details gray (`7`), the source blue (`9`), health red (`c`),
/// a tool that harvests green (`a`) and one that doesn't red (`c`).
pub const NAME: u32 = 0xFF_FFFF;
pub const DETAILS: u32 = 0xAA_AAAA;
pub const SOURCE: u32 = 0x55_55FF;
pub const HEALTH: u32 = 0xFF_5555;
pub const HARVESTS: u32 = 0x55_FF55;
pub const NO_HARVEST: u32 = 0xFF_5555;
/// GUI pixels: the content's distance from the screen's top edge, the
/// frame's thickness around the content, the item icon's size and the gap
/// between it and the text.
pub const MARGIN: f32 = 8.0;
pub const FRAME: f32 = 4.0;
pub const ICON: f32 = 16.0;
pub const ICON_GAP: f32 = 4.0;
/// The space the game's corner boxes keep from the edge and from each
/// other (`extras_hud`); LookAt keeps this much more, so that in an empty
/// corner its content is `MARGIN` from the edges, as in 0.1.0.
pub const CORNER_GAP: f32 = MARGIN - FRAME - 2.0;
/// Heart sprites (`hud/heart/*`, 9 pixels), 8 pixels apart and 10 a row, as
/// the player's own hearts.
pub const HEART_CONTAINER: &str = "gui/sprites/hud/heart/container";
pub const HEART_FULL: &str = "gui/sprites/hud/heart/full";
pub const HEART_HALF: &str = "gui/sprites/hud/heart/half";
pub const HEART: f32 = 9.0;
pub const HEART_STEP: f32 = 8.0;
pub const HEARTS_PER_ROW: usize = 10;
pub const HEART_ROW: f32 = 10.0;
/// The most health drawn as hearts (two rows); more is written as numbers.
pub const MAX_HEARTS_HEALTH: f32 = 40.0;

// ---- Names ------------------------------------------------------------------

/// Item names in the player's language: `None` for an item the game
/// doesn't have.
pub trait ItemNames {
    fn item_name(&self, item: &str) -> Option<String>;
}

impl ItemNames for HashMap<String, String> {
    fn item_name(&self, item: &str) -> Option<String> {
        self.get(item).cloned()
    }
}

/// Blocks whose pick-block item is not their own id with `wall_` dropped.
const PICK_ITEMS: &[(&str, &str)] = &[
    ("attached_melon_stem", "melon_seeds"),
    ("attached_pumpkin_stem", "pumpkin_seeds"),
    ("bamboo_sapling", "bamboo"),
    ("beetroots", "beetroot_seeds"),
    ("big_dripleaf_stem", "big_dripleaf"),
    ("carrots", "carrot"),
    ("cave_vines", "glow_berries"),
    ("cave_vines_plant", "glow_berries"),
    ("cocoa", "cocoa_beans"),
    ("kelp_plant", "kelp"),
    ("lava_cauldron", "cauldron"),
    ("melon_stem", "melon_seeds"),
    ("moving_piston", "piston"),
    ("piston_head", "piston"),
    ("pitcher_crop", "pitcher_pod"),
    ("potatoes", "potato"),
    ("potted_azalea_bush", "azalea"),
    ("potted_flowering_azalea_bush", "flowering_azalea"),
    ("powder_snow", "powder_snow_bucket"),
    ("powder_snow_cauldron", "cauldron"),
    ("pumpkin_stem", "pumpkin_seeds"),
    ("redstone_wire", "redstone"),
    ("sweet_berry_bush", "sweet_berries"),
    ("tall_seagrass", "seagrass"),
    ("torchflower_crop", "torchflower_seeds"),
    ("tripwire", "string"),
    ("twisting_vines_plant", "twisting_vines"),
    ("water_cauldron", "cauldron"),
    ("weeping_vines_plant", "weeping_vines"),
];

/// A block as the box shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    /// The item drawn as its icon; `None` when it has none.
    pub item: Option<String>,
    /// Its name in the player's language, or its id spelled out.
    pub name: String,
    /// The namespace it comes from, spelled out: `Minecraft`.
    pub source: String,
}

/// `block` as the game names it: `minecraft:oak_log`, with or without a
/// `[state]`. A block without an item of its own (a wall torch, a crop, a
/// potted plant) takes the item vanilla's pick block gives for it; one with
/// neither (fire, a portal) gets its id spelled out in English.
pub fn describe(block: &str, items: &impl ItemNames) -> Described {
    let id = block.split_once('[').map_or(block, |(id, _)| id);
    let (namespace, path) = split_id(id);
    let named = |path: &str| {
        let item = format!("{namespace}:{path}");
        items.item_name(&item).map(|name| (item, name))
    };
    let (item, name) = match named(path).or_else(|| named(&pick_item(path))) {
        Some((item, name)) => (Some(item), name),
        None => (None, spell_out(path)),
    };
    Described {
        item,
        name,
        source: spell_out(namespace),
    }
}

/// `ns:path` as (`ns`, `path`); a bare id is vanilla's, as the game reads it.
pub fn split_id(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

/// The item vanilla's pick block gives for a block that has none of its own.
fn pick_item(path: &str) -> String {
    if let Some(&(_, item)) = PICK_ITEMS.iter().find(|(block, _)| *block == path) {
        return item.to_owned();
    }
    if let Some(plant) = path.strip_prefix("potted_") {
        return plant.to_owned();
    }
    // Wall-mounted torches, signs, banners, heads and coral fans.
    if let Some(rest) = path.strip_prefix("wall_") {
        return rest.to_owned();
    }
    path.replacen("_wall_", "_", 1)
}

/// `nether_portal` as `Nether Portal`.
pub fn spell_out(id: &str) -> String {
    id.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The mob's spawn egg, its icon (`Mob.getPickResult`).
pub fn spawn_egg(kind: &str) -> String {
    format!("{kind}_spawn_egg")
}

/// The least tool that harvests a block that needs one, from its block
/// tags (`in_tag("minecraft:mineable/pickaxe")`...): `minecraft:iron_pickaxe`.
/// `None` when no tool's `mineable` tag holds it (cobwebs and the like).
pub fn harvest_tool(in_tag: impl Fn(&str) -> bool) -> Option<String> {
    let tool = ["pickaxe", "axe", "shovel", "hoe"]
        .into_iter()
        .find(|tool| in_tag(&format!("minecraft:mineable/{tool}")))?;
    let tier = ["diamond", "iron", "stone"]
        .into_iter()
        .find(|tier| in_tag(&format!("minecraft:needs_{tier}_tool")))
        .unwrap_or("wooden");
    Some(format!("minecraft:{tier}_{tool}"))
}

// ---- Settings ---------------------------------------------------------------

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
/// a little over survival's 4.5; it goes no further than 1024, across the
/// distant terrain.
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
/// The whole box, text included, in percent of the GUI scale.
pub const SIZE: Choice = Choice {
    key: "size",
    label: "Size",
    values: &[
        ("100", "100%"),
        ("125", "125%"),
        ("150", "150%"),
        ("200", "200%"),
    ],
};
/// The box's background and frame, in percent; the text stays opaque.
pub const OPACITY: Choice = Choice {
    key: "opacity",
    label: "Background opacity",
    values: &[("100", "100%"), ("75", "75%"), ("50", "50%"), ("25", "25%")],
};
pub const ICON_CHOICE: Choice = Choice {
    key: "icon",
    label: "Item icon",
    values: &[ON, OFF],
};
pub const SOURCE_CHOICE: Choice = Choice {
    key: "source",
    label: "Where it comes from",
    values: &[ON, OFF],
};
pub const DETAILS_CHOICE: Choice = Choice {
    key: "details",
    label: "Position and distance",
    values: &[OFF, ON],
};
/// Mobs, players, boats, minecarts, armor stands, item frames, paintings.
pub const ENTITIES: Choice = Choice {
    key: "entities",
    label: "Entities",
    values: &[ON, OFF],
};
/// A mob's health: hearts (numbers past two rows), numbers, or none.
pub const HEALTH_CHOICE: Choice = Choice {
    key: "health",
    label: "Health",
    values: &[("hearts", "Hearts"), ("numbers", "Numbers"), OFF],
};
/// The least tool a block needs to drop anything, and whether the held
/// item is one.
pub const HARVEST: Choice = Choice {
    key: "harvest",
    label: "Harvest tool",
    values: &[OFF, ON],
};

/// The page's options, in its order. A button's index is its index here.
pub const CHOICES: [&Choice; 10] = [
    &RANGE,
    &POSITION,
    &SIZE,
    &OPACITY,
    &ICON_CHOICE,
    &SOURCE_CHOICE,
    &DETAILS_CHOICE,
    &ENTITIES,
    &HEALTH_CHOICE,
    &HARVEST,
];

/// Where the box sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Top,
    TopLeft,
    TopRight,
}

/// How a mob's health shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Hearts,
    Numbers,
    Off,
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

    /// The page's button labels, in order.
    pub fn labels(&self) -> Vec<String> {
        CHOICES
            .iter()
            .zip(self.0)
            .map(|(c, i)| format!("{}: {}", c.label, c.values[i].1))
            .collect()
    }

    /// Moves the option behind button `button` to its next value; false for
    /// an index that is no button.
    pub fn cycle(&mut self, button: usize) -> bool {
        let Some(c) = CHOICES.get(button) else {
            return false;
        };
        let i = &mut self.0[button];
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

    fn percent(&self, choice: &Choice) -> f32 {
        self.value(choice)
            .parse::<f32>()
            .expect("percentages are numbers")
            / 100.0
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

    /// 1.0 is the GUI's own scale.
    pub fn size(&self) -> f32 {
        self.percent(&SIZE)
    }

    /// 0.0 to 1.0.
    pub fn opacity(&self) -> f32 {
        self.percent(&OPACITY)
    }

    pub fn icon(&self) -> bool {
        self.on(&ICON_CHOICE)
    }

    pub fn source(&self) -> bool {
        self.on(&SOURCE_CHOICE)
    }

    pub fn details(&self) -> bool {
        self.on(&DETAILS_CHOICE)
    }

    pub fn entities(&self) -> bool {
        self.on(&ENTITIES)
    }

    pub fn health(&self) -> Health {
        match self.value(&HEALTH_CHOICE) {
            "hearts" => Health::Hearts,
            "numbers" => Health::Numbers,
            _ => Health::Off,
        }
    }

    pub fn harvest(&self) -> bool {
        self.on(&HARVEST)
    }
}

// ---- What the crosshair hits ------------------------------------------------

/// The block or entity under the crosshair, as the game reads it.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Block(BlockHit),
    Entity(EntityHit),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockHit {
    /// `minecraft:oak_log`.
    pub block: String,
    pub pos: [i32; 3],
    /// From the player's eyes to where the view meets it.
    pub distance: f64,
    /// For a block that drops nothing without the right tool.
    pub harvest: Option<Harvest>,
}

/// The least tool a block needs, and whether the held item harvests it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Harvest {
    /// `minecraft:iron_pickaxe` ([`harvest_tool`]).
    pub tool: String,
    pub ok: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityHit {
    /// Its type, `minecraft:zombie`: where it comes from.
    pub kind: String,
    /// Its custom name, a player's name, or its type's name.
    pub name: String,
    /// The item drawn as its icon, when the game has it: a mob's spawn
    /// egg, a boat's or a minecart's item, an item frame's framed item.
    pub icon: Option<String>,
    /// A gray line under the name: what an item frame holds.
    pub note: Option<String>,
    /// (health, max health) of a living one.
    pub health: Option<(f32, f32)>,
    pub pos: [i32; 3],
    pub distance: f64,
}

// ---- The box ----------------------------------------------------------------

/// A line of the box.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    /// Text in an opaque 0xRRGGBB colour.
    Text(String, u32),
    /// Health as hearts: (health, max health), max at most
    /// [`MAX_HEARTS_HEALTH`].
    Hearts(f32, f32),
}

/// What the box shows.
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    pub item: Option<String>,
    pub lines: Vec<Line>,
}

/// What the box shows for `target`, as the settings ask.
pub fn view(target: &Target, s: &Settings, items: &impl ItemNames) -> View {
    let (icon, name, source, pos, distance) = match target {
        Target::Block(b) => {
            let d = describe(&b.block, items);
            (d.item, d.name, d.source, b.pos, b.distance)
        }
        Target::Entity(e) => (
            e.icon.clone(),
            e.name.clone(),
            spell_out(split_id(&e.kind).0),
            e.pos,
            e.distance,
        ),
    };
    let mut lines = vec![Line::Text(name, NAME)];
    match target {
        Target::Block(b) => {
            if let Some(h) = b.harvest.as_ref().filter(|_| s.harvest()) {
                let tool = items
                    .item_name(&h.tool)
                    .unwrap_or_else(|| spell_out(split_id(&h.tool).1));
                lines.push(if h.ok {
                    Line::Text(format!("\u{2714} {tool}"), HARVESTS)
                } else {
                    Line::Text(format!("\u{2718} {tool}"), NO_HARVEST)
                });
            }
        }
        Target::Entity(e) => {
            if let Some(line) = e
                .health
                .and_then(|(h, max)| health_line(h, max, s.health()))
            {
                lines.push(line);
            }
            if let Some(note) = &e.note {
                lines.push(Line::Text(note.clone(), DETAILS));
            }
        }
    }
    if s.details() {
        lines.push(Line::Text(
            format!("{} {} {}  \u{b7}  {distance:.1} m", pos[0], pos[1], pos[2]),
            DETAILS,
        ));
    }
    if s.source() {
        lines.push(Line::Text(source, SOURCE));
    }
    View {
        item: icon.filter(|_| s.icon()),
        lines,
    }
}

/// A mob's health line: hearts while they fit in two rows, else numbers
/// (`❤ 15.5 / 20`); none when off or the mob has no health to speak of.
pub fn health_line(health: f32, max: f32, how: Health) -> Option<Line> {
    if how == Health::Off || !max.is_finite() || max <= 0.0 || !health.is_finite() {
        return None;
    }
    let health = health.clamp(0.0, max);
    if how == Health::Hearts && max <= MAX_HEARTS_HEALTH {
        return Some(Line::Hearts(health, max));
    }
    let number = |v: f32| {
        if v.fract() == 0.0 {
            format!("{v:.0}")
        } else {
            format!("{v:.1}")
        }
    };
    Some(Line::Text(
        format!("\u{2764} {} / {}", number(health), number(max)),
        HEALTH,
    ))
}

/// Hearts for `max` health, and the rows they take.
fn heart_count(max: f32) -> (usize, usize) {
    let hearts = (max / 2.0).ceil().max(1.0) as usize;
    (hearts, hearts.div_ceil(HEARTS_PER_ROW))
}

impl Line {
    fn width(&self, text_width: &dyn Fn(&str) -> f32) -> f32 {
        match self {
            Line::Text(text, _) => text_width(text),
            Line::Hearts(_, max) => {
                let in_row = heart_count(*max).0.min(HEARTS_PER_ROW);
                (in_row - 1) as f32 * HEART_STEP + HEART
            }
        }
    }

    /// Its own height: a line of text is 8 pixels.
    fn height(&self) -> f32 {
        match self {
            Line::Text(..) => 8.0,
            Line::Hearts(_, max) => (heart_count(*max).1 - 1) as f32 * HEART_ROW + HEART,
        }
    }
}

/// The space under line `i`: 4 under the first, 2 under the others, as a
/// tooltip spaces its title (lines of text 12 and 10 pixels apart).
fn gap(i: usize) -> f32 {
    if i == 0 { 4.0 } else { 2.0 }
}

fn lines_height(lines: &[Line]) -> f32 {
    let gaps: f32 = (0..lines.len().saturating_sub(1)).map(gap).sum();
    lines.iter().map(Line::height).sum::<f32>() + gaps
}

/// The content's size (inside the frame), in GUI pixels.
pub fn content_size(v: &View, text_width: &dyn Fn(&str) -> f32) -> (f32, f32) {
    let lines_width = v
        .lines
        .iter()
        .map(|l| l.width(text_width))
        .fold(0.0, f32::max);
    let text_x = if v.item.is_some() {
        ICON + ICON_GAP
    } else {
        0.0
    };
    (text_x + lines_width, lines_height(&v.lines).max(ICON))
}

/// The left of a box `outer_w` wide (frame included) centred on a screen
/// `screen_w` wide.
pub fn top_centre(screen_w: f32, outer_w: f32) -> f32 {
    ((screen_w - outer_w) / 2.0).round()
}

/// What to draw, in order.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    /// [x, y, w, h], 0xAARRGGBB.
    Fill { rect: [f32; 4], argb: u32 },
    /// A 16-pixel item icon.
    Item { item: String, at: [f32; 2] },
    /// Text with its shadow, opaque 0xRRGGBB.
    Text {
        text: String,
        at: [f32; 2],
        rgb: u32,
    },
    /// A GUI atlas sprite at its own size.
    Sprite { sprite: &'static str, at: [f32; 2] },
}

/// The box with its content's top left at `(x, y)`: the frame, the icon
/// centred on the left, the lines after it.
pub fn draws(
    v: &View,
    s: &Settings,
    (x, y): (f32, f32),
    text_width: &dyn Fn(&str) -> f32,
) -> Vec<Draw> {
    let (w, h) = content_size(v, text_width);
    let text_height = lines_height(&v.lines);
    let mut out = tooltip_frame(x, y, w, h, s.opacity());
    let text_x = match &v.item {
        Some(item) => {
            out.push(Draw::Item {
                item: item.clone(),
                at: [x, y + ((h - ICON) / 2.0).round()],
            });
            x + ICON + ICON_GAP
        }
        None => x,
    };
    let mut line_y = y + ((h - text_height) / 2.0).round();
    for (i, line) in v.lines.iter().enumerate() {
        match line {
            Line::Text(text, rgb) => out.push(Draw::Text {
                text: text.clone(),
                at: [text_x, line_y],
                rgb: *rgb,
            }),
            Line::Hearts(health, max) => hearts(&mut out, *health, *max, [text_x, line_y]),
        }
        line_y += line.height() + gap(i);
    }
    out
}

/// The hearts of `max` health, `health` of them filled (rounded up, as the
/// player's own), rows of ten from `at`.
fn hearts(out: &mut Vec<Draw>, health: f32, max: f32, at: [f32; 2]) {
    let filled = health.ceil() as usize;
    for i in 0..heart_count(max).0 {
        let at = [
            at[0] + (i % HEARTS_PER_ROW) as f32 * HEART_STEP,
            at[1] + (i / HEARTS_PER_ROW) as f32 * HEART_ROW,
        ];
        out.push(Draw::Sprite {
            sprite: HEART_CONTAINER,
            at,
        });
        let sprite = if filled >= 2 * i + 2 {
            HEART_FULL
        } else if filled == 2 * i + 1 {
            HEART_HALF
        } else {
            continue;
        };
        out.push(Draw::Sprite { sprite, at });
    }
}

/// Vanilla's tooltip background and frame around a `w` × `h` content box at
/// (`x`, `y`), their alpha scaled by `opacity`. Vanilla draws it with
/// nine-slice sprites; this draws the fills those sprites replaced, as
/// 0.1.0 did: the frame's sides fade one pixel row at a time.
pub fn tooltip_frame(x: f32, y: f32, w: f32, h: f32, opacity: f32) -> Vec<Draw> {
    let fill = |x: f32, y: f32, w: f32, h: f32, color: u32| Draw::Fill {
        rect: [x, y, w, h],
        argb: faded(color, opacity),
    };
    let mut out = vec![
        fill(x - 3.0, y - 4.0, w + 6.0, 1.0, BACKGROUND),
        fill(x - 3.0, y + h + 3.0, w + 6.0, 1.0, BACKGROUND),
        fill(x - 3.0, y - 3.0, w + 6.0, h + 6.0, BACKGROUND),
        fill(x - 4.0, y - 3.0, 1.0, h + 6.0, BACKGROUND),
        fill(x + w + 3.0, y - 3.0, 1.0, h + 6.0, BACKGROUND),
        fill(x - 3.0, y - 3.0, w + 6.0, 1.0, FRAME_TOP),
        fill(x - 3.0, y + h + 2.0, w + 6.0, 1.0, FRAME_BOTTOM),
    ];
    let rows = (h + 4.0) as usize;
    for row in 0..rows {
        let color = lerp_argb(FRAME_TOP, FRAME_BOTTOM, (row as f32 + 0.5) / rows as f32);
        let ry = y - 2.0 + row as f32;
        out.push(fill(x - 3.0, ry, 1.0, 1.0, color));
        out.push(fill(x + w + 2.0, ry, 1.0, 1.0, color));
    }
    out
}

/// `a` to `b`, channel by channel, at `t` in 0..=1.
fn lerp_argb(a: u32, b: u32, t: f32) -> u32 {
    (0..4).fold(0, |out, i| {
        let shift = i * 8;
        let (ca, cb) = ((a >> shift) & 0xFF, (b >> shift) & 0xFF);
        let c = (ca as f32 + (cb as f32 - ca as f32) * t).round() as u32;
        out | (c.min(0xFF) << shift)
    })
}

/// `argb` with its alpha scaled by `opacity` (0 to 1).
fn faded(argb: u32, opacity: f32) -> u32 {
    let alpha = ((argb >> 24) as f32 * opacity.clamp(0.0, 1.0)).round() as u32;
    (alpha << 24) | (argb & 0x00FF_FFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Names --

    fn names() -> HashMap<String, String> {
        [
            ("minecraft:stone", "Камень"),
            ("minecraft:torch", "Факел"),
            ("minecraft:soul_torch", "Факел душ"),
            ("minecraft:oak_sign", "Дубовая табличка"),
            ("minecraft:oak_hanging_sign", "Дубовая подвесная табличка"),
            ("minecraft:creeper_head", "Голова крипера"),
            ("minecraft:poppy", "Мак"),
            ("minecraft:azalea", "Азалия"),
            ("minecraft:carrot", "Морковь"),
            ("minecraft:redstone", "Редстоун"),
            ("minecraft:iron_pickaxe", "Железная кирка"),
            ("minecraft:zombie_spawn_egg", "Яйцо призыва зомби"),
        ]
        .into_iter()
        .map(|(id, name)| (id.to_owned(), name.to_owned()))
        .collect()
    }

    fn named(block: &str) -> (Option<String>, String) {
        let d = describe(block, &names());
        (d.item, d.name)
    }

    fn some(item: &str, name: &str) -> (Option<String>, String) {
        (Some(item.to_owned()), name.to_owned())
    }

    #[test]
    fn a_block_with_an_item_takes_its_name() {
        assert_eq!(named("minecraft:stone"), some("minecraft:stone", "Камень"));
    }

    #[test]
    fn a_block_state_is_ignored() {
        assert_eq!(
            named("minecraft:stone[snowy=false]"),
            some("minecraft:stone", "Камень")
        );
    }

    #[test]
    fn wall_variants_take_the_standing_item() {
        assert_eq!(
            named("minecraft:wall_torch"),
            some("minecraft:torch", "Факел")
        );
        assert_eq!(
            named("minecraft:soul_wall_torch"),
            some("minecraft:soul_torch", "Факел душ")
        );
        assert_eq!(
            named("minecraft:oak_wall_sign"),
            some("minecraft:oak_sign", "Дубовая табличка")
        );
        assert_eq!(
            named("minecraft:oak_wall_hanging_sign"),
            some("minecraft:oak_hanging_sign", "Дубовая подвесная табличка")
        );
        assert_eq!(
            named("minecraft:creeper_wall_head"),
            some("minecraft:creeper_head", "Голова крипера")
        );
    }

    #[test]
    fn a_potted_plant_takes_the_plant() {
        assert_eq!(
            named("minecraft:potted_poppy"),
            some("minecraft:poppy", "Мак")
        );
        assert_eq!(
            named("minecraft:potted_azalea_bush"),
            some("minecraft:azalea", "Азалия")
        );
    }

    #[test]
    fn crops_and_wires_take_their_pick_block_item() {
        assert_eq!(
            named("minecraft:carrots"),
            some("minecraft:carrot", "Морковь")
        );
        assert_eq!(
            named("minecraft:redstone_wire"),
            some("minecraft:redstone", "Редстоун")
        );
    }

    #[test]
    fn a_block_without_any_item_is_spelled_out_without_an_icon() {
        assert_eq!(
            named("minecraft:nether_portal"),
            (None, "Nether Portal".into())
        );
        assert_eq!(named("minecraft:fire"), (None, "Fire".into()));
    }

    #[test]
    fn the_source_is_the_namespace_spelled_out() {
        assert_eq!(describe("minecraft:stone", &names()).source, "Minecraft");
        assert_eq!(describe("cool_mod:ruby_ore", &names()).source, "Cool Mod");
        assert_eq!(describe("stone", &names()).source, "Minecraft");
    }

    #[test]
    fn an_empty_table_still_names_every_block() {
        let d = describe("minecraft:oak_log", &HashMap::new());
        assert_eq!((d.item, d.name.as_str()), (None, "Oak Log"));
    }

    #[test]
    fn the_harvest_tool_is_the_least_tier_of_the_mineable_tool() {
        let tags = |tags: &'static [&'static str]| move |t: &str| tags.contains(&t);
        assert_eq!(
            harvest_tool(tags(&["minecraft:mineable/pickaxe"])).as_deref(),
            Some("minecraft:wooden_pickaxe")
        );
        assert_eq!(
            harvest_tool(tags(&[
                "minecraft:mineable/pickaxe",
                "minecraft:needs_iron_tool"
            ]))
            .as_deref(),
            Some("minecraft:iron_pickaxe")
        );
        assert_eq!(
            harvest_tool(tags(&[
                "minecraft:mineable/shovel",
                "minecraft:needs_diamond_tool"
            ]))
            .as_deref(),
            Some("minecraft:diamond_shovel")
        );
        assert_eq!(harvest_tool(tags(&["minecraft:needs_stone_tool"])), None);
    }

    // -- Settings --

    fn load(pairs: &[(&str, &str)]) -> Settings {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|&(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Settings::load(|key| map.get(key).cloned())
    }

    fn button(key: &str) -> usize {
        CHOICES.iter().position(|c| c.key == key).unwrap()
    }

    #[test]
    fn the_defaults_are_a_vanilla_tooltip_within_reach() {
        let s = Settings::default();
        assert_eq!(s.range(), 5.0);
        assert_eq!(s.position(), Position::Top);
        assert_eq!(s.size(), 1.0);
        assert_eq!(s.opacity(), 1.0);
        assert!(s.icon() && s.source() && !s.details());
        assert!(s.entities() && !s.harvest());
        assert_eq!(s.health(), Health::Hearts);
        assert_eq!(load(&[]), s);
    }

    #[test]
    fn settings_kept_by_0_1_0_are_read_back() {
        // The WebAssembly LookAt's storage: one file per key, same keys.
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
        // The new options are at their defaults.
        assert!(s.entities() && !s.harvest() && s.size() == 1.0);
        let again = load(&s.stored().collect::<Vec<_>>());
        assert_eq!(again, s);
    }

    #[test]
    fn new_settings_are_kept_and_read_back() {
        let s = load(&[
            ("size", "150"),
            ("entities", "false"),
            ("health", "numbers"),
            ("harvest", "true"),
        ]);
        assert_eq!(s.size(), 1.5);
        assert!(!s.entities() && s.harvest());
        assert_eq!(s.health(), Health::Numbers);
        assert_eq!(load(&s.stored().collect::<Vec<_>>()), s);
    }

    #[test]
    fn a_button_cycles_its_option_and_wraps_around() {
        let mut s = Settings::default();
        for blocks in [16.0, 64.0, 1024.0, 5.0] {
            assert!(s.cycle(button("range")));
            assert_eq!(s.range(), blocks);
        }
        for health in [Health::Numbers, Health::Off, Health::Hearts] {
            assert!(s.cycle(button("health")));
            assert_eq!(s.health(), health);
        }
        assert!(!s.cycle(CHOICES.len()));
    }

    #[test]
    fn the_page_labels_every_option_with_its_value() {
        assert_eq!(
            Settings::default().labels(),
            [
                "Range: Reach (5 blocks)",
                "Position: Top",
                "Size: 100%",
                "Background opacity: 100%",
                "Item icon: ON",
                "Where it comes from: ON",
                "Position and distance: OFF",
                "Entities: ON",
                "Health: Hearts",
                "Harvest tool: OFF",
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

    // -- The box --

    /// Six pixels a character, as most of the font's letters.
    fn width(text: &str) -> f32 {
        6.0 * text.chars().count() as f32
    }

    fn stone(harvest: Option<Harvest>) -> Target {
        Target::Block(BlockHit {
            block: "minecraft:stone".into(),
            pos: [10, 64, -3],
            distance: 2.345,
            harvest,
        })
    }

    fn zombie(health: f32, max: f32) -> Target {
        Target::Entity(EntityHit {
            kind: "minecraft:zombie".into(),
            name: "Зомби".into(),
            icon: Some(spawn_egg("minecraft:zombie")),
            note: None,
            health: Some((health, max)),
            pos: [1, 2, 3],
            distance: 1.5,
        })
    }

    fn text(t: &str, rgb: u32) -> Line {
        Line::Text(t.into(), rgb)
    }

    #[test]
    fn a_block_shows_as_in_0_1_0() {
        let s = Settings::default();
        let pickaxe = Some(Harvest {
            tool: "minecraft:iron_pickaxe".into(),
            ok: false,
        });
        let v = view(&stone(pickaxe), &s, &names());
        assert_eq!(
            v,
            View {
                item: Some("minecraft:stone".into()),
                lines: vec![text("Камень", NAME), text("Minecraft", SOURCE)],
            }
        );
        let mut s = Settings::default();
        s.cycle(button("details"));
        s.cycle(button("icon"));
        let v = view(&stone(None), &s, &names());
        assert_eq!(v.item, None);
        assert_eq!(v.lines[1], text("10 64 -3  \u{b7}  2.3 m", DETAILS));
    }

    #[test]
    fn the_harvest_tool_shows_when_asked() {
        let mut s = Settings::default();
        s.cycle(button("harvest"));
        let tool = |ok| {
            Some(Harvest {
                tool: "minecraft:iron_pickaxe".into(),
                ok,
            })
        };
        let v = view(&stone(tool(false)), &s, &names());
        assert_eq!(v.lines[1], text("\u{2718} Железная кирка", NO_HARVEST));
        let v = view(&stone(tool(true)), &s, &names());
        assert_eq!(v.lines[1], text("\u{2714} Железная кирка", HARVESTS));
        // Nothing for a block any hand harvests.
        assert_eq!(view(&stone(None), &s, &names()).lines.len(), 2);
    }

    #[test]
    fn an_entity_shows_its_name_health_and_source() {
        let s = Settings::default();
        let v = view(&zombie(15.0, 20.0), &s, &names());
        assert_eq!(v.item.as_deref(), Some("minecraft:zombie_spawn_egg"));
        assert_eq!(
            v.lines,
            [
                text("Зомби", NAME),
                Line::Hearts(15.0, 20.0),
                text("Minecraft", SOURCE)
            ]
        );
    }

    #[test]
    fn health_turns_to_numbers_past_two_rows_of_hearts() {
        assert_eq!(
            health_line(20.0, 20.0, Health::Hearts),
            Some(Line::Hearts(20.0, 20.0))
        );
        assert_eq!(
            health_line(150.5, 200.0, Health::Hearts),
            Some(text("\u{2764} 150.5 / 200", HEALTH))
        );
        assert_eq!(
            health_line(25.0, 20.0, Health::Numbers),
            Some(text("\u{2764} 20 / 20", HEALTH))
        );
        assert_eq!(health_line(5.0, 20.0, Health::Off), None);
        assert_eq!(health_line(5.0, 0.0, Health::Hearts), None);
    }

    /// 0.1.0's `shapes` as fills, items and texts: the same box, pixel for pixel.
    fn shapes_0_1_0(lines: &[(&str, u32)], icon: bool, screen: f32) -> Vec<Draw> {
        let text_width = lines.iter().map(|(l, _)| width(l)).fold(0.0, f32::max);
        let text_height = match lines.len() {
            1 => 8.0,
            n => 10.0 * n as f32,
        };
        let text_x = if icon { ICON + ICON_GAP } else { 0.0 };
        let (w, h) = (text_x + text_width, text_height.max(ICON));
        let x = ((screen - w) / 2.0).round();
        let y = MARGIN;
        let mut out = tooltip_frame(x, y, w, h, 1.0);
        if icon {
            out.push(Draw::Item {
                item: "minecraft:stone".into(),
                at: [x, y + ((h - ICON) / 2.0).round()],
            });
        }
        let mut line_y = y + ((h - text_height) / 2.0).round();
        for (i, (line, color)) in lines.iter().enumerate() {
            out.push(Draw::Text {
                text: (*line).into(),
                at: [x + text_x, line_y],
                rgb: *color,
            });
            line_y += if i == 0 { 12.0 } else { 10.0 };
        }
        out
    }

    /// The box at the top centre of a `screen`-wide GUI, as `native.rs`
    /// places it.
    fn at_top(v: &View, s: &Settings, screen: f32) -> Vec<Draw> {
        let (w, _) = content_size(v, &width);
        let left = top_centre(screen, w + 2.0 * FRAME);
        draws(v, s, (left + FRAME, MARGIN), &width)
    }

    #[test]
    fn the_default_box_is_0_1_0s_pixel_for_pixel() {
        let s = Settings::default();
        let v = view(&stone(None), &s, &names());
        assert_eq!(
            at_top(&v, &s, 427.0),
            shapes_0_1_0(&[("Камень", NAME), ("Minecraft", SOURCE)], true, 427.0)
        );
        let mut s = Settings::default();
        s.cycle(button("details"));
        let v = view(&stone(None), &s, &names());
        let lines = [
            ("Камень", NAME),
            ("10 64 -3  \u{b7}  2.3 m", DETAILS),
            ("Minecraft", SOURCE),
        ];
        assert_eq!(at_top(&v, &s, 640.0), shapes_0_1_0(&lines, true, 640.0));
        s.cycle(button("source"));
        s.cycle(button("icon"));
        let v = view(&stone(None), &s, &names());
        assert_eq!(
            at_top(&v, &s, 300.0),
            shapes_0_1_0(&lines[..2], false, 300.0)
        );
    }

    #[test]
    fn hearts_take_rows_of_ten() {
        let s = Settings::default();
        let v = View {
            item: None,
            lines: vec![text("Horse", NAME), Line::Hearts(23.0, 30.0)],
        };
        // 10 hearts wide; 8 + 4 + (10 + 9) high.
        assert_eq!(content_size(&v, &width), (9.0 * 8.0 + 9.0, 31.0));
        let sprites: Vec<(&str, [f32; 2])> = draws(&v, &s, (0.0, 0.0), &width)
            .into_iter()
            .filter_map(|d| match d {
                Draw::Sprite { sprite, at } => Some((sprite, at)),
                _ => None,
            })
            .collect();
        let containers = sprites.iter().filter(|(s, _)| *s == HEART_CONTAINER);
        assert_eq!(containers.count(), 15);
        let full = sprites.iter().filter(|(s, _)| *s == HEART_FULL).count();
        let half: Vec<_> = sprites.iter().filter(|(s, _)| *s == HEART_HALF).collect();
        assert_eq!(full, 11);
        // The 12th heart is half: the second of the second row.
        assert_eq!(half, [&(HEART_HALF, [8.0, 22.0])]);
        // The first heart sits under the name, 12 pixels down.
        assert_eq!(sprites[0], (HEART_CONTAINER, [0.0, 12.0]));
    }

    #[test]
    fn the_frame_fades_from_top_to_bottom() {
        assert_eq!(lerp_argb(FRAME_TOP, FRAME_BOTTOM, 0.0), FRAME_TOP);
        assert_eq!(lerp_argb(FRAME_TOP, FRAME_BOTTOM, 1.0), FRAME_BOTTOM);
        assert_eq!(lerp_argb(0x0000_0000, 0xFF00_FF00, 0.5), 0x8000_8000);
    }

    #[test]
    fn opacity_scales_only_the_alpha() {
        assert_eq!(faded(BACKGROUND, 1.0), BACKGROUND);
        assert_eq!(faded(BACKGROUND, 0.5), 0x7810_0010);
        assert_eq!(faded(BACKGROUND, 0.0), 0x0010_0010);
    }

    fn fills(draws: &[Draw]) -> Vec<([f32; 4], u32)> {
        draws
            .iter()
            .map(|d| match d {
                Draw::Fill { rect, argb } => (*rect, *argb),
                _ => unreachable!("the frame is fills only"),
            })
            .collect()
    }

    #[test]
    fn the_frame_surrounds_the_content() {
        let rects: Vec<[f32; 4]> = fills(&tooltip_frame(100.0, 8.0, 50.0, 16.0, 1.0))
            .into_iter()
            .map(|(r, _)| r)
            .collect();
        let left = rects.iter().map(|r| r[0]).fold(f32::MAX, f32::min);
        let right = rects.iter().map(|r| r[0] + r[2]).fold(f32::MIN, f32::max);
        let top = rects.iter().map(|r| r[1]).fold(f32::MAX, f32::min);
        let bottom = rects.iter().map(|r| r[1] + r[3]).fold(f32::MIN, f32::max);
        assert_eq!((left, right, top, bottom), (96.0, 154.0, 4.0, 28.0));
        // `FRAME` is how far it reaches past the content.
        assert_eq!((100.0 - left, top - 4.0), (FRAME, 0.0));
    }

    #[test]
    fn the_frame_follows_the_opacity() {
        let alpha = |opacity| {
            fills(&tooltip_frame(0.0, 0.0, 10.0, 8.0, opacity))
                .iter()
                .map(|(_, c)| c >> 24)
                .max()
                .unwrap()
        };
        assert_eq!(alpha(1.0), 0xF0);
        assert_eq!(alpha(0.25), 0x3C);
    }

    #[test]
    fn an_empty_corner_puts_the_content_where_0_1_0_did() {
        // `extras_hud::Corners::place` leaves 2 pixels from the edge.
        assert_eq!(2.0 + CORNER_GAP + FRAME, MARGIN);
    }
}
