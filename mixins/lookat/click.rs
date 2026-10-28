// native_hooks::Game::native_settings_click (a hook point): a click on
// LookAt's settings page moves the option to its next value and keeps it.
fn lookat_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id == crate::game::lookat::ID {
        game.lookat_settings_click(button);
    }
}
