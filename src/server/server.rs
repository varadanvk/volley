use crate::physics::World;
use crate::server::ipc::IPCChannel;
use crate::server::models::{Action, Command, WorldState};
use std::time::{Duration, Instant};

pub struct Engine {
    pub action_channel: IPCChannel, // PULL for receiving actions
    pub state_channel: IPCChannel,  // PUB for broadcasting state
    pub world: World,
    pub state: WorldState,
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
        let state = WorldState {
            bodies: world.bodies.clone(),
            time: 0.0,
        };
        Ok(Self {
            action_channel,
            state_channel,
            world,
            state,
            start_time,
        })
    }

    pub fn step(&mut self, dt: f64) {
        self.world.step(dt);
        self.state.time = self.start_time.elapsed().as_secs_f32();
    }

    pub fn get_state(&self) -> WorldState {
        WorldState {
            bodies: self.world.bodies.clone(),
            time: self.start_time.elapsed().as_secs_f32(),
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
        self.state.time = 0.0;
    }

    // Updated run method with PUSH/PULL + PUB/SUB
    pub fn run(&mut self) -> Result<(), zmq::Error> {
        let mut last_state_send = Instant::now();
        let state_interval = Duration::from_millis(16); // ~60Hz

        loop {
            // Non-blocking check for actions
            if let Ok(bytes) = self.action_channel.recv_bytes_nonblocking() {
                if let Ok(command) = Command::from_bytes(&bytes) {
                    match command {
                        Command::GetState => {
                            let world_state = self.get_state();
                            if let Ok(response) = world_state.to_bytes() {
                                self.state_channel.send_bytes(&response)?;
                            }
                        }
                        Command::PostAction => {
                            if let Ok(action) = Action::from_bytes(&bytes) {
                                self.post_action(action.clone());
                                println!("Received action: {:?}", action);
                            }
                        }
                        Command::Reset => {
                            if let Ok(state) = WorldState::from_bytes(&bytes) {
                                self.reset(state.clone());
                                println!("Received reset: {:?}", state);
                            }
                        }
                        Command::Step => {
                            self.step(0.016); // Fixed step
                            println!("Received step");
                        }
                    }
                }
            }

            // Broadcast state periodically
            if last_state_send.elapsed() >= state_interval {
                let world_state = self.get_state();
                if let Ok(response) = world_state.to_bytes() {
                    self.state_channel.send_bytes(&response)?;
                }
                last_state_send = Instant::now();
            }

            // Small sleep to avoid 100% CPU
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
