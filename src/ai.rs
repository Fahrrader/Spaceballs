use crate::characters::{AICharacterBundle, BuildCharacter, ControllerHandle, CHARACTER_SPEED};
use crate::controls::CharacterActionInput;
use crate::physics::Velocity;
use crate::teams::Team;
use crate::{
    EntropyGenerator, Equipped, GameState, Gun, GunPreset, Health, InputHandlingSet, PlayerDied,
    AI_DEFAULT_TEAM, CHUNK_SIZE, SCREEN_SPAN,
};

use crate::network::players::PlayerData;
use crate::network::session::LocalPlayer;
use crate::network::{PlayerHandle, PlayerRegistry};
use bevy::prelude::*;
use bevy::utils::default;
use dfdx::optim::Sgd;
use dfdx::prelude::*;
use rand::Rng;
use std::collections::VecDeque;
use std::time::Duration;

type Action = usize;
type Reward = f32;
type EpisodeFinished = bool;

const STATE_PARAMS: usize = CHARACTER_PARAMS * 2;
const CHARACTER_PARAMS: usize = 11/* + 5 * 6*/;
const ACTIONS: Action = 36;

type Network = (
    (Linear<STATE_PARAMS, 64>, ReLU),
    (Linear<64, 256>, ReLU),
    (Linear<256, 1024>, ReLU),
    (Linear<1024, 512>, ReLU),
    (Linear<512, 128>, ReLU),
    Linear<128, ACTIONS>,
);

type BuiltNetwork = <Network as BuildOnDevice<AutoDevice, f32>>::Built;

type Experience = (
    Observation,
    Action,
    Reward,
    /*Option<*/ Observation,
    EpisodeFinished,
);

type Observation = [f32; STATE_PARAMS];
type ObservedSingleCharacter = [f32; CHARACTER_PARAMS];

const EPSILON_DECAY: f64 = 0.000001;
const MIN_EPSILON: f64 = 0.01;
const EXPERIENCE_LIMIT: usize = 1_000_000;
const SYNC_INTERVAL_STEPS: usize = 300;

// const EPOCHS: usize = 32;
const BATCH_SIZE: usize = 256;
const LEARNING_RATE: f64 = 0.02;
const HUBER_THRESHOLD: f64 = 1.0;
const NEXT_STATE_DISCOUNT: f32 = 0.995;

#[derive(Debug)]
struct DQNModel {
    qn: BuiltNetwork,
    target: BuiltNetwork,
    gradients: Gradients<f32, AutoDevice>,
    optimizer: Sgd<BuiltNetwork, f32, AutoDevice>,
    device: AutoDevice,
    // strategy: Greedy, EpsilonGreedy, EpsilonSoftmax,
    epsilon: f64, // exploration threshold
    training_steps: usize,
    // episode: usize,
    replay_buffer: VecDeque<Experience>,
    entropy: EntropyGenerator,
}

impl Default for DQNModel {
    fn default() -> Self {
        let dev = AutoDevice::default();
        let mut qn: BuiltNetwork = dev.build_module::<Network, f32>();
        qn.reset_params();

        // let mut grads = q_net.alloc_grads();

        let optimizer = Sgd::new(
            &qn,
            SgdConfig {
                lr: LEARNING_RATE,
                momentum: Some(Momentum::Nesterov(0.9)),
                weight_decay: None,
            },
        );

        let gradients = qn.alloc_grads();
        let target = qn.clone();

        Self {
            qn,
            target,
            gradients,
            optimizer,
            device: dev,
            epsilon: 1.,
            training_steps: 0,
            replay_buffer: VecDeque::with_capacity(EXPERIENCE_LIMIT),
            entropy: EntropyGenerator::new(0),
        }
    }
}

