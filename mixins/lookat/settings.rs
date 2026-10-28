// native_hooks::Game::native_settings (a hook point): LookAt's settings page.
fn lookat_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == crate::game::lookat::ID {
        buttons.extend(game.lookat_settings());
    }
}
