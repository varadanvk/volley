# Volley - 3D Game Engine for Pong

A 3D volleyball-inspired game built with Rust, WebGPU, and custom physics.

## Controls

### Player 1 (Left Paddle - Green)

- **W/S**: Move forward/backward (X-axis)
- **A/D**: Move left/right (Z-axis)
- **Space**: Move up (Y-axis)
- **Shift**: Move down (Y-axis)

### Player 2 (Right Paddle - Green)

- **Arrow Up/Down**: Move up/down (Y-axis)
- **Arrow Left/Right**: Move forward/backward (Z-axis)

## Game Rules

- Ball spawns in the center
- Score when the ball passes the opponent's goal line (invisible left/right boundaries)
- Ball respawns after each score

## Building and Running

### Prerequisites

- Rust (latest stable version)
- GPU with WebGPU support

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

## Architecture

- **Graphics Module**: Handles rendering, camera, and shaders
- **Physics Module**: Manages rigid bodies, collisions, and movement
- **Game Logic**: Score tracking and game state management

## Dependencies

- `wgpu`: Graphics rendering
- `winit`: Window management
- `glam`: Linear algebra
- `bytemuck`: Byte casting for GPU buffers
- `pollster`: Async runtime for initialization
