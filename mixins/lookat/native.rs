// LookAt's adapter to the game (Horizon 142388e1c136e622652ec7505af15d8ad195e041,
// see horizon-mixin.toml). Injected as `crate::game::lookat_native`: it reads
// what the crosshair hits from the window's game, hands it to the
// game-independent `crate::game::lookat` (runtime.rs), and draws the box it
// lays out with the HUD's own renderer, in the `native_hud` hook point.
//
// It reads the game's own state, not only the hook points' helpers (which
// have no ray, no entities and no item icons), so the bundle is pinned to
// the commit above; README.md, "Hook points and pins", lists what and why.
use super::lookat::{self as lk, Draw, EntityHit, Position, Settings, Target};
use super::{Game, ray_box, view_vector};
use horizon_client::extras::Corner;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::mobs::Mob;
use horizon_client::{boss_overlay, debug_overlay};
use std::sync::Mutex;

/// The settings, read from their files once and kept up to date by the
/// settings page (both run on the window's thread).
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

fn settings(game: &Game<'_>) -> Settings {
    let mut kept = SETTINGS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    kept.get_or_insert_with(|| Settings::load(|key| game.native_setting(lk::ID, key)))
        .clone()
}

/// Item names in the player's language, as the item tooltips show them.
struct Items<'g, 'r>(&'g Game<'r>);
impl lk::ItemNames for Items<'_, '_> {
    fn item_name(&self, item: &str) -> Option<String> {
        let game = self.0;
        game.res.items().get(item)?;
        Some(game.pack.names.display_item(item))
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
}

impl Game<'_> {
    /// `native_hud` (a hook point): the box, over the rest of the HUD.
    pub(super) fn lookat_hud(&self, batch: &mut HudBatch, corners: &mut Corners) {
        if !self.native_on(lk::ID) {
            return;
        }
        let s = settings(self);
        let Some(target) = self.lookat_target(&s) else {
            return;
        };
        let view = lk::view(&target, &s, &Items(self));
        let width = |text: &str| self.native_text_width(text);
        let (w, h) = lk::content_size(&view, &width);
        let unscaled = (w + 2.0 * lk::FRAME, h + 2.0 * lk::FRAME);
        // F3's debug screen has a column in each top corner (this commit's
        // `native_corner_shown` only reports the left one): with it open,
        // the box goes to the top centre, shrunk to keep clear of both.
        let f3 = self.lookat_f3_open();
        let corner = match s.position() {
            Position::Top => None,
            Position::TopLeft => Some(Corner::TopLeft),
            Position::TopRight => Some(Corner::TopRight),
        };
        let corner = corner.filter(|&c| !f3 && self.native_corner_shown(c));
        let scale = match (corner, f3) {
            (None, true) => {
                let ratio = self.lookat_f3_ratio(batch);
                lk::beside_f3(s.size(), unscaled.0, batch.width, ratio)
            }
            _ => s.size(),
        };
        let outer = (unscaled.0 * scale, unscaled.1 * scale);
        let (left, top) = match corner {
            Some(c) => {
                let gap = lk::CORNER_GAP;
                let (x, y) = corners.place(c, outer.0 + 2.0 * gap, outer.1 + 2.0 * gap);
                ((x + gap).round(), y + gap)
            }
            None => {
                let bosses = self.boss_overlay.bars(boss_overlay::now_millis()).len();
                (
                    lk::top_centre(batch.width, outer.0),
                    lk::top_centre_top(bosses, batch.height),
                )
            }
        };
        let content = (left + lk::FRAME, top + lk::FRAME);
        let start = batch.vertices.len();
        for d in lk::draws(&view, &s, content, &width) {
            match d {
                Draw::Fill { rect, argb } => self.native_fill(batch, rect, argb),
                Draw::Text { text, at, rgb } => self.native_text(batch, &text, at, rgb, true),
                Draw::Item { item, at } => {
                    self.draw_item_model(batch, &ItemStack::new(&item, 1), at[0], at[1]);
                }
                Draw::Sprite { sprite, at } => batch.sprite(&self.gui.atlas, sprite, at[0], at[1]),
            }
        }
        if scale != 1.0 {
            batch.scale_since(start, left, top, scale, scale);
        }
    }

