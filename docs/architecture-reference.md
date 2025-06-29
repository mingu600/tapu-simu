# Tapu Simu Architecture Reference

*The definitive guide to understanding Tapu Simu's architecture, data flows, and battle execution system*

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Diagrams](#architecture-diagrams)
3. [Data Flow Patterns](#data-flow-patterns)
4. [Battle Execution Flow](#battle-execution-flow)
5. [Module Integration](#module-integration)
6. [State Management](#state-management)
7. [Critical Path Analysis](#critical-path-analysis)

---

## System Overview

Tapu Simu is a sophisticated Pokemon battle simulator architected from the ground up with modern Rust principles. The system follows a **format-first, position-based targeting design** that supports multiple battle formats while maintaining clean separation between data, business logic, and presentation layers.

### Core Principles

- **Format-First Design**: Every component assumes multi-format support from inception
- **Position-Based Targeting**: Explicit positioning system eliminates format assumptions
- **Immutable State**: Battle state is read-only with transformation-based updates
- **Type Safety**: Extensive use of Rust's type system for correctness guarantees
- **Generation-Aware**: Proper handling of Pokemon mechanics across all generations

---

## Architecture Diagrams

### 1. High-Level System Architecture

```mermaid
graph TB
    subgraph "User Interface Layer"
        UI[User Interface]
        CLI[Command Line Interface]
        API[REST API]
    end

    subgraph "Application Layer"
        BattleBuilder[Battle Builder]
        TeamBuilder[Team Builder]
        FormatBuilder[Format Builder]
        BattleEnvironment[Battle Environment]
    end

    subgraph "Domain Layer"
        BattleState[Battle State]
        BattleEngine[Battle Engine]
        Instructions[Instruction System]
        Players[Player System]
    end

    subgraph "Infrastructure Layer"
        DataRepos[Data Repositories]
        TypeSystem[Type System]
        GenerationSystem[Generation System]
        ErrorHandling[Error Handling]
    end

    subgraph "Data Layer"
        JSONData[Pokemon Showdown JSON]
        RepositoryCache[Repository Cache]
        GenerationData[Generation-Specific Data]
    end

    UI --> BattleBuilder
    CLI --> BattleBuilder
    API --> BattleBuilder
    
    BattleBuilder --> TeamBuilder
    BattleBuilder --> FormatBuilder
    BattleBuilder --> BattleEnvironment
    
    BattleEnvironment --> BattleState
    BattleEnvironment --> BattleEngine
    BattleEnvironment --> Players
    
    BattleEngine --> Instructions
    BattleEngine --> DataRepos
    
    BattleState --> TypeSystem
    Instructions --> TypeSystem
    
    DataRepos --> RepositoryCache
    DataRepos --> GenerationSystem
    
    RepositoryCache --> JSONData
    GenerationSystem --> GenerationData
    
    TypeSystem --> ErrorHandling
    DataRepos --> ErrorHandling
```

### 2. Data Repository Architecture

```mermaid
graph TB
    subgraph "External Data Sources"
        PSData[Pokemon Showdown JSON Files]
        GenData[Generation-Specific Data]
        CustomData[Custom Extensions]
    end

    subgraph "Data Loading Layer"
        SerdeLoader[Serde JSON Loader]
        ValidationLayer[Data Validation]
        NormalizationLayer[Name Normalization]
    end

    subgraph "Repository Layer"
        GameDataRepo[GameDataRepository]
        PokemonRepo[Pokemon Repository]
        MoveRepo[Move Repository]
        ItemRepo[Item Repository]
        AbilityRepo[Ability Repository]
        GenRepo[Generation Repository]
    end

    subgraph "Access Layer"
        IndexCache[Indexed Lookup Cache]
        NormalizedNames[Normalized Name Map]
        GenFallback[Generation Fallback Chain]
    end

    subgraph "Consumer Systems"
        BattleSystem[Battle System]
        BuilderSystem[Builder System]
        TestingFramework[Testing Framework]
    end

    PSData --> SerdeLoader
    GenData --> SerdeLoader
    CustomData --> SerdeLoader
    
    SerdeLoader --> ValidationLayer
    ValidationLayer --> NormalizationLayer
    
    NormalizationLayer --> GameDataRepo
    GameDataRepo --> PokemonRepo
    GameDataRepo --> MoveRepo
    GameDataRepo --> ItemRepo
    GameDataRepo --> AbilityRepo
    GameDataRepo --> GenRepo
    
    PokemonRepo --> IndexCache
    MoveRepo --> IndexCache
    ItemRepo --> IndexCache
    AbilityRepo --> IndexCache
    
    IndexCache --> NormalizedNames
    GenRepo --> GenFallback
    
    NormalizedNames --> BattleSystem
    GenFallback --> BattleSystem
    IndexCache --> BuilderSystem
    GameDataRepo --> TestingFramework
```

### 3. Battle Execution Flow

```mermaid
sequenceDiagram
    participant Player1
    participant Player2
    participant BattleEnvironment
    participant BattleEngine
    participant BattleState
    participant InstructionSystem
    participant DataRepositories

    Note over Player1,DataRepositories: Battle Initialization
    BattleEnvironment->>BattleState: new(format, teams)
    BattleState->>DataRepositories: validate_teams()
    DataRepositories-->>BattleState: validation_result

    loop Turn Loop
        Note over Player1,DataRepositories: Move Selection Phase
        BattleEnvironment->>Player1: choose_move(state, options)
        Player1-->>BattleEnvironment: MoveChoice
        BattleEnvironment->>Player2: choose_move(state, options)
        Player2-->>BattleEnvironment: MoveChoice

        Note over Player1,DataRepositories: Turn Execution Phase
        BattleEnvironment->>BattleEngine: execute_turn(choices)
        BattleEngine->>BattleEngine: resolve_targeting()
        BattleEngine->>BattleEngine: determine_move_order()
        
        loop For Each Move
            BattleEngine->>BattleEngine: check_move_prevention()
            BattleEngine->>BattleEngine: calculate_accuracy()
            BattleEngine->>BattleEngine: apply_move_effects()
            BattleEngine->>InstructionSystem: generate_instructions()
            InstructionSystem-->>BattleEngine: BattleInstructions
        end

        BattleEngine->>BattleState: apply_instructions(instructions)
        BattleState->>BattleState: update_state()
        BattleEngine->>BattleEngine: process_end_of_turn()
        
        BattleEngine-->>BattleEnvironment: new_state
        BattleEnvironment->>BattleState: check_battle_over()
        
        alt Battle Continues
            BattleState-->>BattleEnvironment: false
        else Battle Ends
            BattleState-->>BattleEnvironment: true, winner
            break
        end
    end
```

### 4. Instruction Generation Pipeline

```mermaid
graph TB
    subgraph "Move Selection"
        MoveChoice[Move Choice]
        TargetResolution[Auto-Target Resolution]
        ValidatedChoice[Validated Move Choice]
    end

    subgraph "Pre-Move Checks"
        MoveOrder[Move Order Determination]
        Prevention[Move Prevention Checks]
        AccuracyCheck[Accuracy Calculation]
    end

    subgraph "Effect Resolution"
        EffectLookup[Move Effect Lookup]
        ContextCreation[Move Context Creation]
        EffectExecution[Effect Execution]
    end

    subgraph "Instruction Generation"
        DamageCalc[Damage Calculation]
        CriticalBranch[Critical Hit Branching]
        SecondaryEffects[Secondary Effects]
        InstructionAssembly[Instruction Assembly]
    end

    subgraph "State Application"
        InstructionValidation[Instruction Validation]
        StateTransformation[State Transformation]
        PositionTracking[Position Tracking]
    end

    MoveChoice --> TargetResolution
    TargetResolution --> ValidatedChoice
    ValidatedChoice --> MoveOrder
    
    MoveOrder --> Prevention
    Prevention --> AccuracyCheck
    
    AccuracyCheck --> EffectLookup
    EffectLookup --> ContextCreation
    ContextCreation --> EffectExecution
    
    EffectExecution --> DamageCalc
    DamageCalc --> CriticalBranch
    CriticalBranch --> SecondaryEffects
    SecondaryEffects --> InstructionAssembly
    
    InstructionAssembly --> InstructionValidation
    InstructionValidation --> StateTransformation
    StateTransformation --> PositionTracking
```

### 5. Type System Integration

```mermaid
graph TB
    subgraph "Core Types"
        Positions[Position Types]
        Errors[Error Types]
        Entities[Entity Types]
        FromString[String Conversion]
    end

    subgraph "Game Entities"
        Pokemon[Pokemon Types]
        Moves[Move Types]
        Items[Item Types]
        Abilities[Ability Types]
        Status[Status Types]
    end

    subgraph "Battle System"
        BattleState[Battle State]
        Instructions[Instructions]
        Combat[Combat System]
        Targeting[Targeting System]
    end

    subgraph "Data Layer"
        Repositories[Data Repositories]
        Validation[Data Validation]
        Serialization[Serialization]
    end

    subgraph "Builder System"
        Builders[Builder Pattern]
        Configuration[Configuration]
        TeamBuilding[Team Building]
    end

    Positions --> BattleState
    Positions --> Instructions
    Positions --> Targeting
    
    Errors --> Repositories
    Errors --> Builders
    Errors --> Combat
    
    Entities --> Pokemon
    Entities --> Moves
    Entities --> Items
    Entities --> Abilities
    
    FromString --> Pokemon
    FromString --> Moves
    FromString --> Items
    FromString --> Abilities
    
    Pokemon --> BattleState
    Moves --> Combat
    Items --> Combat
    Abilities --> Combat
    Status --> Instructions
    
    BattleState --> Repositories
    Instructions --> Validation
    Combat --> Serialization
    
    Repositories --> Builders
    Validation --> Configuration
    Serialization --> TeamBuilding
```

---

## Data Flow Patterns

### 1. Battle Creation Data Flow

```mermaid
graph LR
    UserInput[User Input] --> FormatBuilder[Format Builder]
    FormatBuilder --> Generation[Generation Selection]
    Generation --> Rules[Format Rules]
    
    Rules --> TeamBuilder[Team Builder]
    TeamBuilder --> DataLookup[Data Repository Lookup]
    DataLookup --> Validation[Team Validation]
    
    Validation --> BattleBuilder[Battle Builder]
    BattleBuilder --> StateCreation[Battle State Creation]
    StateCreation --> BattleState[Final Battle State]
```

### 2. Move Execution Data Flow

```mermaid
graph TD
    PlayerInput[Player Move Input] --> ChoiceValidation[Choice Validation]
    ChoiceValidation --> AutoTargeting[Auto-Targeting]
    AutoTargeting --> OrderResolution[Move Order Resolution]
    
    OrderResolution --> EffectLookup[Effect Lookup]
    EffectLookup --> ContextBuilding[Context Building]
    ContextBuilding --> EffectExecution[Effect Execution]
    
    EffectExecution --> InstructionGen[Instruction Generation]
    InstructionGen --> StateApplication[State Application]
    StateApplication --> NewState[New Battle State]
```

### 3. Data Repository Access Pattern

```mermaid
graph TB
    AccessRequest[Data Access Request] --> Normalization[Name Normalization]
    Normalization --> PrimaryLookup[Primary Generation Lookup]
    
    PrimaryLookup --> Found{Found?}
    Found -->|Yes| ReturnData[Return Data]
    Found -->|No| FallbackLookup[Fallback to Earlier Generation]
    
    FallbackLookup --> FallbackFound{Found?}
    FallbackFound -->|Yes| ReturnData
    FallbackFound -->|No| ErrorHandling[Error: Not Found]
    
    ReturnData --> Cache[Update Cache]
    Cache --> Client[Return to Client]
```

---

## Battle Execution Flow

### 1. Complete Turn Sequence

```mermaid
stateDiagram-v2
    [*] --> TurnStart
    TurnStart --> MoveSelection: Players choose moves
    MoveSelection --> Targeting: Auto-resolve targets
    Targeting --> OrderDetermination: Calculate move order
    OrderDetermination --> MoveExecution: Execute in order
    
    state MoveExecution {
        [*] --> PreventionCheck
        PreventionCheck --> AccuracyCheck: Not prevented
        PreventionCheck --> NextMove: Prevented
        AccuracyCheck --> EffectApplication: Hit
        AccuracyCheck --> NextMove: Miss
        EffectApplication --> InstructionGeneration
        InstructionGeneration --> StateUpdate
        StateUpdate --> NextMove
        NextMove --> [*]
    }
    
    MoveExecution --> EndOfTurn: All moves processed
    EndOfTurn --> VictoryCheck: Apply end-of-turn effects
    VictoryCheck --> TurnStart: Battle continues
    VictoryCheck --> [*]: Battle ends
```

### 2. Move Prevention State Machine

```mermaid
stateDiagram-v2
    [*] --> CheckFlinch
    CheckFlinch --> Prevented: Flinched
    CheckFlinch --> CheckSleep: Not flinched
    
    CheckSleep --> WakeupRoll: Asleep
    CheckSleep --> CheckParalysis: Awake
    WakeupRoll --> Prevented: Stays asleep
    WakeupRoll --> CheckParalysis: Wakes up
    
    CheckParalysis --> PreventionRoll: Paralyzed
    CheckParalysis --> CheckConfusion: Not paralyzed
    PreventionRoll --> Prevented: Fully paralyzed
    PreventionRoll --> CheckConfusion: Can move
    
    CheckConfusion --> ConfusionRoll: Confused
    CheckConfusion --> MoveExecutes: Not confused
    ConfusionRoll --> SelfDamage: Hurts self
    ConfusionRoll --> MoveExecutes: Snaps out
    
    SelfDamage --> Prevented
    MoveExecutes --> [*]
    Prevented --> [*]
```

### 3. Instruction Application Flow

```mermaid
stateDiagram-v2
    [*] --> ValidateInstructions
    ValidateInstructions --> SortByPriority: Valid
    ValidateInstructions --> Error: Invalid
    
    SortByPriority --> ApplyInstruction
    ApplyInstruction --> CheckType
    
    state CheckType {
        [*] --> PokemonInstruction
        [*] --> FieldInstruction
        [*] --> StatusInstruction
        [*] --> StatsInstruction
        
        PokemonInstruction --> ApplyToPokemon
        FieldInstruction --> ApplyToField
        StatusInstruction --> ApplyStatus
        StatsInstruction --> ApplyStats
        
        ApplyToPokemon --> [*]
        ApplyToField --> [*]
        ApplyStatus --> [*]
        ApplyStats --> [*]
    }
    
    CheckType --> UpdatePositions: Applied
    UpdatePositions --> NextInstruction: More instructions
    UpdatePositions --> ValidateState: All applied
    
    NextInstruction --> ApplyInstruction
    ValidateState --> [*]: Success
    ValidateState --> Error: Validation failed
    
    Error --> [*]
```

---

## Module Integration

### 1. Core Module Dependencies

```mermaid
graph TB
    subgraph "Presentation Layer"
        CLI[CLI Interface]
        WebAPI[Web API]
        TestFramework[Test Framework]
    end

    subgraph "Application Layer"
        BattleBuilder[Battle Builder]
        TeamBuilder[Team Builder]
        FormatBuilder[Format Builder]
    end

    subgraph "Domain Layer"
        BattleState[Battle State]
        BattleEngine[Battle Engine]
        Instructions[Instructions]
        Combat[Combat System]
    end

    subgraph "Infrastructure Layer"
        DataRepos[Data Repositories]
        TypeSystem[Type System]
        Errors[Error System]
    end

    CLI --> BattleBuilder
    WebAPI --> BattleBuilder
    TestFramework --> BattleBuilder
    TestFramework --> BattleState
    TestFramework --> Instructions
    
    BattleBuilder --> TeamBuilder
    BattleBuilder --> FormatBuilder
    BattleBuilder --> BattleState
    
    TeamBuilder --> DataRepos
    FormatBuilder --> TypeSystem
    
    BattleState --> TypeSystem
    BattleState --> Errors
    
    BattleEngine --> Combat
    BattleEngine --> Instructions
    BattleEngine --> DataRepos
    
    Combat --> TypeSystem
    Instructions --> TypeSystem
    
    DataRepos --> TypeSystem
    DataRepos --> Errors
```

### 2. Builder Pattern Integration

```mermaid
graph LR
    subgraph "Builder Chain"
        FormatBuilder[Format Builder]
        TeamBuilder[Team Builder]
        BattleBuilder[Battle Builder]
    end

    subgraph "Validation Chain"
        FormatValidation[Format Validation]
        TeamValidation[Team Validation]
        BattleValidation[Battle Validation]
    end

    subgraph "Data Dependencies"
        GenerationData[Generation Data]
        RepositoryAccess[Repository Access]
        TypeValidation[Type Validation]
    end

    FormatBuilder --> FormatValidation
    FormatValidation --> GenerationData
    
    TeamBuilder --> TeamValidation
    TeamValidation --> RepositoryAccess
    
    BattleBuilder --> BattleValidation
    BattleValidation --> TypeValidation
    
    FormatBuilder --> TeamBuilder
    TeamBuilder --> BattleBuilder
    BattleBuilder --> BattleState[Battle State]
```

### 3. Testing Framework Integration

```mermaid
graph TB
    subgraph "Test Layer"
        TestBuilder[Test Builder]
        Assertions[Assertion System]
        MockData[Mock Data]
    end

    subgraph "Production Systems"
        BattleEngine[Battle Engine]
        DataRepos[Data Repositories]
        Instructions[Instruction System]
    end

    subgraph "Validation Layer"
        StateValidation[State Validation]
        InstructionValidation[Instruction Validation]
        ResultValidation[Result Validation]
    end

    TestBuilder --> BattleEngine
    TestBuilder --> DataRepos
    TestBuilder --> MockData
    
    BattleEngine --> Instructions
    Instructions --> StateValidation
    
    Assertions --> InstructionValidation
    Assertions --> ResultValidation
    
    StateValidation --> TestBuilder
    InstructionValidation --> TestBuilder
    ResultValidation --> TestBuilder
```

---

## State Management

### 1. Battle State Lifecycle

```mermaid
graph TB
    subgraph "State Creation"
        InitialState[Initial State Creation]
        TeamSetup[Team Setup]
        FieldSetup[Field Setup]
        ValidatedState[Validated State]
    end

    subgraph "State Transitions"
        CurrentState[Current State]
        InstructionApplication[Instruction Application]
        StateValidation[State Validation]
        NewState[New State]
    end

    subgraph "State Persistence"
        Serialization[State Serialization]
        Storage[State Storage]
        Restoration[State Restoration]
    end

    InitialState --> TeamSetup
    TeamSetup --> FieldSetup
    FieldSetup --> ValidatedState
    
    ValidatedState --> CurrentState
    CurrentState --> InstructionApplication
    InstructionApplication --> StateValidation
    StateValidation --> NewState
    NewState --> CurrentState
    
    CurrentState --> Serialization
    Serialization --> Storage
    Storage --> Restoration
    Restoration --> CurrentState
```

### 2. State Transformation Pipeline

```mermaid
graph LR
    ImmutableState[Immutable State N] --> Clone[Clone State]
    Clone --> Instructions[Apply Instructions]
    Instructions --> Validation[Validate Changes]
    Validation --> ImmutableState2[Immutable State N+1]
    
    subgraph "Instruction Types"
        PokemonInst[Pokemon Instructions]
        FieldInst[Field Instructions]
        StatusInst[Status Instructions]
        StatsInst[Stats Instructions]
    end
    
    Instructions --> PokemonInst
    Instructions --> FieldInst
    Instructions --> StatusInst
    Instructions --> StatsInst
```

### 3. Position-Based State Access

```mermaid
graph TB
    subgraph "Position System"
        BattlePosition[Battle Position]
        SideReference[Side Reference]
        SlotIndex[Slot Index]
    end

    subgraph "State Access"
        StateQuery[State Query]
        PositionResolution[Position Resolution]
        DataRetrieval[Data Retrieval]
    end

    subgraph "Format Awareness"
        FormatValidation[Format Validation]
        PositionLimits[Position Limits]
        TargetingRules[Targeting Rules]
    end

    BattlePosition --> SideReference
    BattlePosition --> SlotIndex
    
    StateQuery --> PositionResolution
    PositionResolution --> BattlePosition
    BattlePosition --> DataRetrieval
    
    DataRetrieval --> FormatValidation
    FormatValidation --> PositionLimits
    PositionLimits --> TargetingRules
```

---

## Critical Path Analysis

### 1. Performance-Critical Paths

**Turn Execution Pipeline**
- Move order determination: O(n log n) where n = active Pokemon
- Instruction generation: O(m) where m = number of effects
- State application: O(k) where k = number of instructions

**Data Access Patterns**
- Repository lookup: O(1) for cached normalized names
- Generation fallback: O(g) where g = generation depth
- Type effectiveness: O(1) with pre-computed matrices

**Memory Usage Patterns**
- State cloning: ~10KB per battle state
- Instruction storage: ~1KB per instruction set
- Repository caching: ~50MB for complete data set

### 2. Error Recovery Paths

```mermaid
graph TB
    ErrorOccurrence[Error Occurrence] --> ErrorClassification[Error Classification]
    
    ErrorClassification --> DataError[Data Error]
    ErrorClassification --> StateError[State Error]
    ErrorClassification --> ValidationError[Validation Error]
    
    DataError --> FallbackData[Fallback to Default]
    StateError --> StateRollback[Rollback State]
    ValidationError --> UserPrompt[Prompt for Correction]
    
    FallbackData --> LogWarning[Log Warning]
    StateRollback --> LogError[Log Error]
    UserPrompt --> RetryOperation[Retry Operation]
    
    LogWarning --> ContinueExecution[Continue Execution]
    LogError --> ContinueExecution
    RetryOperation --> ContinueExecution
```

### 3. Scaling Considerations

**Horizontal Scaling**
- Battle instances are independent and can run in parallel
- Data repositories are read-only and can be shared
- State serialization enables battle pause/resume

**Vertical Scaling**
- Memory usage scales linearly with battle complexity
- CPU usage spikes during instruction generation
- I/O bottlenecks primarily during initial data loading

**Optimization Opportunities**
- Instruction pre-computation for common scenarios
- Battle state diffing for efficient serialization
- Move effect caching for repeated patterns

---

This architecture reference provides the complete foundation for understanding how Tapu Simu's sophisticated battle simulation system integrates across all layers, from low-level type safety to high-level battle orchestration. The modular design enables efficient testing, extensibility, and maintenance while maintaining the complex accuracy required for authentic Pokemon battle simulation.