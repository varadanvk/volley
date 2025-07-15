use crate::physics::object_pod::{RigidBodyPod, Vec3Pod, AABBPod, BodyId, WorldStatePod, MAX_BODIES};
use bytemuck::{Pod, Zeroable};

// POD command enum - fixed size
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CommandPod {
    pub cmd_type: u32, // 0=GetState, 1=PostAction, 2=Step, 3=Reset
    pub _padding: [u8; 12], // pad to 16 bytes
}

impl CommandPod {
    pub const GET_STATE: u32 = 0;
    pub const POST_ACTION: u32 = 1;
    pub const STEP: u32 = 2;
    pub const RESET: u32 = 3;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ActionPod {
    pub body_id: BodyId,
    pub velocity: Vec3Pod,
    pub position: Vec3Pod,
    pub aabb: AABBPod,
    pub mass: f32,
    pub restitution: f32,
    pub dynamic: u32,
    pub _padding: [u8; 4],
}

// Zero-copy message wrapper
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MessagePod {
    pub msg_type: u32, // 0=Command, 1=Action, 2=WorldState
    pub _padding: [u8; 12],
    pub data: [u8; 8192], // fixed buffer for any message type
}

impl MessagePod {
    pub const TYPE_COMMAND: u32 = 0;
    pub const TYPE_ACTION: u32 = 1;
    pub const TYPE_WORLD_STATE: u32 = 2;

    pub fn as_command(&self) -> &CommandPod {
        bytemuck::from_bytes(&self.data[..std::mem::size_of::<CommandPod>()])
    }

    pub fn as_action(&self) -> &ActionPod {
        bytemuck::from_bytes(&self.data[..std::mem::size_of::<ActionPod>()])
    }

    pub fn as_world_state(&self) -> &WorldStatePod {
        bytemuck::from_bytes(&self.data[..std::mem::size_of::<WorldStatePod>()])
    }

    pub fn from_command(cmd: CommandPod) -> Self {
        let mut msg = Self::zeroed();
        msg.msg_type = Self::TYPE_COMMAND;
        let bytes = bytemuck::bytes_of(&cmd);
        msg.data[..bytes.len()].copy_from_slice(bytes);
        msg
    }

    pub fn from_action(action: ActionPod) -> Self {
        let mut msg = Self::zeroed();
        msg.msg_type = Self::TYPE_ACTION;
        let bytes = bytemuck::bytes_of(&action);
        msg.data[..bytes.len()].copy_from_slice(bytes);
        msg
    }

    pub fn from_world_state(state: WorldStatePod) -> Self {
        let mut msg = Self::zeroed();
        msg.msg_type = Self::TYPE_WORLD_STATE;
        let bytes = bytemuck::bytes_of(&state);
        msg.data[..bytes.len()].copy_from_slice(bytes);
        msg
    }
}

// Conversion helpers
impl From<super::Command> for CommandPod {
    fn from(cmd: super::Command) -> Self {
        let cmd_type = match cmd {
            super::Command::GetState => CommandPod::GET_STATE,
            super::Command::PostAction => CommandPod::POST_ACTION,
            super::Command::Step => CommandPod::STEP,
            super::Command::Reset => CommandPod::RESET,
        };
        Self { cmd_type, _padding: [0; 12] }
    }
}

impl From<&super::Action> for ActionPod {
    fn from(action: &super::Action) -> Self {
        Self {
            body_id: BodyId::new(&action.body_id),
            velocity: action.velocity.into(),
            position: action.position.into(),
            aabb: action.aabb.clone().into(),
            mass: action.mass,
            restitution: action.restitution,
            dynamic: if action.dynamic { 1 } else { 0 },
            _padding: [0; 4],
        }
    }
}