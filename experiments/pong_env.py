"""
Gym-compatible 3D Pong RL Environment

This environment wraps the Volley physics server with a standard Gym interface
for training RL agents to play 3D pong.
"""

import gymnasium as gym
import numpy as np
from gymnasium import spaces
from typing import Optional, Tuple, Dict, Any
import time

from test import AsyncPhysicsClient, WorldState, Vec3


class Pong3DEnv(gym.Env):
    """
    3D Pong Environment for RL training

    Observation Space:
        - Ball position (x, y, z)
        - Ball velocity (vx, vy, vz)
        - Paddle position (y, z) [x is fixed at -25]
        - Paddle velocity (vy, vz)
        Total: 10 continuous values, normalized to [-1, 1]

    Action Space:
        - Paddle velocity (vy, vz)
        2 continuous values in [-1, 1], scaled to max_speed
    """

    metadata = {"render_modes": ["human"], "render_fps": 60}

    def __init__(
        self,
        action_port: str = "tcp://127.0.0.1:5555",
        state_port: str = "tcp://127.0.0.1:5556",
        max_paddle_speed: float = 15.0,
        max_episode_steps: int = 1000,
        render_mode: Optional[str] = None,
        opponent = None,
    ):
        super().__init__()

        self.action_port = action_port
        self.state_port = state_port
        self.max_paddle_speed = max_paddle_speed
        self.max_episode_steps = max_episode_steps
        self.render_mode = render_mode
        self.opponent = opponent

        # Arena bounds for normalization
        self.arena_bounds = {
            "x": (-30.0, 30.0),
            "y": (-20.0, 20.0),
            "z": (-20.0, 20.0),
        }
        self.max_velocity = 20.0  # Reasonable max for normalization

        # Observation space: [ball_pos(3), ball_vel(3), paddle_pos(2), paddle_vel(2)]
        self.observation_space = spaces.Box(
            low=-1.0,
            high=1.0,
            shape=(10,),
            dtype=np.float32
        )

        # Action space: [paddle_vy, paddle_vz] normalized to [-1, 1]
        self.action_space = spaces.Box(
            low=-1.0,
            high=1.0,
            shape=(2,),
            dtype=np.float32
        )

        # Episode tracking
        self.episode_steps = 0
        self.last_score_player1 = 0
        self.last_score_player2 = 0
        self.prev_ball_velocity = None

        # Physics client (initialized in reset)
        self.client: Optional[AsyncPhysicsClient] = None

    def _normalize_position(self, pos: Vec3) -> np.ndarray:
        """Normalize position to [-1, 1] based on arena bounds"""
        return np.array([
            2 * (pos.x - self.arena_bounds["x"][0]) / (self.arena_bounds["x"][1] - self.arena_bounds["x"][0]) - 1,
            2 * (pos.y - self.arena_bounds["y"][0]) / (self.arena_bounds["y"][1] - self.arena_bounds["y"][0]) - 1,
            2 * (pos.z - self.arena_bounds["z"][0]) / (self.arena_bounds["z"][1] - self.arena_bounds["z"][0]) - 1,
        ], dtype=np.float32)

    def _normalize_velocity(self, vel: Vec3) -> np.ndarray:
        """Normalize velocity to [-1, 1] based on max_velocity"""
        return np.array([
            np.clip(vel.x / self.max_velocity, -1, 1),
            np.clip(vel.y / self.max_velocity, -1, 1),
            np.clip(vel.z / self.max_velocity, -1, 1),
        ], dtype=np.float32)

    def _get_observation(self, state: WorldState) -> np.ndarray:
        """Extract observation from world state"""
        # Find paddle1 (index 4) and ball (index 6)
        paddle = state.bodies[4]  # paddle1
        ball = state.bodies[6]     # ball

        # Normalize positions and velocities
        ball_pos = self._normalize_position(ball.position)
        ball_vel = self._normalize_velocity(ball.velocity)

        # Paddle only has y, z (x is fixed)
        paddle_pos = np.array([
            2 * (paddle.position.y - self.arena_bounds["y"][0]) / (self.arena_bounds["y"][1] - self.arena_bounds["y"][0]) - 1,
            2 * (paddle.position.z - self.arena_bounds["z"][0]) / (self.arena_bounds["z"][1] - self.arena_bounds["z"][0]) - 1,
        ], dtype=np.float32)

        paddle_vel = np.array([
            np.clip(paddle.velocity.y / self.max_velocity, -1, 1),
            np.clip(paddle.velocity.z / self.max_velocity, -1, 1),
        ], dtype=np.float32)

        # Concatenate all observations
        obs = np.concatenate([ball_pos, ball_vel, paddle_pos, paddle_vel])
        return obs

    def _compute_reward(self, state: WorldState, action: np.ndarray) -> float:
        """
        Compute reward for the current step

        Reward components:
        1. Distance reward: Encourage paddle to track ball's y, z position
        2. Hit bonus: Reward successful paddle-ball contact
        3. Action penalty: Small penalty for large actions (energy efficiency)
        """
        paddle = state.bodies[4]  # paddle1
        ball = state.bodies[6]     # ball

        reward = 0.0

        # 1. Distance reward (only when ball on our side or near center)
        if ball.position.x < 5.0:
            dy = abs(paddle.position.y - ball.position.y)
            dz = abs(paddle.position.z - ball.position.z)
            distance = np.sqrt(dy**2 + dz**2)
            reward += 0.1 * np.exp(-distance / 10.0)  # Exponential decay

        # 2. Hit bonus (detect ball velocity change near paddle)
        if self.prev_ball_velocity is not None:
            vel_change = abs(ball.velocity.x - self.prev_ball_velocity.x)
            if vel_change > 5.0 and ball.position.x < -20.0:  # Near paddle1
                reward += 1.0  # Reward successful hit

        self.prev_ball_velocity = ball.velocity

        # 3. Action smoothness penalty
        action_magnitude = np.sum(np.abs(action))
        reward -= 0.01 * action_magnitude

        return reward

    def reset(
        self,
        seed: Optional[int] = None,
        options: Optional[Dict[str, Any]] = None,
    ) -> Tuple[np.ndarray, Dict[str, Any]]:
        """Reset the environment"""
        super().reset(seed=seed)

        # Initialize client on first reset
        if self.client is None:
            self.client = AsyncPhysicsClient(self.action_port, self.state_port)
            # Wait for first state
            time.sleep(0.1)
            while self.client.get_latest_state() is None:
                time.sleep(0.01)

        # Reset episode tracking
        self.episode_steps = 0
        state = self.client.get_latest_state()
        if state:
            self.last_score_player1 = state.score_player1
            self.last_score_player2 = state.score_player2

        # Get initial observation
        obs = self._get_observation(state)
        info = {"state": state}

        return obs, info

    def step(self, action: np.ndarray) -> Tuple[np.ndarray, float, bool, bool, Dict[str, Any]]:
        """Execute one step in the environment"""
        self.episode_steps += 1

        # 1. RL Agent action for paddle1
        vel_y = action[0] * self.max_paddle_speed
        vel_z = action[1] * self.max_paddle_speed
        self.client.send_action("paddle1", vel_x=0.0, vel_y=float(vel_y), vel_z=float(vel_z))

        # 2. Heuristic opponent action for paddle2
        if self.opponent is not None:
            state = self.client.get_latest_state()
            if state:
                paddle2 = state.bodies[5]  # paddle2 index
                ball = state.bodies[6]      # ball index

                opp_action = self.opponent.get_action(
                    paddle_y=paddle2.position.y,
                    paddle_z=paddle2.position.z,
                    ball_y=ball.position.y,
                    ball_z=ball.position.z,
                    current_time=time.time()
                )

                opp_vel_y = opp_action[0] * self.max_paddle_speed
                opp_vel_z = opp_action[1] * self.max_paddle_speed
                self.client.send_action("paddle2", vel_x=0.0, vel_y=float(opp_vel_y), vel_z=float(opp_vel_z))

        # 3. Wait briefly for physics to update (running at 120Hz, we can go ~60Hz)
        time.sleep(1.0 / 60.0)

        # Get new state
        state = self.client.get_latest_state()
        if state is None:
            # If no state, return previous observation with zero reward
            obs = np.zeros(10, dtype=np.float32)
            return obs, 0.0, True, False, {"error": "No state received"}

        # Compute observation
        obs = self._get_observation(state)

        # Compute reward
        reward = self._compute_reward(state, action)

        # Check termination conditions
        terminated = False
        truncated = False

        # Episode ends if someone scores
        if state.score_player1 > self.last_score_player1:
            reward += 10.0  # We scored!
            terminated = True
            self.last_score_player1 = state.score_player1
        elif state.score_player2 > self.last_score_player2:
            reward -= 10.0  # Opponent scored
            terminated = True
            self.last_score_player2 = state.score_player2

        # Truncate if max steps reached
        if self.episode_steps >= self.max_episode_steps:
            truncated = True

        info = {
            "state": state,
            "episode_steps": self.episode_steps,
            "score_player1": state.score_player1,
            "score_player2": state.score_player2,
        }

        return obs, reward, terminated, truncated, info

    def close(self):
        """Clean up resources"""
        if self.client is not None:
            self.client.close()
            self.client = None


# Test the environment
if __name__ == "__main__":
    print("🎮 Testing Pong3D Environment")
    print("=" * 50)

    # Create environment
    env = Pong3DEnv()

    # Test reset
    obs, info = env.reset()
    print(f"✅ Reset successful")
    print(f"   Observation shape: {obs.shape}")
    print(f"   Observation: {obs}")

    # Test random actions
    print(f"\n🎲 Testing 100 random steps...")
    for i in range(100):
        action = env.action_space.sample()
        obs, reward, terminated, truncated, info = env.step(action)

        if i % 20 == 0:
            print(f"   Step {i}: reward={reward:.3f}, score={info['score_player1']}-{info['score_player2']}")

        if terminated or truncated:
            print(f"   Episode ended at step {i}")
            obs, info = env.reset()

    env.close()
    print(f"\n✅ Environment test completed!")
