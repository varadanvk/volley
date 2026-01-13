# 3D Pong PPO Training - Quick Start Guide

## Overview

This training setup allows you to train a PPO (Proximal Policy Optimization) agent to play 3D pong against a heuristic opponent using Stable-Baselines3.

## Files Created

### Core Files
- **`heuristic_opponent.py`**: Ball-tracking opponent with configurable difficulty
  - Max speed: 12.0 (vs agent's 15.0) for fair challenge
  - Reaction delay: 0.05s to simulate human response
  - Tracking noise: 0.3 standard deviation for occasional misses

- **`pong_env.py`** (Modified): Gym environment with opponent integration
  - Dual paddle control (RL agent on paddle1, heuristic on paddle2)
  - Dense reward function with distance, hit, and action smoothness components
  - 10D observation space: ball position/velocity + paddle position/velocity
  - 2D action space: paddle y and z velocity

- **`train_ppo.py`**: Main training script
  - PPO agent with 2-layer 256-unit MLPs
  - Parallel environment support (default: 1 env, easily scalable)
  - Checkpointing every 10K steps
  - Evaluation every 5K steps with best model saving
  - TensorBoard logging for monitoring

- **`evaluate_agent.py`**: Evaluation and testing script
  - Test trained models against different opponent difficulties
  - Win rate calculation
  - Reward statistics and performance metrics
  - Easy/medium/hard opponent presets

## Quick Start

### 1. Prerequisites

✅ Dependencies installed:
```bash
uv pip install stable-baselines3 tensorboard torch
```

✅ Rust physics server running:
```bash
cd /Users/varadan/Documents/programs/volley
cargo run --release
```
(You can run this in a separate terminal)

### 2. Start Training

#### Option A: Quick Test (100K steps, ~1 hour on CPU)
```bash
cd experiments
python train_ppo.py --quick --single-env
```

#### Option B: Full Training (1M steps, ~8-12 hours on CPU)
```bash
cd experiments
python train_ppo.py --single-env
```

#### Option C: Custom Configuration
```bash
python train_ppo.py --total-steps 500000 --single-env
```

### 3. Monitor Training

In a separate terminal, watch TensorBoard:
```bash
cd /Users/varadan/Documents/programs/volley/experiments
tensorboard --logdir ./tensorboard_logs/
```

Open http://localhost:6006 to see:
- `rollout/ep_rew_mean` - Average episode reward (should increase over time)
- `rollout/ep_len_mean` - Average episode length
- `train/policy_loss` - Policy gradient loss
- `train/value_loss` - Value function loss

### 4. Evaluate Trained Model

After training, test your model:

```bash
cd experiments

# Test on medium difficulty (balanced)
python evaluate_agent.py --episodes 20

# Test on hard difficulty
python evaluate_agent.py --difficulty hard --episodes 20

# Use custom model path
python evaluate_agent.py --model ./checkpoints/ppo_pong_200000_steps.zip
```

## Training Phases (Expected Progression)

### Phase 1: Initial Learning (0-100K steps)
- Agent learns basic ball tracking
- Discovers that moving towards the ball yields rewards
- Reward gradually increases from ~-2 to ~-1

### Phase 2: Skill Development (100K-500K steps)
- Agent learns to position paddle ahead of ball
- Starts successfully hitting the ball back
- Begins trading goals with opponent
- Reward improves towards ~0

### Phase 3: Strategic Play (500K-1M steps)
- Agent develops consistent defensive play
- Learns to anticipate ball trajectory
- Win rate against heuristic reaches 50%+
- Reward stabilizes positive

## Architecture

```
┌─────────────────────────────────────┐
│      Python Training Loop           │
│  (PPO with Stable-Baselines3)       │
└────────┬────────────────────────────┘
         │ Actions (PUSH)
         │ State (SUB)
┌────────▼────────────────────────────┐
│     Rust Physics Server (120Hz)     │
│   - 2 Paddles                       │
│   - 1 Ball                          │
│   - 4 Walls                         │
│   - Physics simulation              │
└─────────────────────────────────────┘
```

**Communication:**
- Action Channel (port 5555): PUSH socket, agent sends paddle commands
- State Channel (port 5556): PUB socket, server broadcasts world state at 60Hz
- Serialization: MessagePack (efficient binary format)

## Reward Function

```
Total Reward = Distance Reward + Hit Bonus - Action Penalty + Scoring Reward

Distance Reward:    0.1 * exp(-distance/10) when ball on our side
Hit Bonus:          +1.0 when paddle hits ball (velocity change > 5)
Action Penalty:     -0.01 * |action_magnitude|
Scoring Reward:     +10 for goal, -10 for opponent goal (built-in)
```

The reward is designed to encourage:
1. **Tracking**: Being close to the ball when it's near
2. **Hitting**: Successfully returning the ball
3. **Efficiency**: Smooth, not jerky movements

## Outputs

After training, you'll find:

```
experiments/
├── checkpoints/           # Periodic checkpoints every 10K steps
│   ├── ppo_pong_10000_steps.zip
│   ├── ppo_pong_20000_steps.zip
│   └── ...
├── best_model/           # Best model based on eval performance
│   └── best_model.zip
├── final_model/          # Final trained model
│   └── ppo_pong_final.zip
├── tensorboard_logs/     # TensorBoard event files
└── eval_logs/            # Evaluation statistics
```

## Tuning Tips

### If agent learns too slowly:
- Increase distance reward weight (currently 0.1)
- Increase hit bonus (currently 1.0)
- Reduce action penalty (currently 0.01)

### If agent is too jerky:
- Increase action penalty
- Reduce learning rate (currently 3e-4)

### If agent plateaus early:
- Increase opponent difficulty in `train_ppo.py`
- Reduce entropy coefficient to encourage exploitation

### If training is unstable:
- Reduce learning rate to 1e-4
- Increase batch size to 128
- Reduce clip range to 0.15

## Next Steps

After getting the basic training working:

1. **Curriculum Learning**: Start with easy opponent, gradually increase difficulty
2. **Self-Play**: Train two agents against each other
3. **Hyperparameter Optimization**: Use Optuna for automatic HPO
4. **Visualization**: Add rendering to watch agent play
5. **Multi-Objective Rewards**: Separate offensive vs defensive rewards

## Troubleshooting

### "Connection refused" or "No state received"
- Make sure Rust server is running: `cargo run --release`
- Check server is outputting state messages in its terminal

### Model not improving
- Check TensorBoard to see if policy loss is decreasing
- Verify reward signals are being computed (not always 0)
- Try increasing distance reward weight

### Training crashes with OOM
- Reduce batch size from 64 to 32
- Use single environment instead of parallel (--single-env)

### Slow training on CPU
- This is normal for CPU training (~100K-200K steps/hour)
- GPU training would be 3-4x faster

## Files Reference

```
📁 experiments/
├── 📄 pong_env.py              - Gym environment (modified)
├── 📄 test.py                  - Physics client utilities
├── 📄 heuristic_opponent.py    - Opponent controller (NEW)
├── 📄 train_ppo.py             - Training script (NEW)
├── 📄 evaluate_agent.py        - Evaluation script (NEW)
├── 📄 TRAINING_GUIDE.md        - This file (NEW)
├── 📁 checkpoints/             - Model checkpoints
├── 📁 best_model/              - Best trained model
├── 📁 final_model/             - Final model
├── 📁 tensorboard_logs/        - TensorBoard data
└── 📁 eval_logs/               - Evaluation logs
```

## Key Insights

`★ Insight ─────────────────────────────────────`
**Why ZeroMQ for IPC?** The server uses ZeroMQ's PUB/SUB and PUSH/PULL patterns which are designed for efficient message passing. PUB/SUB allows the server to broadcast state to multiple subscribers, and PUSH/PULL provides load-balanced action delivery. This is much more efficient than REST API calls.

**Dense Rewards Matter:** Pong is naturally a sparse reward problem (only +1/-1 when scoring). Without dense rewards (distance, hits), the agent would need millions of samples to stumble upon good behavior. The exponential distance reward creates a smooth learning landscape.

**Reaction Delay as Regularization:** The 50ms reaction delay on the heuristic opponent isn't just for realism—it actually makes the opponent harder to beat because a perfect tracker would be impossible for the RL agent to outplay. The intentional limitations create a calibrated difficulty level.
`─────────────────────────────────────────────────`

## Questions?

- Check the plan file: `/Users/varadan/.claude/plans/pure-watching-matsumoto.md`
- Review implementation: See comments in source files
- Debug: Monitor server output and TensorBoard metrics
