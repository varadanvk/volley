use super::object::{RigidBody, Vec3};

pub struct World {
    tick: i32,
    bodies: Vec<RigidBody>,
}
