mod physics;
use physics::object::{AABB, RigidBody, Vec3};
use physics::world::World;

pub fn main() {
    // Create a ball that will fall and hit the ground
    let falling_ball = RigidBody::new_dynamic(
        "falling_ball".to_string(),
        Vec3::new(0.0, 5.0, 0.0),    // Start 5 units above ground
        Vec3::new(0.0, -150.0, 0.0), // Much faster: -150 units/second
        AABB::from_center_size(&Vec3::new(0.0, 5.0, 0.0), &Vec3::new(1.0, 1.0, 1.0)),
        1.0, // mass
        0.8, // restitution
    );

    // Create ground plane
    let ground = RigidBody::new_static(
        "ground".to_string(),
        Vec3::new(0.0, -1.0, 0.0), // Ground at Y = -1
        Vec3::zero(),
        AABB::from_center_size(&Vec3::new(0.0, -1.0, 0.0), &Vec3::new(20.0, 1.0, 20.0)),
        0.9, // restitution
    );

    // Create two balls moving toward each other
    let ball_left = RigidBody::new_dynamic(
        "ball_left".to_string(),
        Vec3::new(-2.0, 2.0, 0.0), // Start closer: -2.0 instead of -3.0
        Vec3::new(30.0, 0.0, 0.0), // Slower: 30 instead of 60
        AABB::from_center_size(&Vec3::new(-2.0, 2.0, 0.0), &Vec3::new(0.8, 0.8, 0.8)),
        1.0,
        0.9,
    );

    let ball_right = RigidBody::new_dynamic(
        "ball_right".to_string(),
        Vec3::new(2.0, 2.0, 0.0),   // Start closer: 2.0 instead of 3.0
        Vec3::new(-30.0, 0.0, 0.0), // Slower: -30 instead of -60
        AABB::from_center_size(&Vec3::new(2.0, 2.0, 0.0), &Vec3::new(0.8, 0.8, 0.8)),
        1.0,
        0.9,
    );

    // Create a ball moving diagonally toward the corner
    let diagonal_ball = RigidBody::new_dynamic(
        "diagonal_ball".to_string(),
        Vec3::new(4.0, 4.0, 0.0),
        Vec3::new(-1.5, -1.5, 0.0), // Moving diagonally down-left
        AABB::from_center_size(&Vec3::new(4.0, 4.0, 0.0), &Vec3::new(0.6, 0.6, 0.6)),
        0.8, // lighter mass
        0.7, // restitution
    );

    let bodies = vec![falling_ball, ground, ball_left, ball_right, diagonal_ball];

    // Create world with higher tick rate for more precise collision detection
    let mut world = World::new("collision_test".to_string(), 120.0, bodies); // 120 FPS instead of 30

    // Run simulation for more ticks to see collisions
    println!("Starting collision test simulation...");
    println!("Expected collisions:");
    println!("- falling_ball should hit ground around tick 1-2");
    println!("- ball_left and ball_right should collide around tick 1-2");
    println!("- diagonal_ball should hit ground around tick 2-3");
    println!();

    for i in 0..10 {
        world.tick();
        println!("--- Tick {} ---", i + 1);
        println!();
    }

    println!("Simulation complete!");
}
