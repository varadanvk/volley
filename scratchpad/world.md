# 3D Pong World Object Components

## Rigidbody Storage

The world needs to own and manage all physics objects in the game. This includes the ball, two paddles, and court walls. Each rigidbody has position, velocity, mass, and collision bounds.

**Key considerations:**

- Use a Vec<RigidBody> or similar collection
- Each body needs a unique ID for collision pairs
- Different body types (ball, paddle, wall) might need different update behaviors

## Physics Step Execution

The world runs the main physics loop each frame. This involves integrating all bodies (updating positions from velocities) and then resolving any collisions that occurred.

**Key considerations:**

- Fixed timestep vs variable timestep (fixed is more stable)
- Integration order matters - position first, then collision resolution
- May need multiple substeps for fast-moving objects

## Collision Detection System

The world checks every possible pair of bodies for collisions. For pong this is simple since you only have a few objects, but the system needs to be general enough to handle all combinations.

**Key considerations:**

- Broad phase (which objects might collide) vs narrow phase (exact collision math)
- Collision filtering - walls don't collide with each other
- Early exit optimizations for non-colliding pairs

## Game State Management

Beyond physics, the world tracks game-specific state like score, win conditions, and ball respawn logic. This bridges the gap between raw physics and actual game rules.

**Key considerations:**

- Score tracking when ball exits court bounds
- Ball reset position and initial velocity
- Game over conditions and restart logic
- Paddle movement constraints (staying within court bounds)

## Graphics Interface

The world provides a clean interface for the rendering system to query current object states. This keeps physics and graphics decoupled while providing necessary data.

**Key considerations:**

- Iterator over all renderable objects
- Transformation matrices for each object
- Game UI state (score display, game over screen)
- Interpolation data for smooth rendering between physics steps

## Event System

The world can generate events for important game moments like collisions, scoring, or game state changes. This allows other systems (audio, particles, UI) to react without tight coupling.

**Key considerations:**

- Collision events with impact data (for sound effects)
- Scoring events with player information
- Game state transition events
- Event queuing and dispatch mechanism

## Time Management

The world handles timing concerns like maintaining consistent physics rates regardless of frame rate, and providing time delta information to systems that need it.

**Key considerations:**

- Fixed timestep accumulation for stable physics
- Time scaling for slow-motion effects
- Pause/resume functionality
- Frame rate independent updates

## Constraint Solving

The world enforces game rules that can't be handled by simple collision response, like keeping paddles within bounds or maintaining minimum/maximum ball speeds.

**Key considerations:**

- Paddle position clamping to court boundaries
- Ball speed normalization after collisions
- Paddle rotation limits (if applicable)
- Any other game-specific physics constraints

collision detection: two aabbs overlap if they overlap on ALL three axes

for each axis, boxes overlap if:

- box1's max >= box2's min AND
- box1's min <= box2's max

so just check x, y, z axes separately. if all three pass, collision detected

collision response: conservation of momentum with restitution

1. find collision normal (direction to separate objects)
2. calculate relative velocity along that normal
3. compute impulse magnitude using masses and bounciness
4. apply equal and opposite impulses to both objects

the key insight is you're reversing the velocity component along the collision direction, scaled by how bouncy the materials are

for aabb vs aabb, collision normal is usually the axis with minimum penetration depth (smallest overlap)

impulse formula accounts for:

- how fast objects are approaching each other
- how much they should bounce (restitution)
- mass ratio (heavy objects barely move when hit by light ones)

then you just add/subtract the impulse from each object's velocity based on the normal direction

the math ensures momentum is conserved and energy is reduced by the right amount for realistic bouncing
