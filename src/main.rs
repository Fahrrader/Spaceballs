use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(scene_arg)
        .insert_resource(RandomState(StdRng::seed_from_u64(42))) // probably refactor for async
        .insert_resource(RapierConfiguration {
            gravity: Vec2::default(),
            ..default()
        })
        //.register_type::<CharacterActionInput>() // todo make plugins, register all respective types, will make development easier
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: create_window_descriptor((800.0, 800.0)),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<()>::default())
        .add_plugin(SpaceballsPhysicsPlugin)
        /*.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())*/
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
        .add_system(handle_gunfire.after("handle_input"))
        .add_system(handle_gun_picking.after("handle_input"))
        .add_system(handle_letting_gear_go.after("handle_input"))
        .add_system(handle_entities_out_of_bounds)
        .add_system(handle_railgun_things)
        .add_system(display_events)
        .add_system(handle_damage_from_railgun_things)
        .add_system(handle_bullet_collision_events)
        .add_system(handle_inventory_layout_change)
        .add_system(handle_gun_idle_bobbing)
        .add_system(handle_gun_arriving_at_rest)
        // probably execute latest
        .add_system(
            handle_death
                .after(handle_bullet_collision_events) // todo bullet collision plugin / system set
                .after(handle_letting_gear_go),
        )
        .add_system(handle_browser_window_resizing)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            calculate_projection_scale.before(camera_system::<OrthographicProjection>),
        )
        .run();
}
