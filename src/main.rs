use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // todo:mp probably add a separate
    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(process_input)
        .register_rollback_component::<Transform>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<CharacterActionInput>()
        //.register_rollback_component::<Gun>()
        //.register_rollback_component::<Children>()
        .build(&mut app);

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(scene_arg)
        .insert_resource(RandomState(StdRng::seed_from_u64(42))) // probably refactor for async
        .insert_resource(RapierConfiguration {
            gravity: Vec2::default(),
            ..default()
        })
        //.register_type::<CharacterActionInput>() // todo make plugins, register all respective types, will make development easier
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(create_window(800., 800.)),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<()>::default())
        .add_plugin(SpaceballsPhysicsPlugin)
        /*.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())*/
        // todo summon scene only on InGame
        .add_state::<GameState>()
        .configure_set(InputHandlingSet::ResponseProcessing.after(InputHandlingSet::InputReading))
        .add_startup_systems((summon_scene, start_matchbox_socket))
        .add_system(wait_for_players.run_if(in_state(GameState::Matchmaking)))
        .add_system(handle_gamepad_connections)
        /*.add_system(reset_input.in_set(InputHandlingSet::MediaReading))
        .add_systems(
            (handle_keyboard_input, handle_gamepad_input, handle_ai_input)
                .after(reset_input)
                .in_set(InputHandlingSet::MediaReading),
        )*/
        .add_system(
            handle_ai_input
                .in_set(InputHandlingSet::InputReading)
                .run_if(in_state(GameState::InGame)),
        )
        .add_system(
            handle_online_player_input
                .in_set(InputHandlingSet::InputReading)
                .in_schedule(GGRSSchedule),
        )
        .add_systems(
            (
                // todo:mp not good, don't make them share and bottleneck on a single input component
                calculate_character_velocity,
                handle_gunfire,
                handle_gun_picking,
                handle_letting_gear_go,
            )
                .chain()
                .after(handle_online_player_input)
                .in_set(InputHandlingSet::ResponseProcessing)
                .in_schedule(GGRSSchedule),
        )
        .add_system(handle_entities_out_of_bounds)
        .add_system(handle_bullet_collision_events)
        .add_system(handle_railgun_penetration_damage)
        .add_systems((
            handle_inventory_layout_change,
            handle_gun_idle_bobbing,
            handle_gun_arriving_at_rest,
        ))
        // probably execute latest
        .add_system(
            handle_death
                .after(handle_bullet_collision_events) // todo bullet collision plugin / system set
                .after(handle_letting_gear_go),
        )
        .add_system(handle_browser_window_resizing)
        .add_system(
            calculate_main_camera_projection_scale
                .before(camera_system::<OrthographicProjection>)
                .in_base_set(CoreSet::PostUpdate),
        )
        .run();
}
