mod game;
mod graphics;
mod physics;
mod server;

// Complete replacement for graphics client
use crate::game::game_engine::{GameObject, GameObjectType};
use crate::graphics::{Camera, Renderer};
use crate::physics::world::World;
use crate::physics::{RigidBody, Vector3};
use crate::server::models::WorldState;
use crate::server::server::Engine;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use zmq::{Context, Socket, SocketType};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("3D Pong Game")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap(),
    );
    let mut renderer = Renderer::new(window.clone()).await;
    let window_id = renderer.window().id();
    let window_size = renderer.window().inner_size();
    let mut camera = Camera::new(window_size.width, window_size.height);

    let mut world = World::new_empty();
    let mut game_objects = Vec::new();

    let wall_thickness = 1.0;
    let arena_width = 60.0;
    let arena_height = 40.0;
    let arena_depth = 40.0;

    let walls = vec![
        (
            Vector3::new(0.0, -arena_height / 2.0, 0.0),
            Vector3::new(arena_width / 2.0, wall_thickness, arena_depth / 2.0),
        ),
        (
            Vector3::new(0.0, arena_height / 2.0, 0.0),
            Vector3::new(arena_width / 2.0, wall_thickness, arena_depth / 2.0),
        ),
        (
            Vector3::new(0.0, 0.0, -arena_depth / 2.0),
            Vector3::new(arena_width / 2.0, arena_height / 2.0, wall_thickness),
        ),
        (
            Vector3::new(0.0, 0.0, arena_depth / 2.0),
            Vector3::new(arena_width / 2.0, arena_height / 2.0, wall_thickness),
        ),
    ];

    for (i, (position, half_extents)) in walls.iter().enumerate() {
        let wall = RigidBody::from_extents_with_id(
            format!("wall_{}", i),
            *position,
            Vector3::zero(),
            *half_extents,
            0.0,
            1.0,
            true,
        );
        world.add_body(wall.clone());
        game_objects.push(GameObject::new(wall, GameObjectType::Wall));
    }

    let paddle1 = RigidBody::from_extents_with_id(
        "paddle1".to_string(),
        Vector3::new(-25.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0),
        0.0,
        1.0,
        false,
    );
    let paddle1_index = world.bodies.len();
    world.add_body(paddle1.clone());
    game_objects.push(GameObject::new(paddle1, GameObjectType::Paddle));

    let paddle2 = RigidBody::from_extents_with_id(
        "paddle2".to_string(),
        Vector3::new(25.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0),
        1000.0,
        1.0,
        false,
    );
    let paddle2_index = world.bodies.len();
    world.add_body(paddle2.clone());
    game_objects.push(GameObject::new(paddle2, GameObjectType::Paddle));

    let ball = RigidBody::from_extents_with_id(
        "ball".to_string(),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(1.0, 1.0, 1.0),
        0.0,
        1.0,
        false,
    );
    let ball_index = world.bodies.len();
    world.add_body(ball.clone());
    game_objects.push(GameObject::new(ball, GameObjectType::Ball));

    println!("World has {} bodies", world.bodies.len());
    println!("Game objects has {} objects", game_objects.len());

    let mut engine = Engine::new_server(
        "tcp://127.0.0.1:5555",
        "tcp://127.0.0.1:5556",
        world.clone(),
    )
    .unwrap();
    engine.run().unwrap();
}