impl DQNModel {
    pub fn act(&mut self, observation: Observation) -> Action {
        let state_tensor = self
            .device
            .tensor_from_vec(observation.to_vec(), (Const::<STATE_PARAMS>,));

        let is_exploring = self.entropy.gen_bool(self.epsilon);

        let action = match is_exploring {
            true => self.pick_random_action(),
            false => {
                let q_values = self.qn.forward(state_tensor);
                let (action, is_poisoned) = self.pick_softmax_action(&q_values.array(), 1.0);
                if is_poisoned {
                    // might as well abandon the network at this point
                    dbg!(q_values);
                    return self.pick_random_action();
                }
                action
            }
        };

        action
    }

    fn pick_random_action(&mut self) -> Action {
        self.entropy.gen_range(0..ACTIONS)
    }

    fn pick_greedy_action(&mut self, q_values: &[f32]) -> (Action, bool) {
        let mut is_poisoned = false;
        let (best_action, _) = q_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.partial_cmp(b).unwrap_or_else(|| {
                    is_poisoned = true;
                    if b.is_nan() {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                })
            })
            .unwrap();

        (best_action, is_poisoned)
    }

    fn pick_softmax_action(&mut self, q_values: &[f32], _temperature: f32) -> (Action, bool) {
        let (max_q_action, is_poisoned) = self.pick_greedy_action(q_values);
        if is_poisoned {
            return (0, is_poisoned);
        }

        let max_q = q_values[max_q_action];
        let exp: Vec<_> = q_values
            .iter()
            .map(|&q| (/*(*/q - max_q/*) / temperature*/).exp())
            .collect();
        let sum_exp: f32 = exp.iter().sum();
        let probabilities: Vec<_> = exp.iter().map(|&num| num / sum_exp).collect();

        let random_threshold: f32 = self.entropy.gen();
        let mut sum = 0.0;
        for (i, &p) in probabilities.iter().enumerate() {
            sum += p;
            if random_threshold <= sum {
                return (i, false);
            }
        }
        (probabilities.len() - 1, false)
    }

    #[cfg(feature = "safetensors")]
    const FILENAME: &'static str = "spaceball.safetensors";

    #[cfg(feature = "numpy")]
    const FILENAME: &'static str = "spaceball.npy";

    #[cfg(feature = "safetensors")]
    pub fn save(&self) {
        #[cfg(feature = "safetensors")]
        self.target
            .save_safetensors(DQNModel::FILENAME)
            .expect("Failed to save the model");
        #[cfg(feature = "numpy")]
        self.target
            .save_to_npy(DQNModel::FILENAME)
            .expect("Failed to save the model");
    }

    #[cfg(any(feature = "safetensors", feature = "numpy"))]
    pub fn load(&mut self) {
        #[cfg(feature = "numpy")]
        let loaded = self.qn.load_from_npy(DQNModel::FILENAME).is_ok();
        #[cfg(feature = "safetensors")]
        let loaded = self.qn.load_safetensors(DQNModel::FILENAME).is_ok();

        if loaded {
            self.target = self.qn.clone();
            self.epsilon = 0.5_f64.max(MIN_EPSILON);
        } else {
            error!(
                "FAILED TO LOAD MODEL FROM {}. Starting from scratch.",
                DQNModel::FILENAME
            );
        }
    }

    pub fn record_experience(&mut self, experience: Experience) {
        if experience.2 < -300.0 {
            bevy::log::error!("I hurt myself today; {}", experience.2);
        }
        if self.replay_buffer.len() >= EXPERIENCE_LIMIT {
            self.replay_buffer.pop_front();
        }
        self.replay_buffer.push_back(experience);
    }

    pub fn learn(&mut self) {
        if self.replay_buffer.len() < BATCH_SIZE * 10 {
            return;
        }

        let buffer_len = self.replay_buffer.len();
        let (mut states, mut actions, mut rewards, mut next_states, mut have_next_states) = (
            Vec::with_capacity(BATCH_SIZE * STATE_PARAMS),
            Vec::with_capacity(BATCH_SIZE),
            Vec::with_capacity(BATCH_SIZE),
            Vec::with_capacity(BATCH_SIZE * STATE_PARAMS),
            Vec::with_capacity(BATCH_SIZE),
        );
        for _ in 0..BATCH_SIZE {
            let (state, action, reward, next_state, episode_finished) =
                self.replay_buffer[self.entropy.gen_range(0..buffer_len)];
            actions.push(action);
            rewards.push(reward);
            have_next_states.push(!episode_finished as i32 as f32);
            for j in 0..STATE_PARAMS {
                states.push(state[j]);
                next_states.push(next_state[j]);
            }
        }
        let states_tensor = self
            .device
            .tensor_from_vec(states, (Const::<BATCH_SIZE>, Const::<STATE_PARAMS>));
        let next_states_tensor = self
            .device
            .tensor_from_vec(next_states, (Const::<BATCH_SIZE>, Const::<STATE_PARAMS>));
        let actions_tensor = self.device.tensor_from_vec(actions, (Const::<BATCH_SIZE>,));
        let rewards_tensor = self.device.tensor_from_vec(rewards, (Const::<BATCH_SIZE>,));
        let has_next_state_tensor = self
            .device
            .tensor_from_vec(have_next_states, (Const::<BATCH_SIZE>,));

        // getting the current network predicted q-values
        let q_values = self.qn.forward(states_tensor.trace(self.gradients.clone()));
        let action_qs = q_values.select(actions_tensor);

        // getting the target network predicted q-values
        let q_next_values = self.target.forward(next_states_tensor);
        let max_next_q = q_next_values.max::<Rank1<BATCH_SIZE>, _>();
        let target_q = (max_next_q * has_next_state_tensor.clone()) * NEXT_STATE_DISCOUNT
            + rewards_tensor.clone();

        // compute loss and back-propagate
        let loss = huber_loss(action_qs, target_q, HUBER_THRESHOLD);
        let loss_v = loss.array();
        self.optimizer
            .update(&mut self.qn, &loss.backward())
            .expect("Dang it!");

        if self.training_steps % SYNC_INTERVAL_STEPS == 0 {
            dbg!(loss_v);
            self.target = self.qn.clone();
        }

        self.training_steps += 1;
        self.satisfy_curiosity_by(EPSILON_DECAY);
    }

    /// Dampen the exploration threshold
    // in this establishment, curiosity must be "satisfied", not  "decayed" :)
    pub fn satisfy_curiosity_by(&mut self, exploration_satisfaction: f64) {
        self.epsilon = (self.epsilon - exploration_satisfaction).max(MIN_EPSILON);
    }
}

