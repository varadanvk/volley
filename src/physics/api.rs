use crate::physics::object::{AABB, RigidBody, Vec3};
use crate::physics::world::World;

struct Physics {
    objects: Vec<RigidBody>,
    world: World,
}
impl Physics {
    pub fn new() {}
}
