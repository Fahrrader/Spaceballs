use crate::characters::{CHARACTER_RAD_SPEED, CHARACTER_SPEED};
use crate::controls::CharacterActionInput;
use crate::physics::Velocity;
use crate::teams::Team;
use crate::{EntropyGenerator, Equipped, GameState, Gun, Health};

use bevy::prelude::*;
use bevy::utils::default;
use dfdx::optim::Sgd;
use dfdx::prelude::*;
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::PI;

type Action = usize;
type Reward = f32;
type EpisodeFinished = bool;

const STATE_PARAMS: usize = CHARACTER_PARAMS * 2;
const CHARACTER_PARAMS: usize = 7 + 1/*5 * 6*/;
const ACTIONS: Action = 36;

type Network = (
    (Linear<STATE_PARAMS, 64>, ReLU),
    (Linear<64, 64>, ReLU),
    (Linear<64, 32>, ReLU),
    Linear<32, ACTIONS>,
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

const EPSILON_DECAY: f64 = 0.001;
const MIN_EPSILON: f64 = 0.01;
const EXPERIENCE_LIMIT: usize = 1_000_000;
const SYNC_INTERVAL_STEPS: usize = 300;

// const EPOCHS: usize = 32;
const BATCH_SIZE: usize = 256;
const LEARNING_RATE: f64 = 0.005;
const HUBER_THRESHOLD: f64 = 1.0;
const NEXT_STATE_DISCOUNT: f32 = 0.98;

#[derive(Debug)]
struct DQNModel {
    qn: BuiltNetwork,
    target: BuiltNetwork,
    gradients: Gradients<f32, AutoDevice>,
    optimizer: Sgd<BuiltNetwork, f32, AutoDevice>,
    device: AutoDevice,
    epsilon: f64, // exploration threshold
    training_steps: usize,
    replay_buffer: VecDeque<Experience>,
    entropy: EntropyGenerator,
    /*steps_since_last_merge: i32,
    survived_steps: i32,
    episode: i32,
    epsilon: f32,
    experience: Vec<Transition>,*/
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
                lr: 1e-1,
                momentum: Some(Momentum::Nesterov(0.9)),
                weight_decay: None,
            },
        );

        let gradients = qn.alloc_grads();
        let mut target = qn.clone();

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
            /*steps_since_last_merge: 0,
            survived_steps: 0,
            episode: 0,
            epsilon: 1.,
            experience: Vec::new(),*/
        }
    }
}

impl DQNModel {
    pub fn act(&mut self, observation: Observation) -> Action {
        let state_tensor = self
            .device
            .tensor_from_vec(observation.to_vec(), (Const::<STATE_PARAMS>,));

        // todo use EntropyGenerator for these
        let mut rng = rand::thread_rng();
        let is_exploring = rng.gen_bool(self.epsilon);

        let action = match is_exploring {
            true => rng.gen_range(0..ACTIONS - 1),
            false => {
                let q_values = self.qn.forward(state_tensor);
                let max_q_value = q_values.clone().max::<Rank0, _>();
                let maybe_action = q_values
                    .array()
                    .iter()
                    .position(|q| *q >= max_q_value.array());
                if None == maybe_action {
                    dbg!(q_values);
                    panic!();
                }
                maybe_action.unwrap()
            }
        };

        action
    }

    /// Dampen the exploration threshold
    // in this establishment, curiosity must be "satisfied", not  "decayed" :)
    pub fn satisfy_curiosity_by(&mut self, exploration_satisfaction: f64) {
        self.epsilon = (self.epsilon - exploration_satisfaction).max(MIN_EPSILON);
    }

    pub fn record_experience(&mut self, experience: Experience) {
        if self.replay_buffer.len() >= EXPERIENCE_LIMIT {
            self.replay_buffer.pop_front();
        }
        self.replay_buffer.push_back(experience);
    }

