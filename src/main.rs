use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(create_window_descriptor((800.0, 800.0)))
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(scene_arg)
        .init_resource::<PlayerInput>()
        .add_startup_system(summon_scene)
        .add_event::<CharacterDamagedEvent>()
        .add_system(handle_input)
        .add_system(handle_movement)
        .add_system(handle_bullet_spawn)
        .add_system(handle_bullet_flight)
        .add_system(handle_bullet_collision.after(handle_bullet_flight))
        .add_system(calculate_damages.after(handle_bullet_collision))
        .run();
}
