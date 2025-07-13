#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
    pub fn update(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

#[derive(Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
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

    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn get_size(&self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn update_from_center(&mut self, center: &Vec3) {
        let size = self.get_size();
        *self = AABB::from_center_size(center, &size);
    }
}
#[derive(Clone)]
pub struct RigidBody {
    pub id: String,
    pub position: Vec3,
    pub velocity: Vec3,
    pub dynamic: bool,
    pub aabb: AABB,
    pub mass: f32,
    pub restitution: f32,
}
impl RigidBody {
    pub fn get_half_extents(&self) -> Vec3 {
        let size = self.aabb.get_size();
        Vec3::new(size.x / 2.0, size.y / 2.0, size.z / 2.0)
    }
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
    
    pub fn from_extents(
        position: Vec3,
        velocity: Vec3,
        half_extents: Vec3,
        mass: f32,
        restitution: f32,
        is_static: bool,
    ) -> Self {
        let size = Vec3::new(half_extents.x * 2.0, half_extents.y * 2.0, half_extents.z * 2.0);
        let aabb = AABB::from_center_size(&position, &size);
        RigidBody {
            id: String::new(),
            position,
            velocity,
            dynamic: !is_static,
            aabb,
            mass,
            restitution,
        }
    }
    
    pub fn from_extents_with_id(
        id: String,
        position: Vec3,
        velocity: Vec3,
        half_extents: Vec3,
        mass: f32,
        restitution: f32,
        is_static: bool,
    ) -> Self {
        let size = Vec3::new(half_extents.x * 2.0, half_extents.y * 2.0, half_extents.z * 2.0);
        let aabb = AABB::from_center_size(&position, &size);
        RigidBody {
            id,
            position,
            velocity,
            dynamic: !is_static,
            aabb,
            mass,
            restitution,
        }
    }
    pub fn update_position(&mut self, x: f32, y: f32, z: f32) {
        self.position.update(x, y, z);
        self.compute_aabb();
    }
    pub fn update_velocity(&mut self, x: f32, y: f32, z: f32) {
        self.velocity.update(x, y, z)
    }
    pub fn compute_aabb(&mut self) {
        self.aabb.update_from_center(&self.position);
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
