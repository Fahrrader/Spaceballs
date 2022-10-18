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
        .add_plugin(PhysicsPlugin::default())
        .add_event::<EntityDamagedEvent>()
        .add_startup_system(summon_scene)
        .add_system(handle_gamepad_connections)
        .add_system_set(
            SystemSet::new()
                .label("handle_input")
                .with_system(reset_input)
                .with_system(handle_keyboard_input.after(reset_input))
                .with_system(handle_gamepad_input.after(reset_input))
                .with_system(handle_ai_input.after(reset_input)),
        )
        .add_system(calculate_character_velocity.after("handle_input"))
        .add_system(handle_gunfire.after(calculate_character_velocity))
        .add_system(handle_bullets_out_of_bounds.after(handle_gunfire))
        .add_system(handle_bullet_collision_events)
        .add_system(handle_damage.after(handle_bullet_collision_events))
        .add_system(handle_browser_window_resizing)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            calculate_projection_scale.before(camera_system::<OrthographicProjection>),
        )
        .run();
}
