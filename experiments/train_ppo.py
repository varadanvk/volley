#!/usr/bin/env python3
"""
Main training script for 3D Pong with PPO and Stable-Baselines3

Trains a PPO agent to play 3D pong against a heuristic opponent using parallel environments.
Includes checkpointing, evaluation, and TensorBoard logging.
"""

import argparse
import numpy as np
from stable_baselines3 import PPO
from stable_baselines3.common.vec_env import SubprocVecEnv, DummyVecEnv
from stable_baselines3.common.callbacks import CheckpointCallback, EvalCallback
from stable_baselines3.common.monitor import Monitor

from pong_env import Pong3DEnv
from heuristic_opponent import HeuristicOpponent


def make_env(rank, seed=0):
    """
    Factory function for creating environments

    Args:
        rank: Environment index (for process-based vectorization)
        seed: Random seed

    Returns:
        Callable that creates a Pong3DEnv with heuristic opponent
    """

    def _init():
        # Create heuristic opponent with configurable difficulty
        opponent = HeuristicOpponent(
            max_speed=12.0, reaction_delay=0.05, tracking_noise=0.3
        )

        # Create environment with opponent
        env = Pong3DEnv(opponent=opponent)

        # Wrap with Monitor for logging episode stats
        env = Monitor(env)

        # Reset with unique seed
        env.reset(seed=seed + rank)

        return env

    return _init


def train(total_steps=1_000_000, num_envs=1, use_single_env=True):
    """
    Train PPO model against heuristic opponent

    Args:
        total_steps: Total timesteps to train for
        num_envs: Number of parallel environments
        use_single_env: If True, use DummyVecEnv instead of SubprocVecEnv (simpler for testing)
    """
    print("🚀 Starting PPO Training")
    print(f"   Total steps: {total_steps:,}")
    print(f"   Parallel envs: {num_envs}")
    print(f"   Single env mode: {use_single_env}")
    print("=" * 60)

    # Create vectorized environment
    if use_single_env or num_envs == 1:
        print("📦 Creating single environment (DummyVecEnv)...")
        env = DummyVecEnv([make_env(0)])
    else:
        print(f"📦 Creating {num_envs} parallel environments (SubprocVecEnv)...")
        env = SubprocVecEnv([make_env(i) for i in range(num_envs)])

    # Create evaluation environment
    eval_env = DummyVecEnv([make_env(999)])

    # Configure PPO hyperparameters
    ppo_config = {
        "learning_rate": 3e-4,
        "n_steps": 2048,  # Steps per update
        "batch_size": 64,
        "n_epochs": 10,  # Epochs per update
        "gamma": 0.99,  # Discount factor
        "gae_lambda": 0.95,  # GAE lambda
        "clip_range": 0.2,  # PPO clipping range
        "ent_coef": 0.01,  # Entropy coefficient (exploration)
        "vf_coef": 0.5,  # Value function coefficient
        "max_grad_norm": 0.5,
        "policy_kwargs": dict(
            net_arch=[dict(pi=[256, 256], vf=[256, 256])]  # 2-layer 256-unit MLPs
        ),
        "verbose": 1,
        "tensorboard_log": "./tensorboard_logs/",
    }

    print("\n⚙️  PPO Configuration:")
    for key, value in ppo_config.items():
        if key != "policy_kwargs":
            print(f"   {key}: {value}")
    print("   policy_kwargs: 2x [256, 256] MLPs")

    # Create callbacks
    print("\n📊 Setting up callbacks...")

    checkpoint_callback = CheckpointCallback(
        save_freq=10000, save_path="./checkpoints/", name_prefix="ppo_pong"
    )
    print("   ✅ Checkpoint: Save every 10K steps to ./checkpoints/")

    eval_callback = EvalCallback(
        eval_env,
        best_model_save_path="./best_model/",
        log_path="./eval_logs/",
        eval_freq=5000,
        deterministic=True,
        render=False,
    )
    print("   ✅ Evaluation: Test every 5K steps, save best to ./best_model/")

    # Initialize PPO model
    print("\n🤖 Initializing PPO model...")
    model = PPO("MlpPolicy", env, **ppo_config)
    print("   ✅ Model created with MlpPolicy")

    # Train
    print("\n🎓 Starting training...")
    print("   View training progress with: tensorboard --logdir ./tensorboard_logs/")
    print("=" * 60)

    try:
        model.learn(
            total_timesteps=total_steps,
            callback=[checkpoint_callback, eval_callback],
            progress_bar=True,
        )
    except KeyboardInterrupt:
        print("\n⏹️  Training interrupted by user")

    # Save final model
    print("\n💾 Saving final model...")
    model.save("./final_model/ppo_pong_final")
    print("   ✅ Model saved to ./final_model/ppo_pong_final.zip")

    # Cleanup
    env.close()
    eval_env.close()

    print("\n✅ Training complete!")
    print("   Best model: ./best_model/best_model.zip")
    print("   Final model: ./final_model/ppo_pong_final.zip")
    print("   Checkpoints: ./checkpoints/")
    print("\n   Next: Run 'python evaluate_agent.py' to test the model")


def main():
    parser = argparse.ArgumentParser(description="Train PPO for 3D Pong")
    parser.add_argument(
        "--total-steps",
        type=int,
        default=1_000_000,
        help="Total training steps (default: 1,000,000)",
    )
    parser.add_argument(
        "--num-envs",
        type=int,
        default=1,
        help="Number of parallel environments (default: 1)",
    )
    parser.add_argument(
        "--single-env",
        action="store_true",
        help="Use single environment mode (DummyVecEnv) instead of parallel",
    )
    parser.add_argument(
        "--quick", action="store_true", help="Quick test run with 100K steps"
    )

    args = parser.parse_args()

    total_steps = 10_000 if args.quick else args.total_steps
    num_envs = 1 if args.single_env else args.num_envs

    train(total_steps=total_steps, num_envs=num_envs, use_single_env=args.single_env)


if __name__ == "__main__":
    main()
