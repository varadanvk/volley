import zmq
import struct
import time
import threading
from dataclasses import dataclass
from typing import Optional


@dataclass
class Vec3:
    x: float
    y: float
    z: float


@dataclass
class RigidBody:
    id: str
    position: Vec3
    velocity: Vec3
    # Add other fields as needed


@dataclass
class WorldState:
    bodies: list[RigidBody]
    time: float


class AsyncPhysicsClient:
    def __init__(
        self, action_port="tcp://127.0.0.1:5555", state_port="tcp://127.0.0.1:5556"
    ):
        self.context = zmq.Context()

        # PUSH socket for sending actions
        self.action_socket = self.context.socket(zmq.PUSH)
        self.action_socket.connect(action_port)

        # SUB socket for receiving state
        self.state_socket = self.context.socket(zmq.SUB)
        self.state_socket.connect(state_port)
        self.state_socket.setsockopt_string(
            zmq.SUBSCRIBE, ""
        )  # Subscribe to all messages

        self.latest_state = None
        self.running = True

        # Start background thread to receive state updates
        self.state_thread = threading.Thread(target=self._state_receiver, daemon=True)
        self.state_thread.start()

        print(f"🔗 Connected to action port: {action_port}")
        print(f"🔗 Connected to state port: {state_port}")

    def _state_receiver(self):
        """Background thread to continuously receive state updates"""
        while self.running:
            try:
                # Receive state (blocking with timeout)
                raw_state = self.state_socket.recv(zmq.DONTWAIT)

                # For now, just store raw bytes (you'd parse this into WorldState)
                self.latest_state = raw_state
                print(f"📡 Received state update: {len(raw_state)} bytes")

            except zmq.Again:
                # No message available
                continue
            except Exception as e:
                print(f"❌ Error receiving state: {e}")
                time.sleep(0.1)

    def send_action(
        self, body_id: str, vel_x: float = 0.0, vel_y: float = 0.0, vel_z: float = 0.0
    ):
        """Send action to move a body (non-blocking)"""
        try:
            # Create PostAction command (1) + action data
            command_byte = struct.pack("B", 1)  # 1 = PostAction

            # For now, send simple action format (you'll need to match your Action struct)
            # This is a simplified version - adjust based on your actual Action struct
            action_data = struct.pack(
                "20s fff",
                body_id.encode().ljust(20, b"\0"),  # body_id (padded)
                vel_x,
                vel_y,
                vel_z,
            )  # velocity

            # Send command type first, then action data
            self.action_socket.send(command_byte + action_data, zmq.DONTWAIT)
            print(f"🎮 Sent action: {body_id} -> velocity({vel_x}, {vel_y}, {vel_z})")

        except Exception as e:
            print(f"❌ Error sending action: {e}")

    def get_latest_state(self) -> Optional[bytes]:
        """Get the most recent state (non-blocking)"""
        return self.latest_state

    def close(self):
        """Clean shutdown"""
        self.running = False
        self.state_thread.join(timeout=1)
        self.action_socket.close()
        self.state_socket.close()
        self.context.term()


def test_basic_communication():
    """Test basic send/receive"""
    print("🚀 Starting basic communication test...")

    client = AsyncPhysicsClient()

    # Wait a moment for initial state
    time.sleep(1)

    # Check if we received initial state
    state = client.get_latest_state()
    if state:
        print(f"✅ Initial state received: {len(state)} bytes")
    else:
        print("⚠️  No initial state received")

    # Send some actions
    print("\n🎮 Sending test actions...")

    # Move paddle1 up
    client.send_action("paddle1", vel_y=5.0)
    time.sleep(0.1)

    # Move paddle1 right
    client.send_action("paddle1", vel_z=3.0)
    time.sleep(0.1)

    # Move ball
    client.send_action("ball", vel_x=8.0, vel_y=2.0)
    time.sleep(0.1)

    # Wait for state updates
    time.sleep(2)

    client.close()
    print("✅ Test completed!")


def test_rl_simulation():
    """Simulate RL training loop"""
    print("🤖 Starting RL simulation test...")

    client = AsyncPhysicsClient()

    # Wait for initial state
    time.sleep(1)

    # Simulate RL training loop
    for episode in range(5):
        print(f"\n📈 Episode {episode + 1}")

        for step in range(20):
            # Get current state
            state = client.get_latest_state()
            if state:
                # Simulate RL decision (random actions)
                import random

                action_y = random.uniform(-5.0, 5.0)
                action_z = random.uniform(-3.0, 3.0)

                # Send action
                client.send_action("paddle1", vel_y=action_y, vel_z=action_z)

                if step % 5 == 0:
                    print(f"  Step {step}: action({action_y:.1f}, {action_z:.1f})")

            # High frequency (100Hz)
            time.sleep(0.01)

    client.close()
    print("🤖 RL simulation completed!")


if __name__ == "__main__":
    print("🧪 ZeroMQ Async Physics Test")
    print("=" * 50)

    try:
        # Run basic test
        test_basic_communication()

        time.sleep(2)

        # Run RL simulation
        test_rl_simulation()

    except KeyboardInterrupt:
        print("\n⏹️  Test interrupted by user")
    except Exception as e:
        print(f"❌ Test failed: {e}")
