import zmq
import time
import threading
from dataclasses import dataclass
from typing import Optional
import msgpack


@dataclass
class Vec3:
    x: float
    y: float
    z: float

    def to_dict(self):
        return {"x": self.x, "y": self.y, "z": self.z}

    @classmethod
    def from_dict(cls, d):
        return cls(x=d["x"], y=d["y"], z=d["z"])


@dataclass
class AABB:
    min: Vec3
    max: Vec3

    def to_dict(self):
        return {"min": self.min.to_dict(), "max": self.max.to_dict()}

    @classmethod
    def from_dict(cls, d):
        return cls(min=Vec3.from_dict(d["min"]), max=Vec3.from_dict(d["max"]))

    @staticmethod
    def from_center_size(center: Vec3, half_extents: Vec3):
        """Create AABB from center and half extents"""
        return AABB(
            min=Vec3(
                center.x - half_extents.x,
                center.y - half_extents.y,
                center.z - half_extents.z,
            ),
            max=Vec3(
                center.x + half_extents.x,
                center.y + half_extents.y,
                center.z + half_extents.z,
            ),
        )


@dataclass
class RigidBody:
    id: str
    position: Vec3
    velocity: Vec3
    dynamic: bool
    aabb: AABB
    mass: float
    restitution: float

    def to_dict(self):
        return {
            "id": self.id,
            "position": self.position.to_dict(),
            "velocity": self.velocity.to_dict(),
            "dynamic": self.dynamic,
            "aabb": self.aabb.to_dict(),
            "mass": self.mass,
            "restitution": self.restitution,
        }

    @classmethod
    def from_dict(cls, d):
        return cls(
            id=d["id"],
            position=Vec3.from_dict(d["position"]),
            velocity=Vec3.from_dict(d["velocity"]),
            dynamic=d["dynamic"],
            aabb=AABB.from_dict(d["aabb"]),
            mass=d["mass"],
            restitution=d["restitution"],
        )


@dataclass
class Action:
    body_id: str
    velocity: Vec3
    position: Vec3
    aabb: AABB
    mass: float
    restitution: float
    dynamic: bool

    def to_dict(self):
        return {
            "body_id": self.body_id,
            "velocity": self.velocity.to_dict(),
            "position": self.position.to_dict(),
            "aabb": self.aabb.to_dict(),
            "mass": self.mass,
            "restitution": self.restitution,
            "dynamic": self.dynamic,
        }

    def to_msgpack(self):
        """Serialize Action to MessagePack bytes"""
        return msgpack.packb(self.to_dict())


@dataclass
class WorldState:
    bodies: list[RigidBody]
    time: float
    score_player1: int
    score_player2: int

    @classmethod
    def from_msgpack(cls, data: bytes):
        """Deserialize MessagePack WorldState from Rust"""
        try:
            decoded = msgpack.unpackb(data, raw=False)

            # Debug: print structure of first decode attempt
            if not hasattr(cls, '_debug_printed'):
                print(f"\nDEBUG: Successfully unpacked MessagePack")
                print(f"DEBUG: decoded type: {type(decoded)}")
                if isinstance(decoded, dict):
                    print(f"DEBUG: decoded keys: {decoded.keys()}")
                    if "bodies" in decoded:
                        print(f"DEBUG: bodies type: {type(decoded['bodies'])}")
                        if len(decoded['bodies']) > 0:
                            print(f"DEBUG: first body type: {type(decoded['bodies'][0])}")
                            print(f"DEBUG: first body: {decoded['bodies'][0]}")
                elif isinstance(decoded, list):
                    print(f"DEBUG: decoded is a list with {len(decoded)} elements")
                    print(f"DEBUG: first element: {decoded[0] if decoded else 'empty'}")
                else:
                    print(f"DEBUG: decoded content: {decoded}")
                cls._debug_printed = True

            bodies = [RigidBody.from_dict(b) for b in decoded["bodies"]]
            return cls(
                bodies=bodies,
                time=decoded["time"],
                score_player1=decoded["score_player1"],
                score_player2=decoded["score_player2"],
            )
        except Exception as e:
            print(f"\nDEBUG ERROR: {e}")
            print(f"DEBUG: data length: {len(data)} bytes")
            print(f"DEBUG: first 50 bytes: {data[:50]}")
            raise


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
        state_count = 0
        while self.running:
            try:
                # Receive state (non-blocking)
                raw_state = self.state_socket.recv(zmq.DONTWAIT)

                # Parse WorldState from MessagePack
                try:
                    self.latest_state = WorldState.from_msgpack(raw_state)
                    state_count += 1
                    # Print only occasionally to avoid spam
                    if state_count % 60 == 0:  # Every 60 updates (~1 sec at 60Hz)
                        ball = self.latest_state.bodies[6]  # Ball is at index 6
                        print(f"📡 State #{state_count}: {len(self.latest_state.bodies)} bodies, "
                              f"score {self.latest_state.score_player1}-{self.latest_state.score_player2}, "
                              f"ball pos: ({ball.position.x:.1f}, {ball.position.y:.1f}, {ball.position.z:.1f})")
                except Exception as e:
                    print(f"⚠️  Failed to parse state: {e}")
                    self.latest_state = None

            except zmq.Again:
                # No message available - small sleep to avoid busy-waiting
                time.sleep(0.001)
                continue
            except Exception as e:
                print(f"❌ Error receiving state: {e}")
                time.sleep(0.1)

    def send_action(
        self, body_id: str, vel_x: float = 0.0, vel_y: float = 0.0, vel_z: float = 0.0
    ):
        """Send action to move a body (non-blocking)"""
        try:
            # Get current body state from latest world state
            if self.latest_state is None:
                print("⚠️  No state available yet, cannot send action")
                return

            # Find the body in the current state
            body = next((b for b in self.latest_state.bodies if b.id == body_id), None)
            if body is None:
                print(f"⚠️  Body '{body_id}' not found in world state")
                return

            # Create Action with updated velocity but keeping other properties
            action = Action(
                body_id=body_id,
                velocity=Vec3(vel_x, vel_y, vel_z),
                position=body.position,
                aabb=body.aabb,
                mass=body.mass,
                restitution=body.restitution,
                dynamic=body.dynamic,
            )

            # Serialize to MessagePack and send
            action_bytes = action.to_msgpack()
            self.action_socket.send(action_bytes, zmq.DONTWAIT)
            # Only print occasionally to avoid spam
            if not hasattr(self, '_action_count'):
                self._action_count = 0
                print(f"🎮 Sent action: {body_id} -> velocity({vel_x}, {vel_y}, {vel_z})")
            self._action_count += 1

        except Exception as e:
            print(f"❌ Error sending action: {e}")

    def get_latest_state(self) -> Optional[WorldState]:
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
    # client.send_action("paddle1", vel_y=5.0)
    # time.sleep(0.1)

    # # Move paddle1 right
    # client.send_action("paddle1", vel_z=3.0)
    # time.sleep(0.1)

    # Move ball
    client.send_action("ball", vel_x=100.0, vel_y=100.0)
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
        # test_basic_communication()

        time.sleep(2)

        # Run RL simulation
        test_rl_simulation()

    except KeyboardInterrupt:
        print("\n⏹️  Test interrupted by user")
    except Exception as e:
        print(f"❌ Test failed: {e}")
