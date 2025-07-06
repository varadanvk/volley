pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

pub struct AABB {
    min: Vec3,
    max: Vec3,
}
impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        AABB { min, max }
    }

    pub fn from_center_size(center: &Vec3, size: &Vec3) -> Self {
        let half_size = Vec3::new(size.x / 2.0, size.y / 2.0, size.z / 2.0);
        let min = Vec3::new(
            center.x - half_size.x,
            center.y - half_size.y,
            center.z - half_size.z,
        );
        let max = Vec3::new(
            center.x + half_size.x,
            center.y + half_size.y,
            center.z + half_size.z,
        );
        AABB { min, max }
    }
}
pub struct RigidBody {
    id: String,
    position: Vec3,
    velocity: Vec3,
    dynamic: bool,
    aabb: AABB,
    mass: f32,
    restitution: f32,
}
impl RigidBody {
    pub fn new(
        id: String,
        position: Vec3,
        velocity: Vec3,
        dynamic: bool,
        aabb: AABB,
        mass: f32,
        restitution: f32,
    ) -> Self {
        RigidBody {
            id,
            position,
            velocity,
            dynamic,
            aabb,
            mass,
            restitution,
        }
    }
    pub fn new_static(
        id: String,
        position: Vec3,
        velocity: Vec3,
        aabb: AABB,
        restitution: f32,
    ) -> Self {
        RigidBody {
            id,
            position,
            velocity,
            dynamic: false,
            aabb,
            mass: 1.0,
            restitution,
        }
    }
    pub fn new_dynamic(
        id: String,
        position: Vec3,
        velocity: Vec3,
        aabb: AABB,
        mass: f32,
        restitution: f32,
    ) -> Self {
        RigidBody {
            id,
            position,
            velocity,
            dynamic: true,
            aabb,
            mass,
            restitution,
        }
    }
}
