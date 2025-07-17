use crate::game::game_engine::GameState;
use crate::physics::{Vector3, World};
use crate::server::ipc::IPCChannel;
use crate::server::models::{Action, WorldState};
use std::time::{Duration, Instant};

pub struct Engine {
    pub action_channel: IPCChannel, // PULL for receiving actions
    pub state_channel: IPCChannel,  // PUB for broadcasting state
    pub world: World,
    pub game_state: GameState,
    pub start_time: Instant,
}

impl Engine {
    // Updated constructor to create two channels
    pub fn new_server(
        action_endpoint: &str,
        state_endpoint: &str,
        world: World,
    ) -> Result<Self, zmq::Error> {
        let action_channel = IPCChannel::new_pull(action_endpoint)?;
        let state_channel = IPCChannel::new_pub(state_endpoint)?;
        let start_time = Instant::now();

        // Find paddle and ball indices
        let paddle1_index = world
            .bodies
            .iter()
            .position(|b| b.id == "paddle1")
            .unwrap_or(0);
        let paddle2_index = world
            .bodies
            .iter()
            .position(|b| b.id == "paddle2")
            .unwrap_or(1);
        let ball_index = world
            .bodies
            .iter()
            .position(|b| b.id == "ball")
            .unwrap_or(2);

        let game_state = GameState::new(paddle1_index, paddle2_index, ball_index);

        Ok(Self {
            action_channel,
            state_channel,
            world,
            game_state,
            start_time,
        })
    }

    pub fn step(&mut self, dt: f64) {
        self.world.step(dt);

        // Constrain paddles within arena bounds
        // self.constrain_paddles();

        // Check for scoring
        let ball = &self.world.bodies[self.game_state.ball_index];
        if ball.position.x < -30.0 {
            self.game_state.score_player2 += 1;
            self.reset_ball(2);
        } else if ball.position.x > 30.0 {
            self.game_state.score_player1 += 1;
            self.reset_ball(1);
        }
    }

    fn constrain_paddles(&mut self) {
        let arena_half_height = 20.0;
        let arena_half_depth = 20.0;

        // Constrain paddle1
        let paddle1 = &mut self.world.bodies[self.game_state.paddle1_index];
        let paddle1_half = paddle1.get_half_extents();

        if paddle1.position.y - paddle1_half.y < -arena_half_height {
            paddle1.position.y = -arena_half_height + paddle1_half.y;
            paddle1.velocity.y = 0.0;
        } else if paddle1.position.y + paddle1_half.y > arena_half_height {
            paddle1.position.y = arena_half_height - paddle1_half.y;
            paddle1.velocity.y = 0.0;
        }

        if paddle1.position.z - paddle1_half.z < -arena_half_depth {
            paddle1.position.z = -arena_half_depth + paddle1_half.z;
            paddle1.velocity.z = 0.0;
        } else if paddle1.position.z + paddle1_half.z > arena_half_depth {
            paddle1.position.z = arena_half_depth - paddle1_half.z;
            paddle1.velocity.z = 0.0;
        }

        // Constrain paddle2
        let paddle2 = &mut self.world.bodies[self.game_state.paddle2_index];
        let paddle2_half = paddle2.get_half_extents();

        if paddle2.position.y - paddle2_half.y < -arena_half_height {
            paddle2.position.y = -arena_half_height + paddle2_half.y;
            paddle2.velocity.y = 0.0;
        } else if paddle2.position.y + paddle2_half.y > arena_half_height {
            paddle2.position.y = arena_half_height - paddle2_half.y;
            paddle2.velocity.y = 0.0;
        }

        if paddle2.position.z - paddle2_half.z < -arena_half_depth {
            paddle2.position.z = -arena_half_depth + paddle2_half.z;
            paddle2.velocity.z = 0.0;
        } else if paddle2.position.z + paddle2_half.z > arena_half_depth {
            paddle2.position.z = arena_half_depth - paddle2_half.z;
            paddle2.velocity.z = 0.0;
        }
    }

    fn reset_ball(&mut self, scorer: u8) {
        let ball = &mut self.world.bodies[self.game_state.ball_index];
        ball.position = Vector3::new(0.0, 0.0, 0.0);
        ball.velocity = if scorer == 1 {
            Vector3::new(-8.0, 4.0, 0.0)
        } else {
            Vector3::new(8.0, 4.0, 0.0)
        };
    }

    pub fn get_state(&self) -> WorldState {
        WorldState {
            bodies: self.world.bodies.clone(),
            time: self.start_time.elapsed().as_secs_f32(),
            score_player1: self.game_state.score_player1,
            score_player2: self.game_state.score_player2,
        }
    }

    pub fn post_action(&mut self, action: Action) {
        let body_id = action.body_id;
        let body = self.world.get_body_mut(&body_id);
        if let Some(body) = body {
            let position_changed = body.position.x != action.position.x
                || body.position.y != action.position.y
                || body.position.z != action.position.z;
            let velocity_changed = body.velocity.x != action.velocity.x
                || body.velocity.y != action.velocity.y
                || body.velocity.z != action.velocity.z;

            if position_changed {
                body.update_position(action.position.x, action.position.y, action.position.z);
            }
            if velocity_changed {
                body.update_velocity(action.velocity.x, action.velocity.y, action.velocity.z);
            }
            if body.mass != action.mass {
                body.mass = action.mass;
            }
            if body.restitution != action.restitution {
                body.restitution = action.restitution;
            }
            if body.dynamic != action.dynamic {
                body.dynamic = action.dynamic;
            }
        }
    }

    pub fn reset(&mut self, state: WorldState) {
        self.world = World::new(self.world.id.clone(), self.world.tick_rate, state.bodies);
        self.start_time = Instant::now();
        self.game_state.score_player1 = state.score_player1;
        self.game_state.score_player2 = state.score_player2;
    }

    // Updated run method with PUSH/PULL + PUB/SUB
    pub fn run(&mut self) -> Result<(), zmq::Error> {
        let mut last_time = Instant::now();
        let mut last_state_send = Instant::now();
        let state_interval = Duration::from_millis(16); // ~60Hz
        let mut accumulator = 0.0;
        let fixed_timestep = 1.0 / 120.0; // 120Hz physics

        loop {
            // Calculate delta time
            let now = Instant::now();
            let frame_time = (now - last_time).as_secs_f64();
            last_time = now;
            accumulator += frame_time;

            // Process ALL pending actions (drain the queue)
            while let Ok(bytes) = self.action_channel.recv_bytes_nonblocking() {
                if let Ok(action) = Action::from_bytes(&bytes) {
                    self.post_action(action);
                }
            }

            // Fixed timestep physics updates
            while accumulator >= fixed_timestep {
                self.step(fixed_timestep);
                accumulator -= fixed_timestep;
            }

            // Broadcast state periodically
            if last_state_send.elapsed() >= state_interval {
                let world_state = self.get_state();
                if let Ok(response) = world_state.to_bytes() {
                    let _ = self.state_channel.send_bytes(&response);
                }
                last_state_send = Instant::now();
            }

            // Small sleep to avoid 100% CPU
            std::thread::sleep(Duration::from_millis(3));
        }
    }
}
