use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // todo:mp probably run systems used by ggrs also outside of ggrs schedule for single-player if chosen in menu
    GGRSPlugin::<GGRSConfig>::new()
        .with_input_system(process_input)
        .register_rollback_resource::<EntropyGenerator>()
        .register_rollback_component::<Transform>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<ActiveEvents>()
        .register_rollback_component::<CharacterActionInput>()
        .register_rollback_component::<AIActionRoutine>()
        .register_rollback_component::<Gun>()
        .register_rollback_component::<Equipped>()
        .register_rollback_component::<Health>()
        .register_rollback_component::<Dying>()
        // .register_rollback_component::<Children>()
        .build(&mut app);

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(scene_arg)
        .insert_resource(EntropyGenerator::new(42))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::default(),
            ..default()
        })
        // make plugins, register all respective types, will make development easier
        //.register_type::<CharacterActionInput>()
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
        .configure_sets(
            // ggrs couldn't give two flying shits about this one
            (
                InputHandlingSet::InputReading,
                InputHandlingSet::ResponseProcessing,
            )
                .chain(),
        )
        .add_startup_systems((summon_scene, start_matchbox_socket))
        .add_system(wait_for_players.run_if(in_state(GameState::Matchmaking)))
        //.add_system(summon_scene.in_schedule(OnEnter(GameState::InGame)))
        .add_system(handle_gamepad_connections)
        // todo:mp action routine gets abnormally long if in rollback together with ai input, might be interesting to look into
        .add_system(
            handle_ai_input
                .run_if(in_state(GameState::InGame))
                .in_set(InputHandlingSet::InputReading),
        )
        .add_system(
            handle_online_player_input
                .run_if(in_state(GameState::InGame))
                .in_set(InputHandlingSet::InputReading)
                .in_schedule(GGRSSchedule),
        )
        .add_systems(
            (
                calculate_character_velocity,
                handle_gunfire,
                handle_letting_gear_go,
                handle_gun_picking,
            )
                .chain()
                .in_set(InputHandlingSet::ResponseProcessing)
                .after(InputHandlingSet::InputReading)
                .in_schedule(GGRSSchedule),
        )
        .add_system(handle_entities_out_of_bounds)
        .add_system(handle_bullet_collision_events)
        .add_system(handle_railgun_penetration_damage)
        .add_systems((
            handle_gun_ownership_change,
            // todo:mp does it run anyway after rollback because "changed"?
            handle_inventory_layout_change,
            handle_gun_idle_bobbing,
            handle_gun_arriving_at_rest,
        ))
        // probably execute latest -- todo:mp add to GGRS?
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
