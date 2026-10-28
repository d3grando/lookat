//! What a block is called: no API reads block names, so a block is named
//! after its item, in the player's language. A block without an item of its
//! own (a wall torch, a crop, a potted plant) takes the item vanilla's pick
//! block gives for it; one with neither (fire, a portal) gets its id spelled
//! out in English.
use std::collections::HashMap;

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

/// A block as the overlay shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    /// The item drawn as its icon; `None` when it has none.
    pub item: Option<String>,
    /// Its name in the player's language, or its id spelled out.
    pub name: String,
    /// The namespace it comes from, spelled out: `Minecraft`.
    pub source: String,
}

/// Item names by id, from the game's item table.
#[derive(Default)]
pub struct Names {
    by_id: HashMap<String, String>,
}

impl Names {
    pub fn new(items: impl IntoIterator<Item = (String, String)>) -> Self {
        Names {
            by_id: items.into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// `block` as the raycast reports it: `minecraft:oak_log`, with or
    /// without a `[state]`.
    pub fn describe(&self, block: &str) -> Described {
        let id = block.split_once('[').map_or(block, |(id, _)| id);
        let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
        let named = |path: &str| {
            let item = format!("{namespace}:{path}");
            self.by_id.get(&item).map(|name| (item, name.clone()))
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
fn spell_out(id: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Names {
        Names::new(
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
            ]
            .map(|(id, name)| (id.to_owned(), name.to_owned())),
        )
    }

    fn named(block: &str) -> (Option<String>, String) {
        let d = names().describe(block);
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
        assert_eq!(names().describe("minecraft:stone").source, "Minecraft");
        assert_eq!(names().describe("cool_mod:ruby_ore").source, "Cool Mod");
        // A bare id is vanilla's, as the game reads it.
        assert_eq!(names().describe("stone").source, "Minecraft");
    }

    #[test]
    fn an_empty_table_still_names_every_block() {
        let d = Names::default().describe("minecraft:oak_log");
        assert_eq!((d.item, d.name.as_str()), (None, "Oak Log"));
    }
}