    pub fn learn(&mut self) {
        if self.replay_buffer.len() < BATCH_SIZE {
            return;
        }

        let buffer_len = self.replay_buffer.len();
        let (mut states, mut actions, mut rewards, mut next_states, mut endings) = (
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
            endings.push(!episode_finished as i32 as f32);
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
        let ending_tensor = self.device.tensor_from_vec(endings, (Const::<BATCH_SIZE>,));

        // getting the current network predicted q-values
        let q_values = self.qn.forward(states_tensor.trace(self.gradients.clone()));
        let action_qs = q_values.select(actions_tensor);

        // getting the target network predicted q-values
        let q_next_values = self.target.forward(next_states_tensor);
        let max_next_q = q_next_values.max::<Rank1<BATCH_SIZE>, _>();
        let target_q =
            (max_next_q * ending_tensor.clone()) * NEXT_STATE_DISCOUNT + rewards_tensor.clone();

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
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DetailedCharacterObservation {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub v_x: f32,
    pub v_y: f32,
    pub v_ang: f32,
    pub hp: f32,
    pub gun_id: f32,
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
            self.hp,
            self.gun_id,
        ]
    }
}

impl std::ops::Add<DetailedCharacterObservation> for DetailedCharacterObservation {
    type Output = [f32; CHARACTER_PARAMS * 2];

