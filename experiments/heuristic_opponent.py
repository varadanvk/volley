"""
Simple ball-tracking heuristic opponent for 3D pong

This opponent tracks the ball's position with intentional limitations:
- Speed-capped at 12.0 (vs agent's 15.0)
- Reaction delay to simulate human response time
- Tracking noise to cause occasional misses
"""

import numpy as np
import time


class HeuristicOpponent:
    """Ball-tracking heuristic controller for paddle2"""

    def __init__(self, max_speed: float = 12.0, reaction_delay: float = 0.05, tracking_noise: float = 0.3):
        """
        Initialize the heuristic opponent

        Args:
            max_speed: Maximum paddle velocity (default 12.0, agent's is 15.0)
            reaction_delay: Seconds before responding to ball position change
            tracking_noise: Standard deviation of Gaussian noise added to target position
        """
        self.max_speed = max_speed
        self.reaction_delay = reaction_delay
        self.tracking_noise = tracking_noise
        self.last_update_time = time.time()
        self.target_pos = (0.0, 0.0)  # (y, z) target position

    def get_action(
        self,
        paddle_y: float,
        paddle_z: float,
        ball_y: float,
        ball_z: float,
        current_time: float
    ) -> np.ndarray:
        """
        Compute normalized action [-1, 1] for paddle velocity

        Args:
            paddle_y: Paddle's current y position
            paddle_z: Paddle's current z position
            ball_y: Ball's current y position
            ball_z: Ball's current z position
            current_time: Current timestamp (for reaction delay)

        Returns:
            np.ndarray: [vy, vz] normalized to [-1, 1]
        """
        # Update target position only after reaction delay
        if current_time - self.last_update_time > self.reaction_delay:
            # Track ball with Gaussian noise
            target_y = ball_y + np.random.normal(0, self.tracking_noise)
            target_z = ball_z + np.random.normal(0, self.tracking_noise)

            # Clamp to arena bounds (paddle can move ±20 in y and z)
            target_y = np.clip(target_y, -20.0, 20.0)
            target_z = np.clip(target_z, -20.0, 20.0)

            self.target_pos = (target_y, target_z)
            self.last_update_time = current_time

        # Calculate desired velocity toward target
        dy = self.target_pos[0] - paddle_y
        dz = self.target_pos[1] - paddle_z

        # Normalize velocity vector to max_speed
        distance = np.sqrt(dy**2 + dz**2)
        if distance > 0.1:  # Avoid division by zero
            vel_y = (dy / distance) * self.max_speed
            vel_z = (dz / distance) * self.max_speed
        else:
            vel_y, vel_z = 0.0, 0.0

        # Normalize to [-1, 1] assuming max_paddle_speed=15.0 in environment
        return np.array([vel_y / 15.0, vel_z / 15.0], dtype=np.float32)
