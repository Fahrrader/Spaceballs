use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(create_window_descriptor((800.0, 800.0)))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(scene_arg)
        .add_plugins(DefaultPlugins)
        .add_event::<CollisionEvent>()
        .add_event::<EntityDamagedEvent>()
        .add_startup_system(summon_scene)
        .add_system(handle_player_input)
        .add_system(handle_ai_input)
        .add_system(calculate_character_velocity.after(handle_player_input))
        .add_system(handle_movement.after(calculate_character_velocity))
        .add_system(handle_gunfire.after(handle_player_input))
        .add_system(handle_bullet_flight.after(handle_gunfire))
        .add_system(handle_collision.after(handle_bullet_flight))
        .add_system(handle_bullet_collision_events.after(handle_collision))
        .add_system(calculate_damages.after(handle_bullet_collision_events))
        .run();
}