#[derive(Component, Default, Debug)]
pub struct AIAgent {
    pub current_experience: Option<(DetailedObservation, Action)>,
    pub cumulative_reward: (Reward, Reward, Reward, Reward),
    // pub peer_handle: usize,
}

impl AIAgent {
    pub fn maybe_complete_experience(
        &mut self,
        new_state: &DetailedObservation,
        time_delta: f32,
    ) -> Option<Experience> {
        if self.current_experience.is_none() {
            return None;
        }

        let (past_state, action_taken) = self.current_experience.take().unwrap();
        let (reward, episode_finished) = self.calculate_reward(&past_state, new_state, time_delta);

        Some((
            past_state.into(),
            action_taken,
            reward,
            new_state.clone().into(),
            episode_finished,
        ))
    }

    pub fn take_cumulative_reward(&mut self) -> (Reward, Reward, Reward, Reward) {
        let cum = self.cumulative_reward;
        self.cumulative_reward = (0., 0., 0., 0.);
        cum
    }

    pub fn calculate_reward(
        &mut self, // todo delete
        state_0: &DetailedObservation,
        state_1: &DetailedObservation,
        time_delta: f32,
    ) -> (Reward, EpisodeFinished) {
        const DISTANCE_REWARD: Reward = 100.0;
        // const ROTATION_REWARD: Reward = 10.0;

        // const MAINTAINING_HEALTH_REWARD: Reward = 3.0;
        const TAKING_DAMAGE_REWARD: Reward = -2.0;
        const PER_DAMAGE_TAKEN_REWARD: Reward = -0.3;
        const DEATH_REWARD: Reward = -500.0;

        const NOT_DEALING_DAMAGE_REWARD: Reward = -1.0;
        const DEALING_DAMAGE_REWARD: Reward = 3.0;
        const PER_DAMAGE_DEALT_REWARD: Reward = 0.1;
        // todo prioritize picking experiences that lead to these high absolute reward states?
        const KILL_REWARD: Reward = 3000.0;

        let mut finished_episode = false;

        let distance_0 = Vec2::new(state_0.target.x, state_0.target.y)
            - Vec2::new(state_0.actor.x, state_0.actor.y);
        let distance_1 = Vec2::new(state_1.target.x, state_1.target.y)
            - Vec2::new(state_1.actor.x, state_1.actor.y);
        let distance_change = distance_0.length() - distance_1.length();
        // for closing distance
        let reward_0 = //if distance_change.is_sign_positive() {
            DISTANCE_REWARD * distance_change / CHARACTER_SPEED // / time_delta
            /*} else {
                0.
            }*/;

        // facing the target
        /*let desired_rotation_0 = distance_0.y.atan2(distance_0.x);
        let desired_rotation_1 = distance_1.y.atan2(distance_1.x);
        let rotation_difference_0 = (state_0.actor.angle - desired_rotation_0).abs();
        let rotation_difference_1 = (state_1.actor.angle - desired_rotation_1).abs();*/
        // for staying on target
        let reward_1 = 0.0; // ROTATION_REWARD * (rotation_difference_0 - rotation_difference_1)
                            // / CHARACTER_RAD_SPEED;
                            // / time_delta;

        // for being healthy
        let reward_2 = if state_1.actor.hp == state_0.actor.hp {
            0.0 // MAINTAINING_HEALTH_REWARD * time_delta
        } else {
            if state_1.actor.hp.is_dead() && !state_0.actor.hp.is_dead() {
                finished_episode = true;
                DEATH_REWARD
            } else {
                TAKING_DAMAGE_REWARD
                    + PER_DAMAGE_TAKEN_REWARD * (*state_1.actor.hp - *state_0.actor.hp)
            }
        };

        // for crushing your enemies
        let reward_3 = KILL_REWARD * state_1.actor_killings.len() as f32
            + if state_1.target.hp == state_0.target.hp {
                NOT_DEALING_DAMAGE_REWARD * time_delta
            } else {
                /* todo source of damage is unclear; record on characters who hurt them?
                DEALING_DAMAGE_REWARD
                    +*/
                PER_DAMAGE_DEALT_REWARD * (*state_1.target.hp - *state_0.target.hp)
            };

        let cum = self.take_cumulative_reward();
        self.cumulative_reward = (
            cum.0 + reward_0,
            cum.1 + reward_1,
            cum.2 + reward_2,
            cum.3 + reward_3,
        );
        // dbg!(rewards);
        let reward = reward_0 + reward_1 + reward_2 + reward_3;

        (reward, finished_episode)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DetailedCharacterObservation {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub v_x: f32,
    pub v_y: f32,
    pub v_ang: f32,
    pub hp: Health,
    pub gun_id: f32,
    pub bullets_in_magazine: f32,
    pub reload_time: f32,
    pub can_hurt_self: f32,
    // pub time_delta: f32,
}

impl Into<ObservedSingleCharacter> for DetailedCharacterObservation {
    fn into(self) -> ObservedSingleCharacter {
        [
            self.x,
            self.y,
            self.angle,
            self.v_x,
            self.v_y,
            self.v_ang,
            self.hp.hp(),
            self.gun_id,
            self.bullets_in_magazine,
            self.reload_time,
            self.can_hurt_self,
        ]
    }
}

#[derive(Default, Debug, Clone)]
pub struct DetailedObservation {
    pub actor: DetailedCharacterObservation,
    pub target: DetailedCharacterObservation,
    pub actor_killings: Vec<PlayerHandle>,
}

impl Into<Observation> for DetailedObservation {
    fn into(self) -> Observation {
        let actor: ObservedSingleCharacter = self.actor.into();
        let target: ObservedSingleCharacter = self.target.into();
        let mut result = [0.0; CHARACTER_PARAMS * 2];

        for i in 0..CHARACTER_PARAMS {
            result[i] = actor[i];
        }

        for j in 0..CHARACTER_PARAMS {
            result[CHARACTER_PARAMS + j] = target[j];
        }

        result
    }
}

trait AIAction {
    fn action_to_input(self) -> CharacterActionInput;
}

impl AIAction for Action {
    fn action_to_input(self) -> CharacterActionInput {
        // todo rearrange when resetting the model
        let fire = self % 2 == 0;
        let up = (self / 2) % 3;
        let right = (self / 6) % 3;
        let reload = (self / 18) % 2 == 0;

        CharacterActionInput {
            up: up as f32 - 1.0,
            right: right as f32 - 1.0,
            fire,
            reload,
            ..default()
        }
    }
}

type CharacterQuery<'a> = (
    &'a Transform,
    &'a Velocity,
    &'a Health,
    // Option<&'a Dying>,
    &'a Children,
);

// observe world and act
fn handle_acting(
    q_characters: Query<(CharacterQuery, &ControllerHandle, &Team), With<CharacterActionInput>>,
    mut q_agents: Query<(&mut AIAgent, &mut CharacterActionInput, Entity)>,
    q_gear: Query<&Gun, With<Equipped>>,
    mut dead_reader: EventReader<PlayerDied>,
    mut model: NonSendMut<DQNModel>,
    time: Res<Time>,
) {
    let extract_character_state = |character: CharacterQuery| -> DetailedCharacterObservation {
        let (transform, velocity, health, children) = character;
        let mut gun = &Gun::default();
        for child in children.iter() {
            if let Ok(equipped_gun) = q_gear.get(*child) {
                gun = equipped_gun;
                // let gun_ho = gun.fire_cooldown.duration().as_secs_f32() - gun.fire_cooldown.elapsed_secs();
                break;
            }
        }

        DetailedCharacterObservation {
            x: transform.translation.x,
            y: transform.translation.y,
            angle: transform.rotation.z,
            v_x: velocity.linvel.x,
            v_y: velocity.linvel.y,
            v_ang: velocity.angvel,
            hp: *health,
            gun_id: gun.preset.stats().id as f32,
            can_hurt_self: gun.preset.stats().friendly_fire as i32 as f32,
            bullets_in_magazine: gun.shots_before_reload as f32,
            reload_time: gun.reload_progress.remaining_secs(),
        }
    };

    for (mut agent, mut action_input, actor_entity) in q_agents.iter_mut() {
        let (actor_state, actor_handle, actor_team) = q_characters
            .get(actor_entity)
            .expect("Could not get the actor information, agent not in the character query");
        let actor_observation = extract_character_state(actor_state);

        // todo away, away with ye heathen! but still, serve me well prior to thy replacement with proper environment observation
        let mut target_observation = DetailedCharacterObservation::default();
        for (character_state, _handle, character_team) in q_characters.iter() {
            if character_team != actor_team {
                target_observation = extract_character_state(character_state);
                break;
            }
        }

        let mut observation = DetailedObservation {
            actor: actor_observation,
            target: target_observation,
            actor_killings: Vec::new(),
        };

        for event in dead_reader.iter() {
            if let Some(killer_handle) = event.killed_by {
                if killer_handle == actor_handle.0 && event.player_handle != actor_handle.0 {
                    observation.actor_killings.push(event.player_handle);
                }
            }
        }

        //dbg!(observation);

        if let Some(past_experience) =
            agent.maybe_complete_experience(&observation, time.delta_seconds())
        {
            model.record_experience(past_experience);
        }

        let action = model.act(observation.clone().into());
        // dbg!(action);
        *action_input = action.action_to_input();
        agent.current_experience = Some((observation, action));
    }
}

fn setup_ai_network(world: &mut World) {
    let mut _model = DQNModel::default();
    #[cfg(any(feature = "safetensors", feature = "numpy"))]
    _model.load();
    world.insert_non_send_resource(_model);
}

fn handle_spawning(mut player_registry: ResMut<PlayerRegistry>) {
    // player_registry.0.push(PlayerData::from_player_handle(0));
    player_registry.0.push(
        PlayerData::default()
            .with_team(AI_DEFAULT_TEAM - 1)
            .with_name(String::from("Johnny")),
    );
}

fn handle_learning(mut model: NonSendMut<DQNModel>) {
    model.learn();
    #[cfg(any(feature = "safetensors", feature = "numpy"))]
    if model.training_steps != 0 && model.training_steps % SYNC_INTERVAL_STEPS == 0 {
        model.save();
        info!("Saved the model to {}.", DQNModel::FILENAME);
    }
}

struct EpisodeTimer(pub Timer);
impl Default for EpisodeTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(30), TimerMode::Repeating))
    }
}

