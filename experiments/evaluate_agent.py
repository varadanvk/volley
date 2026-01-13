#!/usr/bin/env python3
"""
Evaluation script for trained PPO model

Tests the trained model's performance against the heuristic opponent and reports:
- Win rate
- Average reward
- Score statistics
"""

import argparse
import numpy as np
from stable_baselines3 import PPO

from pong_env import Pong3DEnv
from heuristic_opponent import HeuristicOpponent


def evaluate(
    model_path="./final_model/final_model.zip",
    num_episodes=20,
    opponent_difficulty="medium",
):
    """
    Evaluate a trained PPO model

    Args:
        model_path: Path to the trained model (default: best model)
        num_episodes: Number of episodes to run (default: 20)
        opponent_difficulty: "easy", "medium", or "hard"
    """
    print("🎮 Evaluating Trained Model")
    print("=" * 60)

    # Configure opponent difficulty
    difficulty_config = {
        "easy": {"max_speed": 10.0, "reaction_delay": 0.1, "tracking_noise": 0.5},
        "medium": {"max_speed": 12.0, "reaction_delay": 0.05, "tracking_noise": 0.3},
        "hard": {"max_speed": 14.0, "reaction_delay": 0.02, "tracking_noise": 0.1},
    }

    if opponent_difficulty not in difficulty_config:
        print(f"❌ Unknown difficulty: {opponent_difficulty}")
        return

    config = difficulty_config[opponent_difficulty]
    print(f"📊 Test Configuration:")
    print(f"   Opponent difficulty: {opponent_difficulty}")
    print(f"   Opponent max speed: {config['max_speed']}")
    print(f"   Opponent reaction delay: {config['reaction_delay']}")
    print(f"   Opponent tracking noise: {config['tracking_noise']}")
    print(f"   Episodes: {num_episodes}")
    print("=" * 60)

    # Create opponent and environment
    try:
        print(f"\n📦 Loading model from: {model_path}")
        model = PPO.load(model_path)
        print("   ✅ Model loaded successfully")
    except FileNotFoundError:
        print(f"   ❌ Model not found at {model_path}")
        return

    opponent = HeuristicOpponent(**config)
    env = Pong3DEnv(opponent=opponent)

    # Run evaluation episodes
    print(f"\n🎮 Running {num_episodes} evaluation episodes...\n")

    wins = 0
    losses = 0
    ties = 0
    total_reward = 0.0
    total_agent_score = 0
    total_opponent_score = 0

    episode_rewards = []
    episode_score_diffs = []

    for ep in range(num_episodes):
        obs, info = env.reset()
        done = False
        ep_reward = 0
        step_count = 0

        while not done:
            action, _states = model.predict(obs, deterministic=True)
            obs, reward, terminated, truncated, info = env.step(action)
            ep_reward += reward
            done = terminated or truncated
            step_count += 1

        agent_score = info["score_player1"]
        opponent_score = info["score_player2"]
        score_diff = agent_score - opponent_score

        episode_rewards.append(ep_reward)
        episode_score_diffs.append(score_diff)
        total_reward += ep_reward
        total_agent_score += agent_score
        total_opponent_score += opponent_score

        # Track wins/losses/ties
        if agent_score > opponent_score:
            wins += 1
            status = "✅ WIN"
        elif agent_score < opponent_score:
            losses += 1
            status = "❌ LOSS"
        else:
            ties += 1
            status = "⚪ TIE"

        print(
            f"Ep {ep + 1:2d}: {status} | Score {agent_score}-{opponent_score} | "
            f"Reward {ep_reward:7.2f} | Steps {step_count}"
        )

    # Summary statistics
    print("\n" + "=" * 60)
    print("📊 EVALUATION SUMMARY")
    print("=" * 60)

    win_rate = 100 * wins / num_episodes
    loss_rate = 100 * losses / num_episodes
    tie_rate = 100 * ties / num_episodes
    avg_reward = total_reward / num_episodes
    avg_agent_score = total_agent_score / num_episodes
    avg_opponent_score = total_opponent_score / num_episodes
    avg_score_diff = np.mean(episode_score_diffs)

    print(f"\n📈 Results:")
    print(f"   Win Rate:          {wins:2d}/{num_episodes} episodes ({win_rate:5.1f}%)")
    print(
        f"   Loss Rate:         {losses:2d}/{num_episodes} episodes ({loss_rate:5.1f}%)"
    )
    print(f"   Tie Rate:          {ties:2d}/{num_episodes} episodes ({tie_rate:5.1f}%)")

    print(f"\n🎯 Scoring:")
    print(f"   Average Agent Score:     {avg_agent_score:.2f}")
    print(f"   Average Opponent Score:  {avg_opponent_score:.2f}")
    print(f"   Average Score Difference: {avg_score_diff:+.2f}")

    print(f"\n🏆 Rewards:")
    print(f"   Average Episode Reward:  {avg_reward:.4f}")
    print(f"   Max Episode Reward:      {np.max(episode_rewards):.4f}")
    print(f"   Min Episode Reward:      {np.min(episode_rewards):.4f}")
    print(f"   Std Dev:                 {np.std(episode_rewards):.4f}")

    # Performance assessment
    print(f"\n💬 Assessment:")
    if win_rate >= 70:
        print(f"   🌟 Excellent! Agent dominates the opponent.")
    elif win_rate >= 55:
        print(f"   ⭐ Good performance. Agent beats opponent most of the time.")
    elif win_rate >= 45:
        print(f"   👍 Reasonable performance. Agent is competitive.")
    elif win_rate >= 30:
        print(f"   📈 Learning in progress. Agent shows some skill.")
    else:
        print(f"   ❌ Agent needs more training.")

    env.close()

    print("\n✅ Evaluation complete!")


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate trained PPO model for 3D Pong"
    )
    parser.add_argument(
        "--model",
        type=str,
        default="./best_model/best_model.zip",
        help="Path to trained model (default: ./best_model/best_model.zip)",
    )
    parser.add_argument(
        "--episodes",
        type=int,
        default=20,
        help="Number of evaluation episodes (default: 20)",
    )
    parser.add_argument(
        "--difficulty",
        type=str,
        default="medium",
        choices=["easy", "medium", "hard"],
        help="Opponent difficulty level (default: medium)",
    )

    args = parser.parse_args()

    evaluate(
        model_path=args.model,
        num_episodes=args.episodes,
        opponent_difficulty=args.difficulty,
    )


if __name__ == "__main__":
    main()
