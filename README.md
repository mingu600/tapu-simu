# Tapu Simu (WORK IN PROGRESS)

<div align="center">
  <img src="assets/tapu-simu-logo.png" alt="Tapu Simu Logo" width="200">
</div>

**A next-generation Pokemon battle simulator built from the ground up for multi-format support.**

Tapu Simu is a comprehensive battle engine featuring format-aware mechanics, position-based targeting, and Pokemon Showdown data integration. Designed for accuracy, performance, and extensibility across all Pokemon battle formats.

⚠️ **Note**: This project is under active development and not yet ready for production use.

## 🛠 Installation

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

## 🚀 Quick Start

### Basic Battle Simulation

```rust
use tapu_simu::{State, BattleFormat, MoveChoice, InstructionGenerator};

// Create a new doubles battle
let mut state = State::new(BattleFormat::Doubles);

// Generate instructions for moves
let generator = InstructionGenerator::new(BattleFormat::Doubles);
let instructions = generator.generate_instructions(&mut state, &move1, &move2);
```

### Generation-Specific Data Demo

```bash
# See generation-specific move data in action
cargo run --example generation_data_demo
```

### CLI Usage

```bash
# Run a singles battle
cargo run -- battle --format singles --player-one random --player-two random

# Run multiple doubles battles with verbose output
cargo run -- battle --format doubles --runs 10 --verbose

# Validate a format
cargo run -- validate-format doubles

# Show engine information
cargo run -- info
```

## 🧪 Testing

The engine includes comprehensive tests for all major components:

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test battle_format

# Run tests with output
cargo test -- --nocapture
```

## 🔧 Configuration

### Features

- **`gen1-gen9`**: Enable specific Pokemon generation support
- **`terastallization`**: Enable Terastallization mechanics (requires gen9)
- **`remove_low_chance_instructions`**: Optimization for performance

### Default Configuration

```toml
[features]
default = ["gen9", "terastallization"]
```

## 📊 Battle Mechanics

### Position-Based Targeting

All moves in the V2 engine use explicit position targeting:

```rust
// Target opponent's slot 0 in doubles
let target = BattlePosition::new(SideReference::SideTwo, 0);
let move_choice = MoveChoice::new_move(MoveIndex::M0, vec![target]);
```

### Format-Aware Instructions

Instructions automatically track affected positions:

```rust
let instructions = StateInstructions::new(100.0, vec![
    Instruction::PositionDamage(PositionDamageInstruction {
        target_position: BattlePosition::new(SideReference::SideTwo, 0),
        damage_amount: 50,
    })
]);
// affected_positions automatically populated
```

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Original poke-engine**: Foundation and inspiration for battle mechanics