    /// Whether F3's debug screen is open: the top left is the corner every
    /// Horizon reports taken then (`native_corner_shown`).
    fn lookat_f3_open(&self) -> bool {
        !self.native_corner_shown(Corner::TopLeft)
    }

    /// The debug screen's pixels per GUI pixel (`draw_debug_overlay`'s
    /// scale: 1, or less under the default debug GUI scale).
    fn lookat_f3_ratio(&self, batch: &HudBatch) -> f32 {
        let framebuffer = self.renderer.as_ref().map_or_else(
            || [batch.width * batch.scale, batch.height * batch.scale].map(|v| v as i32),
            |r| [r.config.width, r.config.height].map(|v| v as i32),
        );
        let (ratio, _) = debug_overlay::scale(
            framebuffer,
            batch.scale as i32,
            self.settings.file.int("debugGuiScale"),
            self.settings.force_unicode,
            [batch.width as i32, batch.height as i32],
        );
        ratio
    }

    /// The block or entity under the crosshair within the range, nearest
    /// first; none outside a world.
    fn lookat_target(&self, s: &Settings) -> Option<Target> {
        let eye = self.eye_position()?;
        let dir = view_vector(self.yaw, self.pitch);
        let range = f64::from(s.range());
        let block = self.lookat_block(eye, dir, range);
        if s.entities() {
            let before = block.as_ref().map_or(range, |b| b.distance);
            if let Some(entity) = self.lookat_entity(eye, dir, range, before) {
                return Some(Target::Entity(entity));
            }
        }
        block.map(Target::Block)
    }

