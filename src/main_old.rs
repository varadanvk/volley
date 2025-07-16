mod game;
mod graphics;
mod physics;
mod server;

use game::game_engine::{GameObject, GameObjectType, GameState};
use glam::Vec3;
use graphics::{Camera, Renderer};
use physics::{RigidBody, Vector3, World};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use warp::Filter;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[derive(Deserialize)]
struct Action {
    vel_y: f32,
    vel_z: f32,
}

fn main() {
    pollster::block_on(run());
}

async fn run() {
    env_logger::init();

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

    // Only floor, ceiling, and front/back walls - no left/right walls for scoring
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
        Vector3::new(-25.0, 0.0, 0.0), // Further left in bigger arena
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0), // Slightly bigger paddle
        1000.0,
        1.0,
        false,
    );
    let paddle1_index = world.bodies.len();
    world.add_body(paddle1.clone());
    game_objects.push(GameObject::new(paddle1, GameObjectType::Paddle));

    let paddle2 = RigidBody::from_extents_with_id(
        "paddle2".to_string(),
        Vector3::new(25.0, 0.0, 0.0), // Further right in bigger arena
        Vector3::zero(),
        Vector3::new(1.0, 3.0, 3.0), // Slightly bigger paddle
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
        Vector3::new(8.0, 4.0, 0.0),
        Vector3::new(0.5, 0.5, 0.5),
        1.0,
        1.0,
        false,
    );
    let ball_index = world.bodies.len();
    world.add_body(ball.clone());
    game_objects.push(GameObject::new(ball, GameObjectType::Ball));

    let mut game_state = GameState::new(paddle1_index, paddle2_index, ball_index);

    println!("Created {} game objects", game_objects.len());
    println!("World has {} bodies", world.bodies.len());

    let world_arc = Arc::new(Mutex::new(world));
    let game_state_arc = Arc::new(Mutex::new(game_state));
    let paddle2_index_arc = Arc::new(paddle2_index);

    // Spawn server thread
    let world_clone = world_arc.clone();
    let game_state_clone = game_state_arc.clone();
    let paddle2_index_clone = paddle2_index_arc.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let world_clone1 = world_clone.clone();
            let game_state_clone1 = game_state_clone.clone();
            let state = warp::get().and(warp::path("state")).map(move || {
                let world = world_clone1.lock().unwrap();
                let game_state = game_state_clone1.lock().unwrap();
                let response = serde_json::json!({
                    "world": *world,
                    "game_state": *game_state
                });
                warp::reply::json(&response)
            });

            let world_clone2 = world_clone.clone();
            let paddle2_index_clone2 = paddle2_index_clone.clone();
            let action = warp::post()
                .and(warp::path("action"))
                .and(warp::body::json())
                .map(move |act: Action| {
                    let mut world = world_clone2.lock().unwrap();
                    let idx = *paddle2_index_clone2;
                    world.bodies[idx].velocity.y = act.vel_y;
                    world.bodies[idx].velocity.z = act.vel_z;
                    "OK"
                });

            warp::serve(state.or(action))
                .run(([127, 0, 0, 1], 3030))
                .await;
        });
    });

    let mut last_time = Instant::now();
    let mut keys_pressed = std::collections::HashSet::<KeyCode>::new();
    let mut camera_mode = false;

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

                let paddle_speed = 15.0; // Increased speed for larger arena

                // Paddle 1 movement - Y axis (up/down)
                let mut world = world_arc.lock().unwrap();
                if keys_pressed.contains(&KeyCode::Space) {
                    world.bodies[paddle1_index].velocity.y = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::ShiftLeft) {
                    world.bodies[paddle1_index].velocity.y = -paddle_speed;
                } else {
                    world.bodies[paddle1_index].velocity.y = 0.0;
                }

                // Paddle 1 movement - X axis (left/right)
                if keys_pressed.contains(&KeyCode::KeyS) {
                    world.bodies[paddle1_index].velocity.x = -paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyW) {
                    world.bodies[paddle1_index].velocity.x = paddle_speed;
                } else {
                    world.bodies[paddle1_index].velocity.x = 0.0;
                }

                // Paddle 1 movement - Z axis (forward/backward)
                if keys_pressed.contains(&KeyCode::KeyA) {
                    world.bodies[paddle1_index].velocity.z = -paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyD) {
                    world.bodies[paddle1_index].velocity.z = paddle_speed;
                } else {
                    world.bodies[paddle1_index].velocity.z = 0.0;
                }

                // Paddle 2 movement
                if !camera_mode {
                    if keys_pressed.contains(&KeyCode::ArrowUp) {
                        world.bodies[paddle2_index].velocity.y = paddle_speed;
                    } else if keys_pressed.contains(&KeyCode::ArrowDown) {
                        world.bodies[paddle2_index].velocity.y = -paddle_speed;
                    } else if !keys_pressed.contains(&KeyCode::ArrowUp)
                        && !keys_pressed.contains(&KeyCode::ArrowDown)
                    {
                        world.bodies[paddle2_index].velocity.y = 0.0;
                    }

                    if keys_pressed.contains(&KeyCode::ArrowLeft) {
                        world.bodies[paddle2_index].velocity.z = paddle_speed;
                    } else if keys_pressed.contains(&KeyCode::ArrowRight) {
                        world.bodies[paddle2_index].velocity.z = -paddle_speed;
                    } else if !keys_pressed.contains(&KeyCode::ArrowLeft)
                        && !keys_pressed.contains(&KeyCode::ArrowRight)
                    {
                        world.bodies[paddle2_index].velocity.z = 0.0;
                    }
                }

                if camera_mode {
                    let rotation_speed = 2.0; // radians per second
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

                world.step(dt);

                // Clamp paddles within arena bounds
                let arena_half_width = arena_width / 2.0;
                let arena_half_height = arena_height / 2.0;
                let arena_half_depth = arena_depth / 2.0;

                // Paddle 1 constraints (green boxes)
                let paddle1 = &mut world.bodies[paddle1_index];
                let paddle1_half_extents = paddle1.get_half_extents();
                let paddle1_half_x = paddle1_half_extents.x;
                let paddle1_half_y = paddle1_half_extents.y;
                let paddle1_half_z = paddle1_half_extents.z;

                // Clamp X position
                if paddle1.position.x - paddle1_half_x < -arena_half_width {
                    paddle1.position.x = -arena_half_width + paddle1_half_x;
                    paddle1.velocity.x = 0.0;
                } else if paddle1.position.x + paddle1_half_x > arena_half_width {
                    paddle1.position.x = arena_half_width - paddle1_half_x;
                    paddle1.velocity.x = 0.0;
                }

                // Clamp Y position
                if paddle1.position.y - paddle1_half_y < -arena_half_height {
                    paddle1.position.y = -arena_half_height + paddle1_half_y;
                    paddle1.velocity.y = 0.0;
                } else if paddle1.position.y + paddle1_half_y > arena_half_height {
                    paddle1.position.y = arena_half_height - paddle1_half_y;
                    paddle1.velocity.y = 0.0;
                }

                // Clamp Z position
                if paddle1.position.z - paddle1_half_z < -arena_half_depth {
                    paddle1.position.z = -arena_half_depth + paddle1_half_z;
                    paddle1.velocity.z = 0.0;
                } else if paddle1.position.z + paddle1_half_z > arena_half_depth {
                    paddle1.position.z = arena_half_depth - paddle1_half_z;
                    paddle1.velocity.z = 0.0;
                }

                // Paddle 2 constraints
                let paddle2 = &mut world.bodies[paddle2_index];
                let paddle2_half_extents = paddle2.get_half_extents();
                let paddle2_half_y = paddle2_half_extents.y;
                let paddle2_half_z = paddle2_half_extents.z;

                // Clamp Y position
                if paddle2.position.y - paddle2_half_y < -arena_half_height {
                    paddle2.position.y = -arena_half_height + paddle2_half_y;
                    paddle2.velocity.y = 0.0;
                } else if paddle2.position.y + paddle2_half_y > arena_half_height {
                    paddle2.position.y = arena_half_height - paddle2_half_y;
                    paddle2.velocity.y = 0.0;
                }

                // Clamp Z position
                if paddle2.position.z - paddle2_half_z < -arena_half_depth {
                    paddle2.position.z = -arena_half_depth + paddle2_half_z;
                    paddle2.velocity.z = 0.0;
                } else if paddle2.position.z + paddle2_half_z > arena_half_depth {
                    paddle2.position.z = arena_half_depth - paddle2_half_z;
                    paddle2.velocity.z = 0.0;
                }

                drop(world); // Release lock

                let mut game_state = game_state_arc.lock().unwrap();
                if let Some(scorer) = game_state.check_scoring(&game_objects) {
                    println!(
                        "Player {} scored! Score: {} - {}",
                        scorer, game_state.score_player1, game_state.score_player2
                    );

                    let mut world = world_arc.lock().unwrap();
                    world.bodies[ball_index].position = Vector3::new(0.0, 0.0, 0.0);
                    world.bodies[ball_index].velocity = if scorer == 1 {
                        Vector3::new(-8.0, 4.0, 0.0)
                    } else {
                        Vector3::new(8.0, 4.0, 0.0)
                    };
                }

                // Update game_objects from world
                let world = world_arc.lock().unwrap();
                for (i, body) in world.bodies.iter().enumerate() {
                    game_objects[i].body = body.clone();
                }
                drop(world);

                // Update camera to follow paddle1 in first person
                let paddle1_pos = game_objects[paddle1_index].body.position;
                camera.position = Vec3::new(
                    paddle1_pos.x + 2.0, // Slightly behind the paddle
                    paddle1_pos.y + 1.0, // Slightly above center
                    paddle1_pos.z,
                );

                // Sort game objects for transparency
                let mut opaque = vec![];
                let mut transparent = vec![];
                for obj in game_objects.iter() {
                    if obj.color[3] < 1.0 {
                        transparent.push(obj);
                    } else {
                        opaque.push(obj);
                    }
                }

                // Sort transparent back to front
                transparent.sort_by(|a, b| {
                    let pos_a =
                        glam::Vec3::new(a.body.position.x, a.body.position.y, a.body.position.z);
                    let dist_a = (camera.position - pos_a).length_squared();
                    let pos_b =
                        glam::Vec3::new(b.body.position.x, b.body.position.y, b.body.position.z);
                    let dist_b = (camera.position - pos_b).length_squared();
                    dist_b.partial_cmp(&dist_a).unwrap()
                });

                let mut all_objects: Vec<GameObject> = opaque.into_iter().cloned().collect();
                all_objects.extend(transparent.into_iter().cloned());

                // Render
                match renderer.render(&camera, &all_objects) {
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
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id: window_id_ev,
            } => {
                println!(
                    "RedrawRequested! window_id_ev={:?}, window_id={:?}, match={}",
                    window_id_ev,
                    window_id,
                    window_id_ev == window_id
                );
                if window_id_ev == window_id {
                    match renderer.render(&camera, &game_objects) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = renderer.window().inner_size();
                            renderer.resize(size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop_window_target.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            _ => {}
        }
    });
}