fn handle_resetting_episode(
    mut commands: Commands,
    q_agents: Query<(&AIAgent, Option<&ControllerHandle>, Entity)>,
    //mut rejoin_teller: EventWriter<PlayerJoined>,
    time: Res<Time>,
    mut entropy: ResMut<EntropyGenerator>,
    mut episode_timer: Local<EpisodeTimer>,
) {
    if !episode_timer.0.tick(time.delta()).finished() {
        return;
    }
    for (agent, _maybe_controller, entity) in q_agents.iter() {
        // maybe complete their experiences and feed them to the network? nah, probably not
        dbg!(agent.cumulative_reward);
        commands.entity(entity).despawn_recursive();
    }

    // rejoin_teller.send(PlayerJoined { player_handle: 0 });
    // rejoin_teller.send(PlayerJoined { player_handle: 1 });

    // spawn them independently of the spawn points
    let mut gen_coordinate = |consider_walls: bool| -> f32 {
        (entropy.gen::<f32>() - 0.5) * (SCREEN_SPAN - if consider_walls { CHUNK_SIZE } else { 0.0 })
    };
    let translation_0 = Vec3::new(gen_coordinate(false), gen_coordinate(false), 0.0);
    let translation_1 = Vec3::new(gen_coordinate(false), gen_coordinate(false), 0.0);
    let transform_0 = Transform::from_translation(translation_0).looking_at(
        Vec3::new(gen_coordinate(true), gen_coordinate(true), 0.0),
        translation_0,
    );
    let transform_1 = Transform::from_translation(translation_1).looking_at(
        Vec3::new(gen_coordinate(true), gen_coordinate(true), 0.0),
        translation_1,
    );

    // agent 0
    let agent_0 = AICharacterBundle::new(transform_0, *Team::from_player_handle(0), 0)
        .spawn_with_equipment(
            &mut commands,
            entropy.fork(),
            vec![GunPreset::random(&mut entropy.0)],
        )[0];
    commands.entity(agent_0).insert(LocalPlayer);

    // agent 1
    AICharacterBundle::new(transform_1, AI_DEFAULT_TEAM - 1, 1).spawn_with_equipment(
        &mut commands,
        entropy.fork(),
        vec![GunPreset::random(&mut entropy.0)],
    )[0];
}

pub struct AIPlugin;
impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ai_network)
            .add_system(handle_spawning.in_schedule(OnEnter(GameState::InGame)))
            .add_system(handle_acting.in_set(InputHandlingSet::InputReading))
            .add_system(handle_learning.run_if(in_state(GameState::InGame)))
            .add_system(handle_resetting_episode.run_if(in_state(GameState::InGame)));
    }
}
