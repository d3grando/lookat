# LookAt

A [Horizon](https://github.com/horizon-rust/horizon) client mod: what is under your crosshair, at the top of the
screen, drawn as a vanilla tooltip. A block shows its item's icon, its name in your language and the namespace it comes
from; a mob shows its name and its health; and the box can also say which tool a block needs.

![LookAt naming a grass block at the top of the screen](docs/screenshot.png)

**0.2.0 is a native mod** (a horizon-mixins source bundle, `.hnmod`). Horizon is removing WebAssembly code mods, so the
WebAssembly LookAt (0.1.0, `.hmod`) is going away with them; this version replaces it with the same id, `lookat`, the
same look and the same settings, and it now sees entities.

> **Native mods are not sandboxed.** Unlike the WebAssembly LookAt, a native mod is Rust source compiled into your own
> Horizon executable and runs with your user rights. Read the source before building (it is small: `mixins/lookat/`).

## What's new in 0.2.0

- **Entities.** Mobs and other players show their name (a custom name or a player's name when they have one) and
  their **health**, as hearts (numbers past two rows of hearts, above 40 health) or as numbers (`❤ 15.5 / 20`).
  A mob's icon is its spawn egg. Boats, minecarts, armor stands, paintings, end crystals and item frames (with what
  they hold) show as the item they are picked as; the ender dragon shows its name. Invisible mobs are left out.
- **Harvest tool** (off by default): for a block that drops nothing without the right tool, the least tool that
  harvests it, green with ✔ when the item in your hand does, red with ✘ when it doesn't (`✘ Iron Pickaxe` on
  diamond ore with a stone pickaxe).
- **Size**: the whole box, text included, at 100%, 125%, 150% or 200%. The WebAssembly API couldn't scale text.
- **Corners share.** In the top left or top right, the box stacks under what Horizon already shows there (the
  minimap, status effect icons, other mods' boxes) instead of over it. In an empty corner it sits exactly where 0.1.0
  put it. With F3 open, a top-left box moves to the top centre.
- **Real text widths and eyes.** Widths come from the game's font, and the distance is from your actual eyes
  (sneaking, swimming), not a standing player's.

The default look is 0.1.0's, pixel for pixel: the same fills for the frame, the same icon and line placement, the same
colours (a unit test draws both and compares them). Your 0.1.0 settings carry over: the native mod reads the same files,
`horizon-mods/.horizon-mod-data/lookat/<option>`.

## Settings

Options → Mods → LookAt → Settings. Each button moves its option to the next value; it applies on the next frame.

| Option | Values (first is the default) |
| --- | --- |
| Range | Reach (5 blocks), 16, 64, 1024 blocks: past the loaded chunks, across distant terrain |
| Position | Top, Top left, Top right |
| Size *(new)* | 100%, 125%, 150%, 200% |
| Background opacity | 100%, 75%, 50%, 25% |
| Item icon | On, Off |
| Where it comes from | On, Off: the namespace, spelled out (`Minecraft`) |
| Position and distance | Off, On: the block's (or entity's) coordinates and its distance from your eyes |
| Entities *(new)* | On, Off: Off shows only blocks, as 0.1.0 did |
| Health *(new)* | Hearts, Numbers, Off |
| Harvest tool *(new)* | Off, On |

The row's own button in Options → Mods turns LookAt off (it then draws nothing) until you turn it on again; the
launcher's next build leaves a mod that is off out.

## Install

Native mods are built by the Horizon launcher into a game of their own. Install LookAt from the launcher's mod list
and let it build the native mods with **trust native code**; then play the **Native mods** instance. If you had the
WebAssembly LookAt, you can remove it: with the native one built in, Horizon doesn't run a WebAssembly mod of the
same id.

From the command line, build this folder (or a folder holding it and other native mods, one subfolder each):

```sh
horizon-launcher native-build <this folder> --version <installed Horizon version> --trust-native-code
```

`--version` is the installed release whose game files the build reuses; the code is always the pinned commit's (below).

## Hook points and pins

Horizon's native mods hook **hook points** (`game/native_hooks.rs`, `NATIVE_HOOKS_API` 1): functions kept stable from
release to release. LookAt hooks only hook points, all with HEAD hooks:

