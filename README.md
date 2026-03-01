# No Free Gravity 👩‍🚀🪐

A 2D procedurally generated, Newtonian physics-based roguelike built entirely in Rust using the [Bevy Engine](https://bevyengine.org/) and `bevy_rapier2d`. 

**Note: This project serves as a demonstration of AI-Agentic Development.** The entire codebase was written autonomously by **Google Antigravity** acting as an agentic AI coder, directed and guided by a human developer.

## The Concept

*No Free Gravity* puts player movement and evasion at the forefront. As an astronaut lost in a deep-space asteroid field, you must:
1. Coast seamlessly through a procedurally generated, infinite universe of rock and ice.
2. Rely entirely on strict Newtonian physics concepts: manage your rotational momentum and limited thrust power to navigate. 
3. Become caught in the gravity wells of passing planets to anchor yourself, mining their surfaces for resources.
4. Use those resources at rare outliner Shops to upgrade your suit.
5. Evade chasing threats (Planned for Milestone 5) using movement tech rather than bullet-hell mechanics.

## Architecture Highlights
- **Engine**: Bevy (Data-driven ECS framework)
- **Physics**: Rapier2D (Strict custom parameters to enforce a realistic zero-g environment mixed with high-mass sticky planets)
- **Generation**: Noise-driven infinite chunking system for seamless travel without loading screens or performance degradation.
- **Modularity**: The codebase is split tightly into isolated Bevy Plugins (`PlayerPlugin`, `PhysicsPlugin`, `WorldGenPlugin`, `EnvironmentPlugin`) tracking distinct domains, joined by a central `GameConfig` resource for easy tuning.

## Getting Started

Make sure you have [Rust and Cargo](https://rustup.rs/) installed.

```bash
# Clone the repo
git clone https://github.com/tyler-larsen/no_free_gravity.git

# Enter the directory
cd no_free_gravity

# Run the game
cargo run
```

### Controls
- **W / Up Arrow**: Engage Primary Thrusters (Hold to accelerate)
- **A / Left Arrow**: Rotate Suit Left (Counterclockwise)
- **D / Right Arrow**: Rotate Suit Right (Clockwise)
- **Q**: Deploy Scanner (Reveals hidden resource nodes within radius)
- **E**: Deploy Drill (Mines revealed nodes when landed directly on top of them)
- **F**: Open Shop Menu (When landed near a vendor)
- **Space** (While Landed): Hold to charge thrusters, release to break free of the planet's gravity.

## AI Development Process (Antigravity)
This project is an exercise in human-agent pairing. The user sets high-level objectives (e.g., "add a scanning mechanic", "fix the spin-only bug between intersecting planets"), and **Google Antigravity** explores the codebase, architects the changes (planning `Components` and `Systems`), edits the `.rs` files directly, runs the Bevy compiler to fix errors, and validates the mechanics. 

For more details on how the AI agent interacted with this repo, see `AGENTS.md`.
