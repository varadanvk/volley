use glam::{Mat4, Vec3};
use crate::physics::{RigidBody, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameObjectType {
    Wall,
    Paddle,
    Ball,
}

pub struct GameObject {
    pub body: RigidBody,
    pub object_type: GameObjectType,
    pub color: [f32; 3],
}

impl GameObject {
    pub fn new(body: RigidBody, object_type: GameObjectType) -> Self {
        let color = match object_type {
            GameObjectType::Wall => [0.5, 0.5, 0.5],
            GameObjectType::Paddle => [0.0, 1.0, 0.0],
            GameObjectType::Ball => [1.0, 1.0, 1.0],
        };
        
        Self {
            body,
            object_type,
            color,
        }
    }
    
    pub fn get_model_matrix(&self) -> Mat4 {
        let position = Vec3::new(
            self.body.position.x as f32,
            self.body.position.y as f32,
            self.body.position.z as f32,
        );
        
        let half_extents = self.body.get_half_extents();
        let scale = Vec3::new(
            half_extents.x * 2.0,
            half_extents.y * 2.0,
            half_extents.z * 2.0,
        );
        
        Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, position)
    }
}

pub struct GameState {
    pub score_player1: u32,
    pub score_player2: u32,
    pub paddle1_index: usize,
    pub paddle2_index: usize,
    pub ball_index: usize,
}

impl GameState {
    pub fn new(paddle1_index: usize, paddle2_index: usize, ball_index: usize) -> Self {
        Self {
            score_player1: 0,
            score_player2: 0,
            paddle1_index,
            paddle2_index,
            ball_index,
        }
    }
    
    pub fn check_scoring(&mut self, game_objects: &[GameObject]) -> Option<u8> {
        let ball = &game_objects[self.ball_index].body;
        
        if ball.position.x < -15.0 {
            self.score_player2 += 1;
            return Some(2);
        } else if ball.position.x > 15.0 {
            self.score_player1 += 1;
            return Some(1);
        }
        
        None
    }
    
    pub fn reset_ball(game_objects: &mut [GameObject]) {
        let ball_index = game_objects.iter().position(|obj| obj.object_type == GameObjectType::Ball).unwrap();
        game_objects[ball_index].body.position = Vector3::new(0.0, 0.0, 0.0);
        game_objects[ball_index].body.velocity = Vector3::new(5.0, 2.0, 0.0);
    }
}