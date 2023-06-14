#[cfg(feature = "diagnostic")]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
        .register_rollback_component::<GlobalTransform>()
        .register_rollback_component::<Transform>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<Sleeping>()
        .register_rollback_component::<ActiveEvents>()
        .register_rollback_component::<CharacterActionInput>()
        .register_rollback_component::<AIActionRoutine>()
        .register_rollback_component::<Gun>()
        .register_rollback_component::<Equipped>()
        .register_rollback_component::<Health>()
        .register_rollback_component::<Dying>()
        // .register_rollback_component::<Children>()
        .build(&mut app);

    #[cfg(feature = "diagnostic")]
    app.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default());

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(EntropyGenerator::new(42))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::default(),
            ..default()
        })
        .add_state::<GameState>()
        .add_event::<GamePauseEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(create_window(800., 800.)),
            ..default()
        }))
        .add_plugin(MenuPlugin)
        .add_plugin(RapierPhysicsPlugin::<()>::default())
        .add_plugin(SpaceballsPhysicsPlugin)
        .configure_sets(
            // ggrs couldn't give two flying shits about this one
            (
                InputHandlingSet::InputReading,
                InputHandlingSet::ResponseProcessing,
            )
                .chain(),
        )
        .add_startup_system(standard_setup)
        .add_system(start_matchbox_socket.in_schedule(OnEnter(GameState::Matchmaking)))
        .add_system(wait_for_players.run_if(in_state(GameState::Matchmaking)))
        .add_system(summon_scene.in_schedule(OnEnter(GameState::InGame)))
        .add_system(despawn_everything.in_schedule(OnExit(GameState::InGame)))
        .add_system(sever_connection.in_schedule(OnExit(GameState::InGame)))
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
            // todo:mp de-chain, group independent systems into sets
            (
                calculate_character_velocity,
                handle_gunfire,
                handle_letting_gear_go,
                handle_gun_picking,
                handle_inventory_layout_change,
                handle_gun_arriving_at_rest,
                handle_bullet_collision_events,
                handle_railgun_penetration_damage,
                handle_death,
            )
                .chain()
                .in_set(InputHandlingSet::ResponseProcessing)
                .after(InputHandlingSet::InputReading)
                .in_schedule(GGRSSchedule),
        )
        .add_system(
            detect_desync
                .in_schedule(GGRSSchedule)
                .run_if(in_state(GameState::InGame)),
        )
        .add_system(handle_pause_input.run_if(in_state(GameState::InGame)))
        .add_system(handle_entities_out_of_bounds)
        .add_systems((
            handle_gun_ownership_cosmetic_change,
            handle_gun_idle_bobbing,
        ))
        .add_system(handle_browser_window_resizing)
        .add_system(
            calculate_main_camera_projection_scale
                .before(camera_system::<OrthographicProjection>)
                .in_base_set(CoreSet::PostUpdate),
        );

    if let Some(scene) = scene_arg {
        app.insert_resource(scene)
            .insert_resource(PlayerCount(1))
            .insert_resource(State::<GameState>(GameState::Matchmaking))
            .insert_resource(State::<MenuState>(MenuState::Disabled));
    }

    app.run();
}
