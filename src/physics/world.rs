use crate::physics::object::AABB;

use super::object::{RigidBody, Vec3};
use rayon::prelude::*;
use std::cmp::{max, min};
use std::{sync::TryLockResult, thread};

pub struct World {
    id: String,
    tick_rate: f32,
    pub bodies: Vec<RigidBody>,
}
impl World {
    pub fn new(id: String, tick_rate: f32, bodies: Vec<RigidBody>) -> Self {
        World {
            id,
            tick_rate,
            bodies,
        }
    }

    pub fn new_empty() -> Self {
        World {
            id: "world".to_string(),
            tick_rate: 60.0,
            bodies: Vec::new(),
        }
    }

    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }
    pub fn step(&mut self, dt: f64) {
        let dt_f32 = dt as f32;
        self.bodies.par_iter_mut().for_each(|body| {
            if body.dynamic {
                body.position = body.position + (body.velocity * dt_f32);
                body.compute_aabb();
            }
        });

        // Run collision detection after updating positions
        self.collide_pong();
    }

    pub fn check_collision(body_1: &RigidBody, body_2: &RigidBody) -> bool {
        let ab2: &AABB = &body_2.aabb;
        let ab1: &AABB = &body_1.aabb;
        let collide_x = ab1.max.x >= ab2.min.x && ab1.min.x <= ab2.max.x;
        let collide_y = ab1.max.y >= ab2.min.y && ab1.min.y <= ab2.max.y;
        let collide_z = ab1.max.z >= ab2.min.z && ab1.min.z <= ab2.max.z;

        let collide = collide_x && collide_y && collide_z;
        collide
    }

    pub fn handle_collision(body_1: &mut RigidBody, body_2: &mut RigidBody) {
        // Find collision normal (direction to separate objects)
        // Calculate penetration depths on each axis
        let penetration_x =
            (body_1.aabb.max.x.min(body_2.aabb.max.x)) - (body_1.aabb.min.x.max(body_2.aabb.min.x));
        let penetration_y =
            (body_1.aabb.max.y.min(body_2.aabb.max.y)) - (body_1.aabb.min.y.max(body_2.aabb.min.y));
        let penetration_z =
            (body_1.aabb.max.z.min(body_2.aabb.max.z)) - (body_1.aabb.min.z.max(body_2.aabb.min.z));

        // Find the axis with minimum penetration (collision normal direction)
        let mut normal = Vec3::zero();
        let min_penetration;

        if penetration_x <= penetration_y && penetration_x <= penetration_z {
            // X axis has minimum penetration
            min_penetration = penetration_x;
            normal.x = if body_1.position.x < body_2.position.x {
                -1.0
            } else {
                1.0
            };
        } else if penetration_y <= penetration_z {
            // Y axis has minimum penetration
            min_penetration = penetration_y;
            normal.y = if body_1.position.y < body_2.position.y {
                -1.0
            } else {
                1.0
            };
        } else {
            // Z axis has minimum penetration
            min_penetration = penetration_z;
            normal.z = if body_1.position.z < body_2.position.z {
                -1.0
            } else {
                1.0
            };
        }

        // Calculate relative velocity along the collision normal
        let relative_velocity = Vec3::new(
            body_1.velocity.x - body_2.velocity.x,
            body_1.velocity.y - body_2.velocity.y,
            body_1.velocity.z - body_2.velocity.z,
        );
        let velocity_along_normal = relative_velocity.x * normal.x
            + relative_velocity.y * normal.y
            + relative_velocity.z * normal.z;

        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.0 {
            return;
        }

        // Handle static bodies (infinite mass)
        let inv_mass_1 = if body_1.dynamic {
            1.0 / body_1.mass
        } else {
            0.0
        };
        let inv_mass_2 = if body_2.dynamic {
            1.0 / body_2.mass
        } else {
            0.0
        };

        // Compute impulse magnitude using masses and restitution
        let restitution = body_1.restitution.min(body_2.restitution);
        let impulse_magnitude =
            -(1.0 + restitution) * velocity_along_normal / (inv_mass_1 + inv_mass_2);

        // Apply equal and opposite impulses to both objects
        let impulse = Vec3::new(
            impulse_magnitude * normal.x,
            impulse_magnitude * normal.y,
            impulse_magnitude * normal.z,
        );

        // Apply impulse to body_1 (add impulse / mass to velocity)
        if body_1.dynamic {
            body_1.velocity.x += impulse.x * inv_mass_1;
            body_1.velocity.y += impulse.y * inv_mass_1;
            body_1.velocity.z += impulse.z * inv_mass_1;
        }

        // Apply opposite impulse to body_2 (subtract impulse / mass from velocity)
        if body_2.dynamic {
            body_2.velocity.x -= impulse.x * inv_mass_2;
            body_2.velocity.y -= impulse.y * inv_mass_2;
            body_2.velocity.z -= impulse.z * inv_mass_2;
        }

        // Position correction to separate overlapping objects
        let total_inv_mass = inv_mass_1 + inv_mass_2;
        let correction_amount = min_penetration * 0.8; // 80% correction to avoid jitter

        if body_1.dynamic {
            let correction_1 = correction_amount * (inv_mass_1 / total_inv_mass);
            body_1.position.x += normal.x * correction_1;
            body_1.position.y += normal.y * correction_1;
            body_1.position.z += normal.z * correction_1;
            body_1.compute_aabb(); // Update AABB after position change
        }

        if body_2.dynamic {
            let correction_2 = correction_amount * (inv_mass_2 / total_inv_mass);
            body_2.position.x -= normal.x * correction_2;
            body_2.position.y -= normal.y * correction_2;
            body_2.position.z -= normal.z * correction_2;
            body_2.compute_aabb(); // Update AABB after position change
        }
    }
    //note: this is only for pong, complete physics sim works for all
    pub fn collide_pong(&mut self) {
        let mut collision_pairs = Vec::new();

        for i in 0..self.bodies.len() {
            if self.bodies[i].id.starts_with("ball") {
                for j in 0..self.bodies.len() {
                    if i != j
                        && (self.bodies[j].id.starts_with("paddle")
                            || self.bodies[j].id.starts_with("wall"))
                    {
                        if Self::check_collision(&self.bodies[i], &self.bodies[j]) {
                            collision_pairs.push((i, j));
                        }
                    }
                }
            } else if self.bodies[i].id.starts_with("paddle") {
                for j in 0..self.bodies.len() {
                    if i != j && self.bodies[j].id.starts_with("wall") {
                        if Self::check_collision(&self.bodies[i], &self.bodies[j]) {
                            collision_pairs.push((i, j));
                        }
                    }
                }
            }
        }

        // Handle collisions
        for (i, j) in collision_pairs {
            // Need to split borrow to avoid borrow checker issues
            let (body1, body2) = if i < j {
                let (left, right) = self.bodies.split_at_mut(j);
                (&mut left[i], &mut right[0])
            } else {
                let (left, right) = self.bodies.split_at_mut(i);
                (&mut right[0], &mut left[j])
            };

            Self::handle_collision(body1, body2);
        }
    }
    pub fn collide(&mut self) {
        let collision_pairs: Vec<(usize, usize)> = (0..self.bodies.len())
            .into_par_iter()
            .flat_map(|i| {
                ((i + 1)..self.bodies.len())
                    .into_par_iter()
                    .filter_map(|j| {
                        if Self::check_collision(&self.bodies[i], &self.bodies[j]) {
                            println!(
                                "🔥 COLLISION DETECTED: {} and {}",
                                self.bodies[i].id, self.bodies[j].id
                            );
                            Some((i, j))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if collision_pairs.is_empty() {
            println!("No collisions detected this tick");
        }

        for (i, j) in collision_pairs {
            // Need to split borrow to avoid borrow checker issues
            let (body1, body2) = if i < j {
                let (left, right) = self.bodies.split_at_mut(j);
                (&mut left[i], &mut right[0])
            } else {
                let (left, right) = self.bodies.split_at_mut(i);
                (&mut right[0], &mut left[j])
            };

            Self::handle_collision(body1, body2);
        }
    }
    pub fn tick(&mut self) {
        let dt: f64 = 1.0 / self.tick_rate as f64;
        for body in &self.bodies {
            println!(
                "Body '{}': position ({:.2}, {:.2}, {:.2})",
                body.id, body.position.x, body.position.y, body.position.z
            );
            println!(
                "Body '{}': AABB min({:.2}, {:.2}, {:.2}) max({:.2}, {:.2}, {:.2})",
                body.id,
                body.aabb.min.x,
                body.aabb.min.y,
                body.aabb.min.z,
                body.aabb.max.x,
                body.aabb.max.y,
                body.aabb.max.z
            );
        }
        self.step(dt);

        //TODO: change this to colldie_pong when we start workign on the pong sim
        self.collide();
    }
}
