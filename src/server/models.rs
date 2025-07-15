use crate::physics::object::{RigidBody, Vec3, AABB};
use crate::physics::world::World;
use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
    GetState,
    PostAction,
    Step,
    Reset,
}
impl Command {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .unwrap()
            .0
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
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
    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .unwrap()
            .0
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct WorldState {
    pub bodies: Vec<RigidBody>,
    pub time: f32,
}
impl WorldState {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .unwrap()
            .0
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap()
    }
}
