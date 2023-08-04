#[cfg(feature = "diagnostic")]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use cosmic_spaceball_tactical_action_arena::*;

fn main() {
    let scene_arg = parse_scene_ext_input();

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    GGRSPlugin::<GGRSConfig>::new()
        .with_input_system(process_input)
        .register_rollback_resource::<EntropyGenerator>()
        // todo:mp figure out why rolling back `SpawnQueue` forces re-rolls of EntropyGenerator
        // not critical
        // .register_rollback_resource::<SpawnQueue>()
        .register_rollback_component::<GlobalTransform>()
        .register_rollback_component::<Transform>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<Sleeping>()
        .register_rollback_component::<ActiveEvents>()
        .register_rollback_component::<SpawnPoint>()
        .register_rollback_component::<CharacterActionInput>()
        .register_rollback_component::<AIActionRoutine>()
        .register_rollback_component::<Gun>()
        .register_rollback_component::<Equipped>()
        .register_rollback_component::<LastUnequippedAt>()
        .register_rollback_component::<Health>()
        .register_rollback_component::<Dying>()
        // .register_rollback_component::<Children>()
        .build(&mut app);

    #[cfg(feature = "diagnostic")]
    app.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default());

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::default(),
            ..default()
        })
        .init_resource::<EntropyGenerator>()
        // probably displace to plugin
        .init_resource::<SpawnQueue>()
        .add_state::<GameState>()
        .add_state::<LimboState>()
        .add_event::<GamePauseEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(create_window(800., 800.)),
            ..default()
        }))
        .add_plugins(MultiplayerPlugins)
        .add_plugins(UIPlugins)
        .add_plugin(RapierPhysicsPlugin::<()>::default())
        .add_plugin(SpaceballsPhysicsPlugin)
        .add_plugin(EasterAnnouncementPlugin)
        .configure_sets(
            // ggrs couldn't give two flying shits about this one
            (
                InputHandlingSet::InputReading,
                InputHandlingSet::ResponseProcessing,
            )
                .chain(),
        )
        .add_startup_system(standard_setup)
        .add_system(
            reset_entropy
                .in_schedule(OnEnter(GameState::InGame))
                .in_base_set(CoreSet::PreUpdate),
        )
        .add_system(summon_scene.in_schedule(OnEnter(GameState::InGame)))
        // maybe just despawn literally everything, but make `standard_setup` apply
        .add_system(despawn_everything.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(despawn_everything.in_schedule(OnExit(GameState::InGame)))
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
                handle_gun_cleanup,
                handle_inventory_layout_change,
                handle_gun_arriving_at_rest,
                handle_bullet_collision_events,
                handle_railgun_penetration_damage,
                handle_death,
                send_new_players_joined,
                handle_respawn_point_occupation,
                handle_player_respawning,
            )
                .chain()
                .in_set(InputHandlingSet::ResponseProcessing)
                .after(InputHandlingSet::InputReading)
                .in_schedule(GGRSSchedule),
        )
        .add_systems(
            (
                handle_match_time, /*.run_if(not(in_state(MenuState::MatchEnd)))*/
            )
                .in_schedule(GGRSSchedule),
        )
        .add_system(handle_respawn_point_occupation.in_schedule(OnEnter(GameState::InGame)))
        .add_system(reset_spawn_queue.in_schedule(OnExit(GameState::InGame)))
        .add_system(handle_waiting_for_rematch_in_limbo.run_if(in_state(LimboState::Limbo)))
        .add_system(handle_pause_input.run_if(in_state(GameState::InGame)))
        // might duplicate if not in GGRS
        .add_system(handle_reporting_death.run_if(in_state(GameState::InGame)))
        // probably should be in GGRS
        .add_system(handle_entities_out_of_bounds)
        .add_systems((
            handle_gun_ownership_cosmetic_change,
            handle_gun_idle_bobbing,
        ))
        .add_system(
            calculate_main_camera_projection_scale
                .before(camera_system::<OrthographicProjection>)
                .in_base_set(CoreSet::PostUpdate),
        );

    #[cfg(target_arch = "wasm32")]
    app.add_system(handle_browser_window_resizing);

    if let Some(scene) = scene_arg {
        app.insert_resource(scene)
            .insert_resource(PlayerCount(1))
            .insert_resource(State::<GameState>(GameState::Matchmaking))
            .insert_resource(State::<MenuState>(MenuState::Disabled));
    }

    app.run();
}
