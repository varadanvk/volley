use crate::physics::object::{RigidBody, Vec3, AABB};
use bincode;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SerializationError {
    EncodeError(bincode::error::EncodeError),
    DecodeError(bincode::error::DecodeError),
}

impl From<bincode::error::EncodeError> for SerializationError {
    fn from(err: bincode::error::EncodeError) -> Self {
        SerializationError::EncodeError(err)
    }
}

impl From<bincode::error::DecodeError> for SerializationError {
    fn from(err: bincode::error::DecodeError) -> Self {
        SerializationError::DecodeError(err)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    GetState,
    PostAction,
    Step,
    Reset,
}
impl Command {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let (command, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(command)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub body_id: String,
    pub velocity: Vec3,
    pub position: Vec3,
    pub aabb: AABB,
    pub mass: f32,
    pub restitution: f32,
    pub dynamic: bool,
} //update based on any rigidbody properties

impl Action {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let (action, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(action)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldState {
    pub bodies: Vec<RigidBody>,
    pub time: f32,
    pub score_player1: u32,
    pub score_player2: u32,
}
impl WorldState {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let (state, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(state)
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        Ok(bytes)
    }
}
