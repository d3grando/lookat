// native_hooks::Game::native_hud (a hook point): the box over the HUD, where
// the WebAssembly mods' shapes are drawn (after Horizon's corner boxes).
fn lookat_hud(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    game.lookat_hud(batch, corners);
}
