use crate::ai::AIActionRoutine;
use crate::controls::CharacterActionInput;
use crate::guns::{reset_gun_transform, team_paint_gun, Equipped, Gun, GunBundle, GunPreset};
use crate::health::{Health, HitPoints};
use crate::physics::{
    popular_collider, ActiveEvents, CollisionLayer, KinematicsBundle, OngoingCollisions, RigidBody,
    Sensor, Velocity,
};
use crate::teams::{team_color, Team, TeamNumber};
use crate::EntropyGenerator;
use bevy::hierarchy::{BuildChildren, Children};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    Bundle, Changed, Commands, Component, Entity, Query, Sprite, SpriteBundle, Transform, With,
    Without,
};
use bevy::utils::default;
use std::f32::consts::PI;

// todo resize all sizes and speeds as percentages of screen-range
/// Standard size for a character body in the prime time of their life.
pub const CHARACTER_SIZE: f32 = 50.0;

/// Standard linear speed per second at full capacity in floating point units.
pub const CHARACTER_SPEED: f32 = CHARACTER_SIZE * 4.0;
/// Standard rotational speed at full capacity per second in radians.
pub const CHARACTER_RAD_SPEED: f32 = PI;

/// The velocity of a gun when thrown. It is only a part of the calculation, the current character velocity is also taken into account.
const GUN_THROW_SPEED: f32 = CHARACTER_SPEED * 2.0;
/// Throw away the gun, it spins. It's a good trick.
const GUN_THROW_SPIN_SPEED: f32 = 4.0 * PI;

/// Standard maximum health for a player character.
pub const CHARACTER_MAX_HEALTH: HitPoints = 100.0;

/// Common trait for all character bodies/bundles when they're not referring to BaseCharacterBundle.
pub trait BuildCharacter {
    /// Make a new character bundle yet to be spawned, with the transform of the initial placement, the team and the online player handle assigned.
    fn new(transform: Transform, team: TeamNumber, player_handle: usize) -> Self;
    /// Spawn a character bundle and attach equipment to it, returning spawned entities, character first.
    fn spawn_with_equipment(
        self,
        commands: &mut Commands,
        random_state: EntropyGenerator,
        equipment: Vec<GunPreset>,
    ) -> Vec<Entity>;
}

/// The Character base all other Character bundles should use and add to.
#[derive(Bundle)]
pub struct BaseCharacterBundle {
    pub action_input: CharacterActionInput,
    pub health: Health,
    pub team: Team,
    #[bundle]
    pub kinematics: KinematicsBundle,
    pub active_physics_events: ActiveEvents,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl BaseCharacterBundle {
    fn new(transform: Transform, team: TeamNumber) -> Self {
        Self {
            action_input: CharacterActionInput::default(),
            health: Health::new(CHARACTER_MAX_HEALTH),
            team: Team(team),
            kinematics: KinematicsBundle::new(
                popular_collider::square(CHARACTER_SIZE),
                &[CollisionLayer::Character],
                CollisionLayer::all(),
            ),
            active_physics_events: ActiveEvents::COLLISION_EVENTS,
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: team_color(team),
                    custom_size: Some(Vec2::new(CHARACTER_SIZE, CHARACTER_SIZE)),
                    ..default()
                },
                transform,
                ..default()
            },
        }
    }

    /// Create and insert some guns into the hands of a character.
    pub fn spawn_equipment(
        commands: &mut Commands,
        character_id: Entity,
        team: TeamNumber,
        mut random_state: EntropyGenerator,
        equipment: Vec<GunPreset>,
    ) -> Vec<Entity> {
        let mut result = Vec::with_capacity(equipment.len());

        for gun_preset in equipment {
            let gun_id = commands
                .spawn(GunBundle::new(gun_preset, None, random_state.fork()).with_paint_job(team))
                .id();

            equip_gear(commands, character_id, gun_id, gun_preset, None, None);
            result.push(gun_id);
        }
        result
    }

    fn spawn_with_equipment<CharacterBundle: BuildCharacter + Bundle>(
        bundle: CharacterBundle,
        team: TeamNumber,
        commands: &mut Commands,
        random_state: EntropyGenerator,
        equipment: Vec<GunPreset>,
    ) -> Vec<Entity> {
        let char_id = commands.spawn(bundle).id();

        let mut result = vec![char_id];
        result.append(&mut BaseCharacterBundle::spawn_equipment(
            commands,
            char_id,
            team,
            random_state,
            equipment,
        ));
        result
    }
}

