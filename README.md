# Tapu Simu

A format-aware Pokemon battle simulator designed for multi-format support with position-based targeting and comprehensive battle mechanics.

## 🚀 Key Features

- **Multi-Format Support**: Singles, Doubles, VGC, and Triples formats
- **Position-Based Targeting**: All moves use explicit position targeting
- **Format-Aware Architecture**: Battle logic adapts to the active format
- **Advanced Battle Mechanics**: Complete implementation of doubles-specific moves (Follow Me, Helping Hand, etc.)
- **Spread Move Support**: Automatic 0.75x damage reduction for multi-target moves in doubles/VGC
- **Critical Hit Branching**: Percentage-based critical hit calculations with proper damage multipliers
- **Auto-Targeting Engine**: Automatic target resolution for all 16 rustemon/PokeAPI move targets
- **Modern Design**: Built from the ground up with V2 principles
- **No Legacy Compatibility**: Clean, focused implementation
- **Rustemon Integration**: Built-in PokeAPI data fetching via rustemon

## 🏗 Architecture

### Core Modules

- **`battle_format`**: Format definitions and position management
- **`instruction`**: Battle instruction system with position tracking and multi-target support
- **`move_choice`**: Format-aware move choice system with explicit targeting
- **`state`**: Battle state representation with multi-format support
- **`data`**: Pokemon data integration with rustemon/PokeAPI
- **`genx`**: Generation-specific battle mechanics (Phase 4 complete)
  - **`format_targeting`**: Format-aware move target resolution
  - **`format_instruction_generator`**: Spread move and critical hit handling
  - **`doubles_mechanics`**: Complete doubles-specific move implementations
  - **`instruction_generator`**: Main coordinator with auto-targeting

### Design Principles

- **KISS (Keep It Simple, Stupid)**: Straightforward, readable code
- **YAGNI (You Aren't Gonna Need It)**: Only implement immediate requirements
- **Format-First Design**: Everything is designed around multi-format support
- **Position-Based Everything**: All targeting uses explicit positions

## 🎮 Supported Battle Formats

| Format | Active Pokemon | Spread Moves | Ally Damage | Description |
|--------|----------------|--------------|-------------|-------------|
| Singles | 1v1 | ❌ | ❌ | Traditional single Pokemon battles |
| Doubles | 2v2 | ✅ | ✅ | Double battles with ally interactions |
| VGC | 2v2 | ✅ | ✅ | VGC tournament format |
| Triples | 3v3 | ✅ | ✅ | Triple battles (deprecated) |

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

## 🎯 Phase 4: Advanced Battle Mechanics Implementation

Tapu Simu includes a complete implementation of advanced battle mechanics migrated from poke-engine V2:

### Format-Aware Targeting System
- **Complete Move Target Resolution**: Supports all 16 rustemon/PokeAPI move targets
- **Format-Specific Logic**: Singles vs Doubles vs VGC targeting differences
- **Auto-Targeting Engine**: Automatic target resolution for moves without explicit targets

### Multi-Target Instruction System
- **Position-Based Instructions**: All instructions track affected positions
- **Spread Move Support**: Automatic 0.75x damage reduction in doubles/VGC
- **Critical Hit Branching**: Proper percentage-based calculations (1/24 chance, 1.5x multiplier)

### Doubles-Specific Mechanics
- **Follow Me/Rage Powder**: Complete redirection mechanics implementation
- **Helping Hand**: 1.5x damage boost for allies with proper volatile status tracking
- **Wide Guard/Quick Guard**: Protection from spread moves and priority moves
- **Ally Damage**: Proper handling of moves that hit your partner (Earthquake, Surf, etc.)

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

## 🧬 Generation Support

The engine supports multiple Pokemon generations through feature flags:

- **Gen 1-3**: Specific implementations for classic mechanics
- **Gen 4-9**: Modern mechanics (default)
- **Terastallization**: Gen 9 exclusive feature

## 🔄 Data Integration

### Rustemon/PokeAPI Integration

```rust
use tapu_simu::data::RustemonClient;

let mut client = RustemonClient::new();
let pokemon = client.get_pokemon("pikachu").await?;
let move_data = client.get_move("thunderbolt").await?;
```

### Engine Data Conversion

Automatic conversion between PokeAPI data and engine-optimized formats:

```rust
let engine_pokemon = rustemon_pokemon_to_engine(&pokemon);
let engine_move = rustemon_move_to_engine(&move_data);
```

## 🎯 Roadmap

### Phase 1: Core Foundation ✅
- ✅ Multi-format architecture
- ✅ Position-based targeting
- ✅ Basic instruction system
- ✅ State management
- ✅ Rustemon/PokeAPI data integration

### Phase 4: Advanced Battle Mechanics ✅
- ✅ **Format-Aware Targeting System**: Complete move target resolution for all 16 rustemon/PokeAPI targets
- ✅ **Format Instruction Generator**: Spread move damage reduction (0.75x in doubles/VGC)
- ✅ **Doubles-Specific Mechanics**: Follow Me, Helping Hand, Wide Guard, Quick Guard implementation
- ✅ **Multi-Target Instruction System**: Position-aware damage and status instructions
- ✅ **Critical Hit Branching**: Proper percentage-based critical hit calculations
- ✅ **Auto-Targeting Engine**: Automatic target resolution for moves

### Phase 2: Core Battle Mechanics (In Progress)
- 🔄 Enhanced damage calculation with type effectiveness
- 🔄 Comprehensive status condition effects
- 🔄 Weather and terrain effects
- 🔄 Ability system integration
- 🔄 Item effects implementation

### Phase 3: Advanced Features
- ⏳ AI battle simulation
- ⏳ Team validation
- ⏳ Battle replay system
- ⏳ Performance optimization

### Phase 5: Ecosystem
- ⏳ Python bindings
- ⏳ WebAssembly support
- ⏳ REST API interface
- ⏳ Documentation website

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

### Development Guidelines

- Follow the existing code style
- Write comprehensive tests
- Update documentation for new features
- Ensure all tests pass before submitting

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **PokeAPI**: Comprehensive Pokemon data source
- **Rustemon**: Rust wrapper for PokeAPI
- **Original poke-engine**: Foundation and inspiration for battle mechanics

## 📚 Documentation

- [API Documentation](docs/api.md)
- [Battle Format Guide](docs/formats.md)
- [Move Implementation Guide](docs/moves.md)
- [Contributing Guide](docs/contributing.md)