    fn add(self, rhs: DetailedCharacterObservation) -> Self::Output {
        let a: ObservedSingleCharacter = self.into();
        let b: ObservedSingleCharacter = rhs.into();
        let mut result = [a[0]; CHARACTER_PARAMS * 2];

        for i in 0..CHARACTER_PARAMS {
            result[i] = a[i];
        }

        for j in 0..CHARACTER_PARAMS {
            result[CHARACTER_PARAMS + j] = b[j];
        }

        result
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DetailedObservation {
    pub actor: DetailedCharacterObservation,
    pub target: DetailedCharacterObservation,
}

impl Into<Observation> for DetailedObservation {
    fn into(self) -> Observation {
        self.actor + self.target
    }
}

trait AIAction {
    fn action_to_input(self) -> CharacterActionInput;
}

impl AIAction for Action {
    fn action_to_input(self) -> CharacterActionInput {
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

#[derive(Component, Default, Debug)]
pub struct AIAgent {
    pub current_experience: Option<(DetailedObservation, Action)>,
    // pub prev_reward: f32,
    // pub peer_handle: usize,
}

impl AIAgent {
    pub fn maybe_complete_experience(
        &mut self,
        new_state: DetailedObservation,
        time_delta: f32, /*, _episode_done: bool*/
    ) -> Option<Experience> {
        if self.current_experience.is_none() {
            return None;
        }

        let (past_state, action_taken) = self.current_experience.take().unwrap();
        let (reward, episode_finished) = Self::calculate_reward(past_state, new_state, time_delta);

        Some((
            past_state.into(),
            action_taken,
            reward,
            new_state.into(),
            episode_finished,
        ))
    }

    pub fn calculate_reward(
        state_0: DetailedObservation,
        state_1: DetailedObservation,
        time_delta: f32,
    ) -> (Reward, EpisodeFinished) {
        const DISTANCE_REWARD: Reward = 1.0;
        const ROTATION_REWARD: Reward = 3.0;
        const MAINTAINING_HEALTH_REWARD: Reward = 3.0;
        const TAKING_DAMAGE_REWARD: Reward = 15.0;
        const DEATH_REWARD: Reward = -1000.0;
        const NOT_DEALING_DAMAGE_REWARD: Reward = -2.0;
        const DEALING_DAMAGE_REWARD: Reward = 25.0;
        const KILL_REWARD: Reward = 500.0;

        let mut reward = 0.;
        let mut finished_episode = false;

        let distance_0 = Vec2::new(state_0.target.x, state_0.target.y)
            - Vec2::new(state_0.actor.x, state_0.actor.y);
        let distance_1 = Vec2::new(state_1.target.x, state_1.target.y)
            - Vec2::new(state_1.actor.x, state_1.actor.y);
        let distance_change = (distance_0 - distance_1).length();
        // for closing distance
        reward += if distance_change.is_sign_positive() {
            DISTANCE_REWARD * distance_change / CHARACTER_SPEED / time_delta
        } else {
            0.
        };

        // facing the target
        let desired_rotation_0 = distance_0.y.atan2(distance_0.x);
        let desired_rotation_1 = distance_1.y.atan2(distance_1.x);
        let rotation_difference_0 = (state_0.actor.angle - desired_rotation_0).abs();
        let rotation_difference_1 = (state_1.actor.angle - desired_rotation_1).abs();
        // for staying on target
        reward += ROTATION_REWARD * (rotation_difference_0 - rotation_difference_1) / time_delta;

        // for being healthy
        reward += if state_1.actor.hp == state_0.actor.hp {
            MAINTAINING_HEALTH_REWARD
        } else {
            if state_1.actor.hp <= 0.0 && state_0.actor.hp > 0.0 {
                finished_episode = true;
                DEATH_REWARD
            } else {
                TAKING_DAMAGE_REWARD * (state_1.actor.hp - state_0.actor.hp)
            }
        };

        // for crushing your enemies
        reward += if state_1.target.hp == state_0.target.hp {
            NOT_DEALING_DAMAGE_REWARD
        } else {
            if state_1.target.hp <= 0.0 && state_0.target.hp > 0.0 {
                finished_episode = true;
                KILL_REWARD
            } else {
                DEALING_DAMAGE_REWARD * (state_1.target.hp - state_0.target.hp)
            }
        };

        (reward, finished_episode)
    }
}

fn setup_ai_network(world: &mut World) {
    world.insert_non_send_resource(DQNModel::default());
}

type CharacterQuery<'a> = (&'a Transform, &'a Velocity, &'a Health, &'a Children);

// observe world and act
fn handle_acting(
    q_characters: Query<(CharacterQuery, &Team), With<CharacterActionInput>>,
    mut q_agents: Query<(&mut AIAgent, &mut CharacterActionInput, &Team, Entity)>,
    q_gear: Query<&Gun, With<Equipped>>,
    mut model: NonSendMut<DQNModel>,
    time: Res<Time>,
) {
    let extract_character_state = |character: CharacterQuery| -> DetailedCharacterObservation {
        let (transform, velocity, health, children) = character;
        let mut gun_id = 0.0;
        for child in children.iter() {
            if let Ok(gun) = q_gear.get(*child) {
                gun_id = gun.preset.stats().id as f32;
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
            hp: health.hp(),
            // shooting, - fire cooldown or reload progress?
            gun_id,
        }
    };

    for (mut agent, mut action_input, actor_team, actor_entity) in q_agents.iter_mut() {
        let actor_observation = extract_character_state(
            q_characters
                .get(actor_entity)
                .expect("Could not get the actor information, agent not in the character query")
                .0,
        );

        // todo away, away with ye heathen! but still, serve me well prior to thy replacement with proper environment observation
        let mut target_observation = DetailedCharacterObservation::default();
        for (character_state, character_team) in q_characters.iter() {
            if character_team != actor_team {
                target_observation = extract_character_state(character_state);
                break;
            }
        }

        let observation = DetailedObservation {
            actor: actor_observation,
            target: target_observation,
        };

        if let Some(past_experience) =
            agent.maybe_complete_experience(observation, time.delta_seconds())
        {
            model.record_experience(past_experience);
        }

        let action = model.act(observation.into());
        *action_input = action.action_to_input();
        agent.current_experience = Some((observation, action));
    }
}

fn handle_learning(mut model: NonSendMut<DQNModel>) {
    model.learn();
}

pub struct AIPlugin;
impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
            //.init_resource::<Model>()
            .add_startup_system(setup_ai_network)
            .add_system(handle_acting)
            .add_system(handle_learning.run_if(in_state(GameState::InGame)))
            // action routine gets abnormally long if in rollback together with ai input, might be interesting to look into
            /*.add_system(
                handle_ai_input
                    .run_if(in_state(GameState::InGame))
                    .in_set(InputHandlingSet::InputReading),
            )*/;
    }
}
