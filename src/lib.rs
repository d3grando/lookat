//! LookAt: the block under the crosshair at the top of the screen, in a box
//! drawn as vanilla draws a tooltip: its item's icon, its name in the
//! player's language and the namespace it comes from. Within reach by
//! default; the settings page reaches further (up to the raycast's 1024
//! blocks, across distant terrain), moves the box, fades its background and
//! picks its lines ([`settings`]). It looks once a tick and redraws only
//! when what it shows changed.
mod names;
mod settings;

use horizon_mod_sdk::{
    Mod, Registration, game, recipes,
    render::{self, FillShape, ItemShape, Point, Rect, Shape, TextShape},
    storage,
    ui::{self, ButtonWidget, Widget},
};
use names::Names;
use settings::{Position, Settings};

/// Vanilla's tooltip (TooltipRenderUtil): a near-black purple fill and a
/// frame fading from violet at the top to a darker violet at the bottom,
/// 0xAARRGGBB as `render` takes them.
const BACKGROUND: u32 = 0xF010_0010;
const FRAME_TOP: u32 = 0x5050_00FF;
const FRAME_BOTTOM: u32 = 0x5028_007F;
/// Text colours (0xRRGGBB): the name white, the details gray and the
/// source blue, as vanilla's chat formatting codes `f`, `7` and `9`.
const NAME: u32 = 0xFF_FFFF;
const DETAILS: u32 = 0xAA_AAAA;
const SOURCE: u32 = 0x55_55FF;
/// GUI pixels: the content's distance from the screen's edges (the frame
/// takes 4 of it), the item icon's size and the gap between it and the text.
const MARGIN: f32 = 8.0;
const ICON: f32 = 16.0;
const ICON_GAP: f32 = 4.0;
/// A standing player's eyes above their feet: distances are from there.
const EYE_HEIGHT: f64 = 1.62;

/// What the box shows; a change redraws it.
#[derive(Clone, PartialEq)]
struct View {
    item: Option<String>,
    lines: Vec<(String, u32)>,
    /// The GUI's width, which places the box.
    screen_width: f32,
}

struct LookAt {
    settings: Settings,
    /// Item names, read once a world is open.
    names: Names,
    /// What is on screen now: `None` draws nothing.
    shown: Option<View>,
}

impl LookAt {
    /// The item table exists only in a world, and its names follow the
    /// language the player had when it opened: read it on entering, drop
    /// it on leaving.
    fn load_names(&mut self, in_world: bool) {
        if in_world && self.names.is_empty() {
            self.names = Names::new(recipes::items().into_iter().map(|i| (i.id, i.name)));
        } else if !in_world && !self.names.is_empty() {
            self.names = Names::default();
        }
    }

    fn view(&self) -> Option<View> {
        let s = &self.settings;
        let player = game::get_player()?;
        let hit = game::raycast(s.range())?;
        let block = self.names.describe(&hit.block);
        let mut lines = vec![(block.name, NAME)];
        if s.details() {
            let (dx, dy, dz) = (
                hit.x - player.x,
                hit.y - (player.y + EYE_HEIGHT),
                hit.z - player.z,
            );
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            lines.push((
                format!(
                    "{} {} {}  ·  {distance:.1} m",
                    hit.block_x, hit.block_y, hit.block_z
                ),
                DETAILS,
            ));
        }
        if s.source() {
            lines.push((block.source, SOURCE));
        }
        Some(View {
            item: block.item.filter(|_| s.icon()),
            lines,
            screen_width: render::gui_size().0,
        })
    }

    fn draw(&mut self, view: Option<View>) {
        if view == self.shown {
            return;
        }
        match &view {
            Some(v) => render::set_hud(&shapes(v, &self.settings)),
            None => render::set_hud(&[]),
        }
        self.shown = view;
    }

    /// The settings page: the screen's title is the mod's name already.
    fn page(&self) {
        let buttons: Vec<Widget> = self
            .settings
            .buttons()
            .map(|(id, label)| Widget::Button(ButtonWidget { id, label }))
            .collect();
        ui::set_settings(&buttons);
    }

