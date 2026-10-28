// LookAt's game-independent logic as a Cargo library, for its unit tests:
// the same file the bundle adds to the game as `crate::game::lookat`
// (mixins/lookat/runtime.rs).
pub use runtime::*;

#[path = "../mixins/lookat/runtime.rs"]
mod runtime;

// The game-side files, never compiled here (`cfg(any())` is never true) but
// declared so that `cargo fmt` formats and checks them too: the adapter, and
// each hook's helper (one function each, injected into a hook point).
#[cfg(any())]
#[path = "../mixins/lookat/click.rs"]
mod hook_click;
#[cfg(any())]
#[path = "../mixins/lookat/hud.rs"]
mod hook_hud;
#[cfg(any())]
#[path = "../mixins/lookat/list.rs"]
mod hook_list;
#[cfg(any())]
#[path = "../mixins/lookat/settings.rs"]
mod hook_settings;
#[cfg(any())]
#[path = "../mixins/lookat/native.rs"]
mod lookat_native;
