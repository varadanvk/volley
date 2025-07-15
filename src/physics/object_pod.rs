use bytemuck::{Pod, Zeroable};

// POD-compatible ID type - 32 bytes fixed
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BodyId {
    pub data: [u8; 32],
}

impl BodyId {
    pub fn new(s: &str) -> Self {
        let mut data = [0u8; 32];
        let bytes = s.as_bytes();
        let len = bytes.len().min(32);
        data[..len].copy_from_slice(&bytes[..len]);
        Self { data }
    }

    pub fn as_str(&self) -> &str {
        let len = self.data.iter().position(|&b| b == 0).unwrap_or(32);
        std::str::from_utf8(&self.data[..len]).unwrap_or("")
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vec3Pod {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AABBPod {
    pub min: Vec3Pod,
    pub max: Vec3Pod,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RigidBodyPod {
    pub id: BodyId,
    pub position: Vec3Pod,
    pub velocity: Vec3Pod,
    pub aabb: AABBPod,
    pub mass: f32,
    pub restitution: f32,
    pub dynamic: u32, // bool as u32 for alignment
    pub _padding: [u8; 4], // explicit padding for alignment
}

// Fixed-size world for zero-copy batches
pub const MAX_BODIES: usize = 1024; // adjust for your RL scale

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WorldStatePod {
    pub bodies: [RigidBodyPod; MAX_BODIES],
    pub body_count: u32,
    pub tick_rate: f32,
    pub time: f32,
    pub _padding: [u8; 4],
}

// Conversion traits
impl From<super::Vec3> for Vec3Pod {
    fn from(v: super::Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Vec3Pod> for super::Vec3 {
    fn from(v: Vec3Pod) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<super::AABB> for AABBPod {
    fn from(aabb: super::AABB) -> Self {
        Self {
            min: aabb.min.into(),
            max: aabb.max.into(),
        }
    }
}

impl From<&super::RigidBody> for RigidBodyPod {
    fn from(rb: &super::RigidBody) -> Self {
        Self {
            id: BodyId::new(&rb.id),
            position: rb.position.into(),
            velocity: rb.velocity.into(),
            aabb: rb.aabb.clone().into(),
            mass: rb.mass,
            restitution: rb.restitution,
            dynamic: if rb.dynamic { 1 } else { 0 },
            _padding: [0; 4],
        }
    }
}

impl RigidBodyPod {
    pub fn to_rigidbody(&self) -> super::RigidBody {
        super::RigidBody {
            id: self.id.as_str().to_string(),
            position: self.position.into(),
            velocity: self.velocity.into(),
            dynamic: self.dynamic != 0,
            aabb: super::AABB {
                min: self.aabb.min.into(),
                max: self.aabb.max.into(),
            },
            mass: self.mass,
            restitution: self.restitution,
        }
    }
}