pub mod api;
pub mod object;
pub mod world;

// Re-export commonly used types
pub use object::{RigidBody, Vec3, Vec3 as Vector3, AABB};
pub use world::World;
