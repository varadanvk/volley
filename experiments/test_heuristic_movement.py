#!/usr/bin/env python3
"""
Diagnostic script to verify heuristic opponent is moving correctly

This shows:
1. Heuristic target tracking
2. Opponent paddle movement
3. Comparison with ball position
"""

import time
import numpy as np
from pong_env import Pong3DEnv
from heuristic_opponent import HeuristicOpponent


def test_heuristic_tracking():
    """Test that heuristic opponent properly tracks the ball"""
    print("🧪 Heuristic Opponent Movement Test")
    print("=" * 70)

    # Create opponent with NO noise for clear tracking
    opponent = HeuristicOpponent(max_speed=12.0, reaction_delay=0.0, tracking_noise=0.0)
    print("✅ Created heuristic with zero noise for clear tracking")

    # Create environment
    env = Pong3DEnv(opponent=opponent)
    print("✅ Connected to physics server\n")

    # Reset
    obs, _ = env.reset()

    print("📊 Tracking Ball Position vs Paddle Target")
    print("-" * 70)
    print(f"{'Step':>4} | {'Ball Y':>8} | {'Ball Z':>8} | {'Paddle2 Y':>10} | {'Paddle2 Z':>10} | {'Tracking?':>12}")
    print("-" * 70)

    # Run steps and observe
    for step in range(30):
        # Get current state before action
        state = env.client.get_latest_state()

        if state:
            ball = state.bodies[6]
            paddle2 = state.bodies[5]

            # Get opponent action to see target
            opp_action = opponent.get_action(
                paddle_y=paddle2.position.y,
                paddle_z=paddle2.position.z,
                ball_y=ball.position.y,
                ball_z=ball.position.z,
                current_time=time.time()
            )

            # Check if paddle is moving toward ball
            ball_y = ball.position.y
            ball_z = ball.position.z
            paddle_y = paddle2.position.y
            paddle_z = paddle2.position.z

            dy = ball_y - paddle_y
            dz = ball_z - paddle_z
            dist = np.sqrt(dy**2 + dz**2)

            # Predict next paddle position based on action
            vel_y = opp_action[0] * 15.0  # Scale back to velocity
            vel_z = opp_action[1] * 15.0
            next_paddle_y = paddle_y + vel_y * 0.016  # ~60Hz step
            next_paddle_z = paddle_z + vel_z * 0.016

            # Check if moving closer
            next_dy = ball_y - next_paddle_y
            next_dz = ball_z - next_paddle_z
            next_dist = np.sqrt(next_dy**2 + next_dz**2)

            tracking_status = "✓ TRACKING" if next_dist < dist else "✗ NOT TRACKING"

            print(f"{step:4d} | {ball_y:8.2f} | {ball_z:8.2f} | "
                  f"{paddle_y:10.2f} | {paddle_z:10.2f} | {tracking_status:>12}")

        # Random action for agent
        action = np.array([0.5, -0.3])  # Some movement
        obs, reward, terminated, truncated, info = env.step(action)

        if terminated or truncated:
            print(f"\n   Episode ended at step {step}")
            break

    env.close()
    print("\n✅ Test complete!")


def test_opponent_difficulty_levels():
    """Test different difficulty levels"""
    print("\n\n🎯 Testing Opponent Difficulty Levels")
    print("=" * 70)

    difficulties = {
        "Easy": {"max_speed": 10.0, "reaction_delay": 0.1, "tracking_noise": 0.5},
        "Medium": {"max_speed": 12.0, "reaction_delay": 0.05, "tracking_noise": 0.3},
        "Hard": {"max_speed": 14.0, "reaction_delay": 0.02, "tracking_noise": 0.1},
    }

    env = Pong3DEnv()

    for difficulty_name, config in difficulties.items():
        opponent = HeuristicOpponent(**config)
        print(f"\n📊 {difficulty_name} Difficulty")
        print(f"   Max Speed: {config['max_speed']}")
        print(f"   Reaction Delay: {config['reaction_delay']:.2f}s")
        print(f"   Tracking Noise: σ={config['tracking_noise']}")

        # Test a few actions
        env.opponent = opponent
        obs, _ = env.reset()

        total_distance = 0
        num_steps = 20

        for _ in range(num_steps):
            action = np.array([0.3, -0.2])
            state = env.client.get_latest_state()

            if state:
                ball = state.bodies[6]
                paddle2 = state.bodies[5]

                dy = abs(ball.position.y - paddle2.position.y)
                dz = abs(ball.position.z - paddle2.position.z)
                dist = np.sqrt(dy**2 + dz**2)
                total_distance += dist

            obs, _, term, trunc, _ = env.step(action)
            if term or trunc:
                break

        avg_distance = total_distance / num_steps
        print(f"   Avg Distance to Ball: {avg_distance:.2f}")
        print(f"   → {'Easier (larger distance)' if avg_distance > 8 else 'Harder (closer tracking)'}")

    env.close()
    print("\n✅ Difficulty test complete!")


if __name__ == "__main__":
    test_heuristic_tracking()
    test_opponent_difficulty_levels()
