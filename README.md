# Tapu Simu

![Tapu Simu](assets/tapu-simu-hero.png)

**A next-generation Pokemon battle simulator built from the ground up for multi-format support.**

Tapu Simu is a comprehensive battle engine featuring format-aware mechanics, position-based targeting, and Pokemon Showdown data integration. Designed for accuracy, performance, and extensibility across all Pokemon battle formats.

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

### Spread Move Mechanics

- Doubles/VGC: 0.75x damage multiplier for multi-target moves
- Automatic target resolution based on move data
- Format-aware ally damage calculation

## 🎯 Features

### ✅ **Pokemon Showdown Integration**
- **Battle-tested accuracy** - Direct Pokemon Showdown data integration
- **772+ moves** with complete metadata (flags, effects, secondary effects)
- **Generation-specific data** - Historical move evolution across Gen 1-9
- **244+ items** with comprehensive effect data
- **Synchronous data access** - No async dependencies, blazing fast

### ✅ **Multi-Format Battle Engine**
- **Singles, Doubles, VGC** format support
- **Position-based targeting** with explicit move targeting
- **Format-aware damage calculation** (spread move reduction, etc.)
- **Advanced mechanics** - Redirection, ally interactions, critical hits

### ✅ **Generation Support**
- **Complete Gen 1-9 data** with 252-777 moves per generation
- **Move change tracking** - 319 moves with historical evolution
- **Generation-aware API** - Access moves as they existed in any generation
- **Type evolution tracking** - Bite (Normal→Dark), Gust (Normal→Flying), etc.


## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Original poke-engine**: Foundation and inspiration for battle mechanics