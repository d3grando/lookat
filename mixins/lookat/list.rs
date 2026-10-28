// native_hooks::Game::native_mods (a hook point): LookAt's row in Options > Mods.
fn lookat_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::lookat as lk;
    mods.push(crate::game::native_hooks::NativeMod {
        id: lk::ID.into(),
        title: format!("LookAt {}", lk::VERSION),
        description: lk::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
