mod graphics;
mod physics;

use graphics::{Camera, GameObject, GameObjectType, GameState, Renderer};
use physics::{RigidBody, Vector3, World};
use std::sync::Arc;
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
    let arena_width = 30.0;
    let arena_height = 20.0;
    let arena_depth = 20.0;

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
        (
            Vector3::new(-arena_width / 2.0, 0.0, 0.0),
            Vector3::new(wall_thickness, arena_height / 2.0, arena_depth / 2.0),
        ),
        (
            Vector3::new(arena_width / 2.0, 0.0, 0.0),
            Vector3::new(wall_thickness, arena_height / 2.0, arena_depth / 2.0),
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
        Vector3::new(-12.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(0.5, 2.0, 2.0),
        1000.0,
        1.0,
        false,
    );
    let paddle1_index = world.bodies.len();
    world.add_body(paddle1.clone());
    game_objects.push(GameObject::new(paddle1, GameObjectType::Paddle));

    let paddle2 = RigidBody::from_extents_with_id(
        "paddle2".to_string(),
        Vector3::new(12.0, 0.0, 0.0),
        Vector3::zero(),
        Vector3::new(0.5, 2.0, 2.0),
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

    let mut last_time = Instant::now();
    let mut keys_pressed = std::collections::HashSet::<KeyCode>::new();

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

                let paddle_speed = 10.0;

                if keys_pressed.contains(&KeyCode::KeyW) {
                    world.bodies[paddle1_index].velocity.y = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyS) {
                    world.bodies[paddle1_index].velocity.y = -paddle_speed;
                } else {
                    world.bodies[paddle1_index].velocity.y = 0.0;
                }

                if keys_pressed.contains(&KeyCode::KeyA) {
                    world.bodies[paddle1_index].velocity.z = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::KeyD) {
                    world.bodies[paddle1_index].velocity.z = -paddle_speed;
                } else {
                    world.bodies[paddle1_index].velocity.z = 0.0;
                }

                if keys_pressed.contains(&KeyCode::ArrowUp) {
                    world.bodies[paddle2_index].velocity.y = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::ArrowDown) {
                    world.bodies[paddle2_index].velocity.y = -paddle_speed;
                } else {
                    world.bodies[paddle2_index].velocity.y = 0.0;
                }

                if keys_pressed.contains(&KeyCode::ArrowLeft) {
                    world.bodies[paddle2_index].velocity.z = paddle_speed;
                } else if keys_pressed.contains(&KeyCode::ArrowRight) {
                    world.bodies[paddle2_index].velocity.z = -paddle_speed;
                } else {
                    world.bodies[paddle2_index].velocity.z = 0.0;
                }

                world.step(dt);

                for (i, body) in world.bodies.iter().enumerate() {
                    game_objects[i].body = body.clone();
                }

                if let Some(scorer) = game_state.check_scoring(&game_objects) {
                    println!(
                        "Player {} scored! Score: {} - {}",
                        scorer, game_state.score_player1, game_state.score_player2
                    );

                    world.bodies[ball_index].position = Vector3::new(0.0, 0.0, 0.0);
                    world.bodies[ball_index].velocity = if scorer == 1 {
                        Vector3::new(-8.0, 4.0, 0.0)
                    } else {
                        Vector3::new(8.0, 4.0, 0.0)
                    };
                }

                renderer.window().request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id: window_id_ev,
            } if window_id_ev == window_id => match renderer.render(&camera, &game_objects) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => {
                    let size = renderer.window().inner_size();
                    renderer.resize(size);
                }
                Err(wgpu::SurfaceError::OutOfMemory) => event_loop_window_target.exit(),
                Err(e) => eprintln!("{:?}", e),
            },
            _ => {}
        }
    });
}
