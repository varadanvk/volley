use crate::physics::World;
use crate::server::ipc::IPCChannel;
use crate::server::models::{Action, Command, WorldState};
use std::time::Instant;

pub struct Engine {
    pub ipc_channel: IPCChannel,
    pub world: World,
    pub state: WorldState,
    pub start_time: Instant,
}
impl Engine {
    pub fn new(ipc_channel: IPCChannel, world: World) -> Self {
        let start_time = Instant::now();
        let state = WorldState {
            bodies: world.bodies.clone(),
            time: 0.0,
        };
        Self {
            ipc_channel,
            world,
            state,
            start_time,
        }
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
            // Check if any changes are needed before applying them
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

    pub fn run(&mut self) {
        loop {}
    }
}
