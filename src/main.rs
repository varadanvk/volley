mod game;
mod graphics;
mod physics;
mod server;

use crate::game::game_engine::{GameObject, GameObjectType};
use crate::graphics::{Camera, Renderer};
use crate::physics::world::World;
use crate::physics::{RigidBody, Vector3};
use crate::server::ipc::IPCChannel;
use crate::server::models::{Action, WorldState};
use crate::server::server::Engine;
use glam::Vec3;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    env_logger::init();

    // Create the world and initial game objects
    let mut world = World::new_empty();

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
        world.add_body(wall);
    }

    let paddle1 = RigidBody::from_extents_with_id(
        "paddle1".to_string(),
        Vector3::new(-25.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0),
        1000.0,
        1.0,
        false,
    );
    world.add_body(paddle1);

    let paddle2 = RigidBody::from_extents_with_id(
        "paddle2".to_string(),
        Vector3::new(25.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0),
        1000.0,
        1.0,
        false,
    );
    world.add_body(paddle2);

    let ball = RigidBody::from_extents_with_id(
        "ball".to_string(),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(8.0, 4.0, 0.0),
        Vector3::new(0.5, 0.5, 0.5),
        1.0,
        1.0,
        false,
    );
    world.add_body(ball);

    println!("World has {} bodies", world.bodies.len());

    // Start the server in a background thread
    let server_world = world.clone();
    thread::spawn(move || {
        let mut engine =
            Engine::new_server("tcp://127.0.0.1:5555", "tcp://127.0.0.1:5556", server_world)
                .expect("Failed to create server");

        println!("Server started on ports 5555 (actions) and 5556 (state)");
        engine.run().expect("Server failed");
    });

    // Give server time to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create client IPC channels
    let action_channel =
        IPCChannel::new_push("tcp://127.0.0.1:5555").expect("Failed to connect to action channel");
    let state_channel =
        IPCChannel::new_sub("tcp://127.0.0.1:5556").expect("Failed to connect to state channel");

    // Create window and renderer
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

    // Initialize game objects for rendering
    let mut game_objects = Vec::new();
    for body in world.bodies.iter() {
        let obj_type = match body.id.as_str() {
            "ball" => GameObjectType::Ball,
            "paddle1" | "paddle2" => GameObjectType::Paddle,
            _ => GameObjectType::Wall,
        };
        game_objects.push(GameObject::new(body.clone(), obj_type));
    }

    let mut last_time = Instant::now();
    let mut keys_pressed = HashSet::<KeyCode>::new();
    let mut camera_mode = false;
    let mut score_player1 = 0;
    let mut score_player2 = 0;

    let _ = event_loop.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id: window_id_ev,
            } if window_id_ev == window_id => match event {
                WindowEvent::CloseRequested => event_loop_window_target.exit(),
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                    camera.resize(physical_size.width, physical_size.height);
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    let new_size = renderer.window().inner_size();
                    renderer.resize(new_size);
                    camera.resize(new_size.width, new_size.height);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(keycode) = event.physical_key {
                        match event.state {
                            ElementState::Pressed => {
                                if !event.repeat && keycode == KeyCode::KeyC {
                                    camera_mode = !camera_mode;
                                }
                                keys_pressed.insert(keycode);
                            }
                            ElementState::Released => {
                                keys_pressed.remove(&keycode);
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = (now - last_time).as_secs_f64();
                last_time = now;

                // Process ALL available state updates (get the latest)
                let mut latest_state = None;
                while let Ok(bytes) = state_channel.recv_bytes_nonblocking() {
                    if let Ok(world_state) = WorldState::from_msgpack(&bytes) {
                        latest_state = Some(world_state);
                    }
                }

                if let Some(world_state) = latest_state {
                    // Update game objects from server state
                    for (i, body) in world_state.bodies.iter().enumerate() {
                        if i < game_objects.len() {
                            game_objects[i].body = body.clone();
                        }
                    }

                    // Update scores
                    if world_state.score_player1 != score_player1
                        || world_state.score_player2 != score_player2
                    {
                        score_player1 = world_state.score_player1;
                        score_player2 = world_state.score_player2;
                        println!(
                            "Score: Player 1: {} - Player 2: {}",
                            score_player1, score_player2
                        );
                    }
                }

                // Handle paddle movements via IPC
                let paddle_speed = 15.0;
                let mut paddle1_vel = Vector3::zero();
                let mut paddle2_vel = Vector3::zero();

                // Paddle 1 movement
                if keys_pressed.contains(&KeyCode::KeyW) {
                    paddle1_vel.x = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyS) {
                    paddle1_vel.x = -paddle_speed;
                }

                if keys_pressed.contains(&KeyCode::Space) {
                    paddle1_vel.y = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::ShiftLeft) {
                    paddle1_vel.y = -paddle_speed;
                }

                if keys_pressed.contains(&KeyCode::KeyA) {
                    paddle1_vel.z = -paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyD) {
                    paddle1_vel.z = paddle_speed;
                }

                // Always send paddle1 velocity (including zero)
                if let Some(paddle1) = game_objects.iter().find(|obj| obj.body.id == "paddle1") {
                    let action = Action {
                        body_id: "paddle1".to_string(),
                        velocity: paddle1_vel,
                        position: paddle1.body.position,
                        aabb: paddle1.body.aabb.clone(),
                        mass: paddle1.body.mass,
                        restitution: paddle1.body.restitution,
                        dynamic: paddle1.body.dynamic,
                    };
                    if let Ok(bytes) = action.to_msgpack() {
                        let _ = action_channel.send_bytes(&bytes);
                    }
                }

                // Paddle 2 movement (when not in camera mode)
                if !camera_mode {
                    if keys_pressed.contains(&KeyCode::ArrowUp) {
                        paddle2_vel.y = paddle_speed;
                    } else if keys_pressed.contains(&KeyCode::ArrowDown) {
                        paddle2_vel.y = -paddle_speed;
                    }

                    if keys_pressed.contains(&KeyCode::ArrowLeft) {
                        paddle2_vel.z = paddle_speed;
                    } else if keys_pressed.contains(&KeyCode::ArrowRight) {
                        paddle2_vel.z = -paddle_speed;
                    }

                    // Always send paddle2 velocity (including zero)
                    if let Some(paddle2) = game_objects.iter().find(|obj| obj.body.id == "paddle2")
                    {
                        let action = Action {
                            body_id: "paddle2".to_string(),
                            velocity: paddle2_vel,
                            position: paddle2.body.position,
                            aabb: paddle2.body.aabb.clone(),
                            mass: paddle2.body.mass,
                            restitution: paddle2.body.restitution,
                            dynamic: paddle2.body.dynamic,
                        };
                        if let Ok(bytes) = action.to_msgpack() {
                            let _ = action_channel.send_bytes(&bytes);
                        }
                    }
                }

                // Camera controls
                if camera_mode {
                    let rotation_speed = 2.0;
                    if keys_pressed.contains(&KeyCode::ArrowLeft) {
                        camera.yaw += rotation_speed * dt as f32;
                    }
                    if keys_pressed.contains(&KeyCode::ArrowRight) {
                        camera.yaw -= rotation_speed * dt as f32;
                    }
                    if keys_pressed.contains(&KeyCode::ArrowUp) {
                        camera.pitch += rotation_speed * dt as f32;
                    }
                    if keys_pressed.contains(&KeyCode::ArrowDown) {
                        camera.pitch -= rotation_speed * dt as f32;
                    }
                    camera.pitch = camera.pitch.clamp(
                        -std::f32::consts::FRAC_PI_2 + 0.01,
                        std::f32::consts::FRAC_PI_2 - 0.01,
                    );
                }

                // Update camera to follow paddle1
                if let Some(paddle1) = game_objects.iter().find(|obj| obj.body.id == "paddle1") {
                    let paddle1_pos = paddle1.body.position;
                    camera.position =
                        Vec3::new(paddle1_pos.x + 2.0, paddle1_pos.y + 1.0, paddle1_pos.z);
                }

                // Render directly with game_objects - no need to sort for transparency
                match renderer.render(&camera, &game_objects) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        eprintln!("Surface lost!");
                        let size = renderer.window().inner_size();
                        renderer.resize(size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("Out of memory!");
                        event_loop_window_target.exit();
                    }
                    Err(e) => eprintln!("Render error: {:?}", e),
                }
            }
            _ => {}
        }
    });
}
