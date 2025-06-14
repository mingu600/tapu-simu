# Pokemon Showdown Target System Integration

## Current State Analysis

### Rustemon/PokeAPI Targets (current)
Our current system uses rustemon's 16 target types from PokeAPI:
1. `specific-move` - Counter, Mirror Coat
2. `selected-pokemon-me-first` - Me First  
3. `ally` - Single ally
4. `users-field` - User's side field effects
5. `user-or-ally` - User or ally
6. `opponents-field` - Opponent's side field effects
7. `user` - The user
8. `random-opponent` - Random opponent
9. `all-other-pokemon` - All except user
10. `selected-pokemon` - Standard single target
11. `all-opponents` - All opponents
12. `entire-field` - Whole field
13. `user-and-allies` - User + allies
14. `all-pokemon` - Everyone
15. `all-allies` - All allies
16. `fainting-pokemon` - Fainting Pokemon

### Pokemon Showdown Targets
PS uses 15 target types with cleaner naming:
1. `normal` - Standard single target move
2. `self` - Targets the user
3. `adjacentAlly` - Adjacent ally only
4. `adjacentAllyOrSelf` - User or adjacent ally
5. `adjacentFoe` - Single adjacent opponent
6. `allAdjacentFoes` - All adjacent opponents (spread)
7. `allAdjacent` - All adjacent Pokemon
8. `all` - Entire field
9. `allyTeam` - User's entire team
10. `allySide` - User's side field
11. `foeSide` - Opponent's side field  
12. `any` - Any single target (Long-range)
13. `randomNormal` - Random opponent
14. `scripted` - Counter, Mirror Coat, etc.
15. `allies` - All active allies

## Mapping Analysis

### Direct Mappings
| Rustemon | Pokemon Showdown | Notes |
|----------|------------------|-------|
| `selected-pokemon` | `normal` | Standard targeting |
| `user` | `self` | Self-targeting |
| `ally` | `adjacentAlly` | Single ally |
| `user-or-ally` | `adjacentAllyOrSelf` | User or ally |
| `all-opponents` | `allAdjacentFoes` | Spread moves |
| `entire-field` | `all` | Field effects |
| `users-field` | `allySide` | Side conditions |
| `opponents-field` | `foeSide` | Opposite side |
| `random-opponent` | `randomNormal` | Random targeting |
| `specific-move` | `scripted` | Counter/Mirror Coat |

### Complex Mappings
| Rustemon | Pokemon Showdown | Notes |
|----------|------------------|-------|
| `all-other-pokemon` | `allAdjacent` | Adjacent only in PS |
| `user-and-allies` | No exact match | PS separates these |
| `all-pokemon` | No exact match | PS uses `all` for field |
| `selected-pokemon-me-first` | Part of `scripted` | Me First handling |
| No equivalent | `any` | Long-range singles |
| No equivalent | `allyTeam` | Full team effects |
| No equivalent | `allies` | All active allies |

## Refactoring Strategy

### Phase 1: Add PS Target Support
1. Create new enum with PS targets
2. Add conversion from rustemon → PS
3. Update format targeting to handle PS targets

### Phase 2: Migrate Internal Usage  
1. Replace MoveTarget usage throughout
2. Update tests
3. Remove old enum

### Phase 3: Direct PS Data Usage
1. Load moves from PS extractor
2. Use PS targets natively
3. Remove rustemon dependency for moves

## Implementation Benefits

1. **Cleaner semantics** - "normal" vs "selected-pokemon"
2. **Better doubles support** - Adjacent targeting explicit
3. **Simpler code** - Direct PS data usage
4. **Community alignment** - Same terms as PS