/// Bundle for a Player Character, controlled over internet.
#[derive(Bundle)]
pub struct PlayerCharacterBundle {
    #[bundle]
    pub character_bundle: BaseCharacterBundle,
    pub player_marker: PlayerControlled,
}

/// Marker designating an entity controlled by a player.
#[derive(Component, Debug)]
pub struct PlayerControlled {
    pub handle: usize,
}

impl BuildCharacter for PlayerCharacterBundle {
    fn new(transform: Transform, team: TeamNumber, player_handle: usize) -> Self {
        Self {
            character_bundle: BaseCharacterBundle::new(transform, team),
            player_marker: PlayerControlled {
                handle: player_handle,
            },
        }
    }

    fn spawn_with_equipment(
        self,
        commands: &mut Commands,
        random_state: EntropyGenerator,
        equipment: Vec<GunPreset>,
    ) -> Vec<Entity> {
        let team = self.character_bundle.team.0;
        BaseCharacterBundle::spawn_with_equipment(self, team, commands, random_state, equipment)
    }
}

/// Bundle for an artificially-intelligent character.
#[derive(Bundle)]
pub struct AICharacterBundle {
    #[bundle]
    pub character_bundle: BaseCharacterBundle,
    pub player_marker: AiControlled,
    pub ai_controller: AIActionRoutine,
}

/// Marker designating an entity controlled by a player.
#[derive(Component, Debug)]
pub struct AiControlled;
// pub peer_handle: usize,

impl BuildCharacter for AICharacterBundle {
    fn new(transform: Transform, team: TeamNumber, _player_handle: usize) -> Self {
        Self {
            character_bundle: BaseCharacterBundle::new(transform, team),
            player_marker: AiControlled,
            ai_controller: AIActionRoutine::default(),
        }
    }

    fn spawn_with_equipment(
        self,
        commands: &mut Commands,
        random_state: EntropyGenerator,
        equipment: Vec<GunPreset>,
    ) -> Vec<Entity> {
        let team = self.character_bundle.team.0;
        BaseCharacterBundle::spawn_with_equipment(self, team, commands, random_state, equipment)
    }
}

/// Attach some equippable gear to a character and allow it to be interacted with.
/// Unchecked if actually equippable, or if the equipping entity is a character!
fn equip_gear(
    commands: &mut Commands,
    char_entity: Entity,
    gear_entity: Entity,
    // only guns for now
    gun_preset: GunPreset,
    gear_transform: Option<&mut Transform>,
    gear_paint_job: Option<(&mut Sprite, Option<TeamNumber>)>,
) {
    commands.entity(char_entity).add_child(gear_entity);
    commands
        .entity(gear_entity)
        .remove::<KinematicsBundle>()
        .remove::<Sensor>()
        .insert(Equipped { by: char_entity });

    if let Some(such) = gear_transform {
        reset_gun_transform(gun_preset, such);
    }
    if let Some(such) = gear_paint_job {
        team_paint_gun(gun_preset, such.0, such.1);
    }
}

/// Un-attach something equipped on some entity and give it physics.
/// No safety checks are made.
fn unequip_gear(
    commands: &mut Commands,
    gear_entity: Entity,
    gun_type: GunPreset,
    gear_sprite: &mut Sprite,
    kinematics: KinematicsBundle,
) {
    commands
        .entity(gear_entity)
        .remove::<Equipped>()
        .insert(kinematics);

    // reset_gun_transform(gun_type, gear_transform);
    team_paint_gun(gun_type, gear_sprite, None);
}

/// Unequip gear and give it some speed according to its type.
/// No safety checks are made.
fn throw_away_gear(
    commands: &mut Commands,
    char_transform: &Transform,
    gear_entity: Entity,
    gun_type: GunPreset,
    gear_transform: &mut Transform,
    gear_sprite: &mut Sprite,
    gear_given_velocity: Vec3,
) {
    let kinematics = gun_type
        .stats()
        .get_kinematics()
        .with_linear_velocity(gear_given_velocity)
        .with_angular_velocity(GUN_THROW_SPIN_SPEED)
        .with_rigidbody_type(RigidBody::Dynamic);

    unequip_gear(commands, gear_entity, gun_type, gear_sprite, kinematics);

    let gear_offset_forward = char_transform.up() * char_transform.scale.y * CHARACTER_SIZE / 2.;
    *gear_transform = Transform::from_translation(
        char_transform.translation
            + gear_offset_forward
            + char_transform.rotation * gear_transform.translation,
    )
    .with_rotation(char_transform.rotation);
}

