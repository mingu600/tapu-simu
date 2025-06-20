# Tapu Simu

<div align="center">
  <img src="assets/tapu-simu-logo.png" alt="Tapu Simu Logo" width="200">
</div>

**A next-generation Pokemon battle simulator built from the ground up for multi-format support.**

Tapu Simu is a comprehensive battle engine featuring format-aware mechanics, position-based targeting, and Pokemon Showdown data integration. Designed for accuracy, performance, and extensibility across all Pokemon battle formats.

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

⚠️ **Note**: This project is under active development and not yet ready for production use.


## 🏗️ Architecture

Tapu Simu follows a clean, modular architecture designed for maintainability and extensibility:

```
src/
├── core/           # Core battle abstractions
│   ├── battle_format.rs    # Format definitions and mechanics
│   ├── instruction.rs      # Battle instruction system
│   ├── move_choice.rs      # Move selection and targeting
│   └── state.rs           # Battle state management
├── engine/         # Battle mechanics implementation
│   ├── combat/            # Damage calculation and type effectiveness
│   ├── mechanics/         # Abilities, items, and switch effects
│   ├── targeting/         # Auto-targeting and format-specific targeting
│   └── turn/              # Turn processing and instruction generation
├── data/           # Pokemon data integration
│   ├── services/          # Type charts and move services
│   ├── loader.rs          # Data loading from PS JSON files
│   └── ps_types.rs        # Pokemon Showdown type definitions
├── testing/        # Testing framework and utilities
├── ui/             # Web interface bridge
└── bin/            # Binary executables
```

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd tapu-simu

# Build the project
cargo build --release

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### Basic Usage

#### Battle CLI

Tapu Simu includes a comprehensive command-line interface for running Pokemon battles with AI players:

```bash
# Run a single battle between two random players
cargo run --bin tapu-simu -- battle

# Run multiple battles with different player types
cargo run --bin tapu-simu -- battle \
  --format doubles \
  --player-one random \
  --player-two damage \
  --runs 100

# Run battles with verbose logging
cargo run --bin tapu-simu -- battle \
  --player-one first \
  --player-two random \
  --max-turns 1 \
  --verbose \
  --log-file battle.log \
  --team-index 3

# Test different battle formats
cargo run --bin tapu-simu -- battle --format vgc --runs 10
cargo run --bin tapu-simu -- battle --format gen4ou --runs 5
```

**Available Player Types:**
- `random` - Selects moves randomly from available options
- `first` - Always chooses the first available move
- `damage` - Estimates damage and picks highest-damage moves

**Supported Formats:**
- `singles` - Standard 1v1 singles battles
- `doubles` - 2v2 doubles battles  
- `vgc` - VGC 2024 format with official rules
- `triples` - 3v3 triples battles
- `gen9ou` - Generation 9 OU singles
- `gen4ou` - Generation 4 OU singles

**Battle Statistics:**
When running multiple battles, the CLI provides detailed statistics:

```bash
$ cargo run --bin tapu-simu -- battle --runs 50 --player-one random --player-two damage

Running 50 battle(s) in Singles format

=== Battle Summary ===
Total battles: 50
Player 1 (random) wins: 18 (36.0%)
Player 2 (damage) wins: 32 (64.0%)
Draws: 0 (0.0%)
```

#### Utility Commands

```bash
# Validate battle format configuration
cargo run --bin tapu-simu -- validate-format doubles

# Show engine information and supported features
cargo run --bin tapu-simu -- info
```

#### Web Interface

```bash
# Start the UI server
cargo run --bin ui-server

# Open browser to http://localhost:3000
# Build and serve the frontend (requires Node.js)
cd ui && npm install && npm run dev
```

#### Library Usage

```rust
use tapu_simu::{BattleFormat, State, MoveChoice};
use tapu_simu::core::move_choice::{MoveIndex, PokemonIndex};
use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::engine::turn::instruction_generator::InstructionGenerator;

// Create a new doubles battle
let mut state = State::new(BattleFormat::gen9_vgc());

// Create move choices with explicit targeting
let move1 = MoveChoice::new_move(
    MoveIndex::M0, 
    vec![BattlePosition::new(SideReference::SideTwo, 0)]
);
let move2 = MoveChoice::new_move(
    MoveIndex::M0, 
    vec![BattlePosition::new(SideReference::SideOne, 0)]
);

// Generate and apply instructions
let generator = InstructionGenerator::new(BattleFormat::gen9_vgc());
let instructions = generator.generate_instructions(&mut state, &move1, &move2);

for instruction_set in instructions {
    state.apply_instructions(&instruction_set.instruction);
    // Process battle state...
    state.reverse_instructions(&instruction_set.instruction); // Undo if needed
}
```