    fn save(&self) {
        for (key, value) in self.settings.stored() {
            let _ = storage::set(key, value.as_bytes());
        }
    }
}

/// The box: a vanilla tooltip around the icon and the lines, at the top of
/// the screen. Lines are 10 pixels apart, the first 2 more, as a tooltip
/// spaces its title.
fn shapes(v: &View, s: &Settings) -> Vec<Shape> {
    let text_width = v
        .lines
        .iter()
        .map(|(line, _)| render::text_width(line))
        .fold(0.0, f32::max);
    let text_height = match v.lines.len() {
        1 => 8.0,
        n => 10.0 * n as f32,
    };
    let text_x = if v.item.is_some() {
        ICON + ICON_GAP
    } else {
        0.0
    };
    let (w, h) = (text_x + text_width, text_height.max(ICON));
    let x = match s.position() {
        Position::Top => ((v.screen_width - w) / 2.0).round(),
        Position::TopLeft => MARGIN,
        Position::TopRight => (v.screen_width - w - MARGIN).round(),
    };
    let y = MARGIN;
    let mut out = tooltip_frame(x, y, w, h, s.opacity());
    if let Some(item) = &v.item {
        out.push(Shape::Item(ItemShape {
            item: item.clone(),
            at: Point {
                x,
                y: y + ((h - ICON) / 2.0).round(),
            },
            scale: 1.0,
        }));
    }
    let mut line_y = y + ((h - text_height) / 2.0).round();
    for (i, (line, color)) in v.lines.iter().enumerate() {
        out.push(Shape::Text(TextShape {
            text: line.clone(),
            at: Point {
                x: x + text_x,
                y: line_y,
            },
            color: *color,
            centered: false,
            shadow: true,
        }));
        line_y += if i == 0 { 12.0 } else { 10.0 };
    }
    out
}

/// Vanilla's tooltip background and frame around a `w` × `h` content box at
/// (`x`, `y`), their alpha scaled by `opacity`. The sprites vanilla draws it
/// with stretch as nine-slices, which `render` doesn't do, so it is drawn
/// as the fills the sprites replaced: the frame's sides fade one pixel row
/// at a time.
fn tooltip_frame(x: f32, y: f32, w: f32, h: f32, opacity: f32) -> Vec<Shape> {
    let fill = |x: f32, y: f32, w: f32, h: f32, color: u32| {
        Shape::Fill(FillShape {
            rect: Rect { x, y, w, h },
            color: faded(color, opacity),
        })
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

impl Mod for LookAt {
    fn init() -> Result<(Self, Registration), String> {
        let settings =
            Settings::load(|key| storage::get(key).and_then(|v| String::from_utf8(v).ok()));
        let look = LookAt {
            settings,
            names: Names::default(),
            shown: None,
        };
        look.page();
        Ok((look, Registration::new().on_tick()))
    }

    fn tick(&mut self, _tick: u64) {
        self.load_names(game::get_player().is_some());
        let view = self.view();
        self.draw(view);
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        if !self.settings.cycle(id) {
            return;
        }
        self.save();
        self.page();
        // The look changed even where the view didn't: redraw on the next tick.
        self.shown = None;
        render::set_hud(&[]);
    }

    fn ui_settings_opened(&mut self) {
        self.page();
    }
}

horizon_mod_sdk::export_mod!(LookAt);

#[cfg(test)]
mod tests {
    use super::*;

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

    fn fills(shapes: &[Shape]) -> Vec<(Rect, u32)> {
        shapes
            .iter()
            .map(|s| match s {
                Shape::Fill(f) => (f.rect, f.color),
                _ => unreachable!("the frame is fills only"),
            })
            .collect()
    }

    #[test]
    fn the_frame_surrounds_the_content() {
        let rects: Vec<Rect> = fills(&tooltip_frame(100.0, 8.0, 50.0, 16.0, 1.0))
            .into_iter()
            .map(|(r, _)| r)
            .collect();
        let left = rects.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        let right = rects.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max);
        let top = rects.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        let bottom = rects.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max);
        assert_eq!((left, right, top, bottom), (96.0, 154.0, 4.0, 28.0));
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
}