/// System to convert a character's action input (human or not) to linear and angular velocities.
pub fn calculate_character_velocity(
    // inputs: Res<PlayerInputs<GgrsConfig>>,
    mut query: Query<(&mut Velocity, &Transform, &CharacterActionInput)>,
) {
    for (mut velocity, transform, action_input) in query.iter_mut() {
        velocity.linvel = (transform.up() * action_input.speed() * CHARACTER_SPEED).truncate();
        velocity.angvel = action_input.angular_speed() * -CHARACTER_RAD_SPEED;
    }
}

/// System to pick up and equip guns off the ground according to a character's input.
pub fn handle_gun_picking(
    mut commands: Commands,
    query_characters: Query<(&CharacterActionInput, &Team, Entity)>,
    mut query_weapons: Query<
        (
            &Gun,
            &OngoingCollisions,
            // todo:mp event for color/sprite changing
            &mut Sprite,
            &mut Transform,
            Entity,
        ),
        (With<RigidBody>, Without<Equipped>),
    >,
) {
    for (weapon, collisions, mut weapon_sprite, mut weapon_transform, weapon_entity) in
        query_weapons.iter_mut()
    {
        if collisions.is_empty() {
            continue;
        }

        for (char_input, char_team, char_entity) in query_characters.iter() {
            if !char_input.interact_1 || !collisions.contains(&char_entity) {
                continue;
            }

            let weapon_preset = weapon.preset;

            equip_gear(
                &mut commands,
                char_entity,
                weapon_entity,
                weapon_preset,
                Some(&mut weapon_transform),
                Some((&mut weapon_sprite, Some(char_team.0))),
            );
        }
    }
}

/// System to, according to either to a character's input or its untimely demise, unequip guns and throw them to the ground with some gusto.
/// That perfect gun is gone, and the heat never bothered it anyway.
pub fn handle_letting_gear_go(
    mut commands: Commands,
    mut query_characters: Query<
        (
            &CharacterActionInput,
            &Velocity,
            &Transform,
            &mut Children,
            &Health,
            Entity,
        ),
        Without<Equipped>,
    >,
    mut query_gear: Query<(&Gun, &mut Sprite, &mut Transform), With<Equipped>>,
) {
    for (action_input, velocity, transform, children, health, entity) in query_characters.iter_mut()
    {
        // Only proceed with the throwing away if either the drop-gear button is pressed, or if the guy's wasted.
        if !(action_input.interact_2 || health.is_dead()) || children.is_empty() {
            continue;
        }

        let mut equipped_gears = Vec::<Entity>::new();
        for child in children.iter() {
            let child = *child;
            if let Ok((gun, mut gun_sprite, mut gun_transform)) = query_gear.get_mut(child) {
                let gun_type = gun.preset;
                equipped_gears.push(child);
                let gun_velocity = velocity.linvel.extend(0.) + transform.up() * GUN_THROW_SPEED;
                throw_away_gear(
                    &mut commands,
                    transform,
                    child,
                    gun_type,
                    &mut gun_transform,
                    &mut gun_sprite,
                    gun_velocity,
                );
            }
        }

        commands.entity(entity).remove_children(&equipped_gears);
    }
}

/// System to distribute guns around a character's face whenever a new one is added or an old one removed.
pub fn handle_inventory_layout_change(
    query_characters: Query<
        (&Transform, &Children),
        (With<CharacterActionInput>, Changed<Children>, Without<Gun>),
    >,
    mut query_gear: Query<(&Gun, &mut Transform), With<Equipped>>,
) {
    for (char_transform, children) in query_characters.iter() {
        let step_size = (CHARACTER_SIZE / (children.len() as f32 + 1.0)) * char_transform.scale.x;
        let far_left_x = -CHARACTER_SIZE * char_transform.scale.x / 2.0;
        for (i, child) in children.iter().enumerate() {
            if let Ok((gun, mut gun_transform)) = query_gear.get_mut(*child) {
                let original_transform = gun
                    .preset
                    .stats()
                    .get_transform_with_scale(char_transform.scale);

                gun_transform.translation.x =
                    original_transform.translation.x + far_left_x + step_size * (i + 1) as f32;
            }
        }
    }
}

// dead men walking parsing (dying characters and other entities, through sparse-set components --> do a variety of laying-to-rest activities to them prior to their passing)