## 🧪 Testing

The engine includes comprehensive testing with a dedicated framework:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test battle_format

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test test_core_battle_mechanics
cargo test --test test_instruction_generation_integration
```

## ⚙️ Configuration



## 📊 Battle Mechanics

### Position-Based Targeting System

All moves use explicit position targeting instead of implicit "opponent" targeting:

```rust
// Singles targeting
let target = BattlePosition::new(SideReference::SideTwo, 0);

// Doubles targeting - hit both opponents
let targets = vec![
    BattlePosition::new(SideReference::SideTwo, 0),
    BattlePosition::new(SideReference::SideTwo, 1),
];

let move_choice = MoveChoice::new_move(MoveIndex::M0, targets);
```

### Instruction System

Battle mechanics are implemented through atomic, reversible instructions:

```rust
// Damage instruction with undo support
let instruction = Instruction::PositionDamage(PositionDamageInstruction {
    target_position: BattlePosition::new(SideReference::SideTwo, 0),
    damage_amount: 50,
    previous_hp: Some(100), // For undo support
});

// Probabilistic instruction sets
let instructions = StateInstructions::new(95.0, vec![normal_damage]);
let crit_instructions = StateInstructions::new(5.0, vec![critical_damage]);
```

### Format-Aware Mechanics

The engine automatically handles format-specific behavior:

- **Singles**: Direct targeting, no redirection
- **Doubles**: Spread move damage reduction, redirection abilities, position-aware targeting
- **VGC**: Team preview, restricted legendaries, format-specific clauses

### State Serialization Format

The engine supports complete battle state serialization for debugging, logging, and replay functionality. States serialized to compact string format with hierarchical structure:

```
format/side_one/side_two/weather/terrain/turn/trick_room
```

**Format Structure:**
- **Top Level**: `/` separates major components (format, sides, conditions)
- **Components**: `|` separates fields within components  
- **Pokemon Data**: `,` separates individual pokemon attributes
- **Arrays**: `~` separates array elements (stats, types, moves)
- **Move Data**: `!` separates move attributes, `#` separates count from data

**Example Serialized State:**
```
Gen 9 Random Battle|9|0|6|1|2|###/6|Greninja-Bond,259,259,174~156~191~174~174,,0,x,,Battle Bond,Life Orb,Normal,80,1,0#Ice Beam!80!100!Normal!20!20!0!0!0~3#Dark Pulse!80!100!Normal!20!20!0!0!0|...|0,x,x||/6|Sawsbuck,284,284,191~210~191~171~191,,0,x,,Sap Sipper,Life Orb,Normal,88,1,2#Horn Leech!80!100!Normal!20!20!0!0!0~3#Double-Edge!80!100!Normal!20!20!0!0!0|...|3,x,x||/0/0/1/0
```

**Key Features:**
- Complete battle state reconstruction from string
- Compact format suitable for logging and transmission
- Used in battle logs (`battle.log`) and testing framework
- Supports undo system and state comparison

```rust
// Serialize current battle state
let serialized = state.serialize();

// Restore state from string
let restored_state = State::deserialize(&serialized)?;
```

## 🗂️ Data Integration

### Pokemon Showdown Data

Tapu Simu integrates directly with Pokemon Showdown's data format:

```
data/ps-extracted/
├── moves.json              # Complete move database
├── moves-by-generation.json # Generation-specific move data
├── items.json              # Item database
├── pokemon.json            # Pokemon species data
└── move-changes.json       # Generation-specific changes
```

### Data Extraction

```bash
# Extract latest PS data (requires Node.js)
cd tools/ps-data-extractor
npm install
npm run extract
```

See `CLAUDE.md` for detailed development guidelines.

## 📁 Project Structure

```
tapu-simu/
├── src/                    # Source code
├── tests/                  # Integration tests
├── data/                   # Pokemon data files
├── docs/                   # Documentation
├── ui/                     # Web interface (React/TypeScript)
├── tools/                  # Development tools
└── examples/               # Usage examples
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Note**: Tapu Simu is an independent project and is not affiliated with The Pokemon Company, Game Freak, or Pokemon Showdown.