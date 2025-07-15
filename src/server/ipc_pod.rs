use crate::physics::object_pod::WorldStatePod;
use crate::physics::world::World;
use crate::server::models_pod::{MessagePod, CommandPod, ActionPod};
use std::error::Error;
use zmq::{Context, Socket};

pub struct ZeroCopyIPC {
    socket: Socket,
}

impl ZeroCopyIPC {
    pub fn new(ctx: &Context, addr: &str) -> Result<Self, Box<dyn Error>> {
        let socket = ctx.socket(zmq::REP)?;
        socket.bind(addr)?;
        Ok(Self { socket })
    }

    pub fn recv_message(&self) -> Result<MessagePod, Box<dyn Error>> {
        let msg = self.socket.recv_bytes(0)?;
        if msg.len() != std::mem::size_of::<MessagePod>() {
            return Err("Invalid message size".into());
        }
        Ok(*bytemuck::from_bytes::<MessagePod>(&msg))
    }

    pub fn send_bytes(&self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        self.socket.send(bytes, 0)?;
        Ok(())
    }

    pub fn send_world_state(&self, world: &World) -> Result<(), Box<dyn Error>> {
        let mut state = WorldStatePod::zeroed();
        state.tick_rate = world.tick_rate;
        state.body_count = world.bodies.len() as u32;
        
        // Copy bodies up to MAX_BODIES
        let count = world.bodies.len().min(state.bodies.len());
        for (i, body) in world.bodies.iter().take(count).enumerate() {
            state.bodies[i] = body.into();
        }

        let msg = MessagePod::from_world_state(state);
        self.send_bytes(bytemuck::bytes_of(&msg))?;
        Ok(())
    }

    pub fn handle_request(&self, world: &mut World) -> Result<(), Box<dyn Error>> {
        let msg = self.recv_message()?;
        
        match msg.msg_type {
            MessagePod::TYPE_COMMAND => {
                let cmd = msg.as_command();
                match cmd.cmd_type {
                    CommandPod::GET_STATE => {
                        self.send_world_state(world)?;
                    }
                    CommandPod::STEP => {
                        world.tick();
                        self.send_world_state(world)?;
                    }
                    CommandPod::RESET => {
                        // Reset world to initial state
                        *world = World::new_empty();
                        self.send_world_state(world)?;
                    }
                    _ => {
                        return Err("Unknown command".into());
                    }
                }
            }
            MessagePod::TYPE_ACTION => {
                let action = msg.as_action();
                // Apply action to world
                if let Some(body) = world.get_body_mut(action.body_id.as_str()) {
                    body.velocity = action.velocity.into();
                    body.position = action.position.into();
                    body.mass = action.mass;
                    body.restitution = action.restitution;
                    body.dynamic = action.dynamic != 0;
                    body.compute_aabb();
                }
                self.send_world_state(world)?;
            }
            _ => {
                return Err("Unknown message type".into());
            }
        }
        Ok(())
    }
}