    /// The first block along the view, as `game.raycast` found it for the
    /// WebAssembly LookAt: through the loaded chunks as the crosshair's ray
    /// goes (fluids pass), then across the distant terrain's ground heights.
    fn lookat_block(&self, eye: [f64; 3], dir: [f64; 3], range: f64) -> Option<lk::BlockHit> {
        let (pos, point, state) = match self.clip_ray(eye, dir, range) {
            Some(hit) => (
                hit.pos,
                hit.point,
                self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]),
            ),
            None => self.lookat_lod_ray(eye, dir, range)?,
        };
        let facts = self.res.facts().get(state);
        let name: &str = &facts.name;
        let items = self.res.items();
        // Only for a block that drops nothing without the right tool.
        let harvest = if facts.requires_tool && facts.hardness >= 0.0 {
            lk::harvest_tool(|tag| items.block_in_tag(tag, name)).map(|tool| lk::Harvest {
                tool,
                ok: items.correct_for_drops(self.held(), name),
            })
        } else {
            None
        };
        Some(lk::BlockHit {
            block: name.to_owned(),
            pos,
            distance: distance(point, eye),
            harvest,
        })
    }

    /// Past the resident chunks: the view walked across the distant
    /// terrain's ground heights, a block at a time (`mods.rs`'s `lod_ray`,
    /// which is private there).
    fn lookat_lod_ray(
        &self,
        eye: [f64; 3],
        dir: [f64; 3],
        reach: f64,
    ) -> Option<([i32; 3], [f64; 3], horizon_world::registry::BlockStateId)> {
        let lod = self.lod.as_ref()?;
        let mut t = 0.0;
        while t <= reach {
            let p = [0, 1, 2].map(|i| eye[i] + dir[i] * t);
            let (x, z) = (p[0].floor() as i32, p[2].floor() as i32);
            if !self.world.loaded(x, z)
                && let Some((height, state)) = lod.ground(x, z)
                && p[1] <= f64::from(height)
            {
                return Some(([x, height - 1, z], [p[0], f64::from(height), p[2]], state));
            }
            t += 1.0;
        }
        None
    }

    /// The nearest entity the view hits before `before`: a mob or another
    /// player within the range (`mob_hit` with LookAt's range; invisible
    /// ones are left out), else an object or the dragon within the game's
    /// own reach (`remote_object_hit`, `dragon_hit`).
    fn lookat_entity(
        &self,
        eye: [f64; 3],
        dir: [f64; 3],
        range: f64,
        before: f64,
    ) -> Option<EntityHit> {
        let mut mob: Option<(f64, &Mob)> = None;
        for m in &self.mobs {
            if !m.alive() || m.invisible || Some(m.id) == self.riding {
                continue;
            }
            if let Some(t) = ray_box(eye, dir, range, m.bounds())
                && t < before
                && mob.is_none_or(|(best, _)| t < best)
            {
                mob = Some((t, m));
            }
        }
        let object = self
            .remote_object_hit(true)
            .filter(|h| h.distance < before && h.distance <= range)
            .map(|h| (h.distance, h.id));
        let dragon = self.dragon_hit().filter(|&(_, d)| d < before && d <= range);
        let nearest = [
            mob.map(|(d, _)| d),
            object.map(|(d, _)| d),
            dragon.map(|(_, d)| d),
        ]
        .into_iter()
        .flatten()
        .fold(f64::INFINITY, f64::min);
        let at = |d: f64| [0, 1, 2].map(|i| (eye[i] + dir[i] * d).floor() as i32);
        let items = Items(self);
        let item_name = |item: &str| {
            lk::ItemNames::item_name(&items, item)
                .unwrap_or_else(|| lk::spell_out(lk::split_id(item).1))
        };
        if let Some((d, m)) = mob.filter(|&(d, _)| d <= nearest) {
            let egg = lk::spawn_egg(&m.kind.id);
            let p = m.position;
            return Some(EntityHit {
                kind: m.kind.id.clone(),
                name: self.lookat_mob_name(m),
                icon: self.res.items().get(&egg).map(|_| egg),
                note: None,
                health: Some((m.health, horizon_client::mobs::finalize::max_health(m))),
                pos: [p.x, p.y, p.z].map(|v| v.floor() as i32),
                distance: d,
            });
        }
        if let Some((d, id)) = object.filter(|&(d, _)| d <= nearest) {
            // An item frame is named as one, with what it holds under it;
            // the others after the item they are picked as.
            let frame = self.item_frames.iter().find(|f| f.id as i32 == id);
            let (item, note) = match frame {
                Some(f) => {
                    let held = f.item.as_ref().map(|s| s.item.to_string());
                    (f.kind().to_owned(), held)
                }
                None => (self.entity_pick_result(id)?.item.to_string(), None),
            };
            return Some(EntityHit {
                kind: item.clone(),
                name: item_name(&item),
                icon: note.clone().or(Some(item)),
                note: note.as_deref().map(item_name),
                health: None,
                pos: at(d),
                distance: d,
            });
        }
        let (_, d) = dragon?;
        let kind = "minecraft:ender_dragon";
        let egg = lk::spawn_egg(kind);
        Some(EntityHit {
            kind: kind.to_owned(),
            name: self.lookat_entity_name(kind),
            icon: self.res.items().get(&egg).map(|_| egg),
            note: None,
            health: None,
            pos: at(d),
            distance: d,
        })
    }

    /// A mob's custom name, a player's name from the player list, else its
    /// type's name in the player's language.
    fn lookat_mob_name(&self, m: &Mob) -> String {
        if let Some(name) = &m.custom_name_text {
            return name.plain();
        }
        if let Some(name) = &m.custom_name {
            return name.clone();
        }
        if m.kind.id == "minecraft:player"
            && let Some((_, name)) = self
                .remote_player_names()
                .into_iter()
                .find(|(id, _)| *id == m.id)
        {
            return name;
        }
        self.lookat_entity_name(&m.kind.id)
    }

    /// `entity.<namespace>.<path>` in the player's language, else the id
    /// spelled out.
    fn lookat_entity_name(&self, kind: &str) -> String {
        let (namespace, path) = lk::split_id(kind);
        self.pack
            .names
            .display_get(&format!("entity.{namespace}.{path}"))
            .unwrap_or_else(|| lk::spell_out(path))
    }

    /// `native_settings` (a hook point): the settings page's buttons.
    pub(super) fn lookat_settings(&self) -> Vec<String> {
        settings(self).labels()
    }

    /// `native_settings_click` (a hook point): the option behind `button`
    /// moves to its next value, is kept and shows from the next frame.
    pub(super) fn lookat_settings_click(&self, button: usize) {
        let mut s = settings(self);
        if !s.cycle(button) {
            return;
        }
        for (key, value) in s.stored() {
            self.native_set_setting(lk::ID, key, value);
        }
        *SETTINGS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(s);
    }
}