| Hook point | For |
| --- | --- |
| `native_hud` | drawing the box, with the HUD (hidden under F1), after Horizon's corner boxes; `Corners` stacks it in a corner |
| `native_mods` | the row in Options → Mods |
| `native_settings`, `native_settings_click` | the settings page |

and uses their helpers for everything they offer: `native_on` (the off switch), `native_setting` /
`native_set_setting` (the settings files), `native_fill`, `native_text`, `native_text_width` (the box) and
`native_corner_shown` (F3).

The helpers don't see what the crosshair hits, so `native.rs` also reads the game's own state, and the bundle is
**pinned** to one Horizon commit (`horizon_revision` in `horizon-mixin.toml`) rather than to a hook-points version. No
game function is hooked; these are the internals it reads, each because no hook point or helper offers it:

| Game internal | Why |
| --- | --- |
| `eye_position`, `view_vector`, `clip_ray`, `lod` (`Lod::ground`) | the crosshair's ray through the loaded chunks and on across the distant terrain, as `game.raycast` gave the WebAssembly LookAt (its `lod_ray` is private to `mods.rs`, so `native.rs` repeats it) |
| `res.facts()`, `world.block` | the block hit and its facts (name, hardness, needs a tool) |
| `res.items()`: `get`, `block_in_tag`, `correct_for_drops`; `held` | item existence, the `mineable/*` and `needs_*_tool` block tags, and whether the held item harvests the block |
| `pack.names` (`display_item`, `display_get`) | item and entity names in your language, as tooltips show them |
| `mobs`, `ray_box`, `riding`, `remote_player_names`, `mobs::finalize::max_health` | the mob or player under the crosshair within LookAt's range, its name and its health |
| `remote_object_hit`, `entity_pick_result`, `item_frames`, `dragon_hit` | boats, minecarts, armor stands, paintings, end crystals, item frames and the dragon, within the game's own reach |
| `draw_item_model`, `gui.atlas` (`HudBatch::sprite`, `scale_since`) | item icons and heart sprites as the HUD draws them, and the Size option |

Pinned: Horizon `142388e1c136e622652ec7505af15d8ad195e041` and horizon-mixins
`0d6b103c70633d8fd9443d8d0acd3aadaacedb06`, game 26.3, the same as TreeFall 0.3.0 and Mason 0.2.0, so the three build
into one game (checked: `horizon-mixins plan` of all three, 20 injections and 14 modules, no conflict). When hook
points cover the crosshair's target, entities and item icons, the pin can go.

## Limits

- Mobs and players are seen within LookAt's range; boats, minecarts, armor stands, paintings, item frames, end
  crystals and the dragon only within the game's own reach (3 blocks, 5 in creative), as the game aims at them.
- The ender dragon's health isn't shown.
- A block without an item of its own (a crop, a wall torch, a potted plant) is named after the item vanilla's pick
  block gives; one with none at all (fire, a portal) gets its id spelled out in English.
- The harvest tool covers the four `mineable` tags (pickaxe, axe, shovel, hoe); blocks that need shears or a sword
  (cobwebs) show no tool.
- Vanilla draws its tooltip with nine-slice sprites; LookAt draws the fills those sprites replaced, as 0.1.0 did, so
  a resource pack's tooltip sprites don't change it.

## Development

`mixins/lookat/`:

- `horizon-mixin.toml`: two support modules and four HEAD hooks on hook points (above).
- `runtime.rs`: all decisions as pure code with unit tests: naming blocks, the harvest tool, the settings, what the
  box shows and its layout (`Draw`s). It is both this repository's Cargo library (through `src/lib.rs`) and the
  injected `crate::game::lookat` module.
- `native.rs`: the adapter to the game (`crate::game::lookat_native`).
- `hud.rs`, `list.rs`, `settings.rs`, `click.rs`: the hook helpers.

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
python tools/pack_hnmod.py   # dist/lookat-<version>.hnmod
```

`cargo fmt` formats the game-side files too (`src/lib.rs` declares them under `cfg(any())`); only the game compiles
them. The WebAssembly version is on the `main` branch (0.1.0).

Written with [Claude Code](https://claude.com/claude-code).

## License

[MIT](LICENSE)
