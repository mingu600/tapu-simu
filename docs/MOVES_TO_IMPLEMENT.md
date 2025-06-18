# Comprehensive List of Moves to Implement in Tapu Simu

This document contains all move effects that have been implemented in poke-engine and need to be ported to tapu-simu for 100% parity. Each move listed has actual logic implementation (not just placeholders).

## Implementation Status Legend
- ✅ **COMPLETE**: Function implemented + dedicated tests + gen-aware mechanics
- 🟡 **PARTIAL**: Function implemented + basic testing, but missing dedicated tests
- ❌ **MISSING**: No implementation found

## Implementation Summary
- **Complete**: 3 moves (2%)
- **Partial**: ~61 moves (38%)
- **Missing**: ~97+ moves (60%)

## Status and Utility Moves

- 🟡 ABSORB - HP draining move
- ❌ ACID - Defense lowering move with secondary effect
- 🟡 AGILITY - Speed boosting move
- ❌ AQUARING - Gradual HP recovery
- ❌ AROMATHERAPY - Clears status conditions for entire team
- ❌ ATTRACT - Causes infatuation
- ❌ AURORAVEIL - Sets up combined Light Screen + Reflect
- ❌ BATONPASS - Passes stat changes to incoming Pokemon
- ❌ BELLYDRUM - Maximizes Attack at cost of 50% HP
- ❌ CHARM - Lowers opponent's Attack by 2 stages
- ❌ CHILLYRECEPTION - Sets snow weather + switches out
- ❌ CLEARSMOG - Removes all stat changes from target
- ❌ CONFUSERAY - Causes confusion
- ❌ COURTCHANGE - Swaps hazards between sides
- ❌ CURSE - Different effects for Ghost vs non-Ghost types
- ❌ DEFOG - Removes hazards and terrain
- ❌ DESTINYBOND - KO's opponent if user faints
- ❌ ENCORE - Forces opponent to repeat last move
- 🟡 ENDURE - Survives with 1 HP
- ❌ GLARE - Inflicts paralysis
- ❌ GRASSYGLIDE - Priority in Grassy Terrain
- ❌ GROWTH - Stat boost (enhanced in sun)
- ❌ HAZE - Resets all stat changes
- ❌ HEALBELL - Cures status conditions for team
- ❌ HEALINGWISH - User faints, fully heals replacement
- ❌ KINESIS - Lowers accuracy
- ❌ LEECHSEED - Drains HP every turn
- ❌ LIFEDEW - Heals user and ally
- ❌ LIGHTSCREEN - Reduces Special damage
- 🟡 MORNINGSUN - Variable healing based on weather
- 🟡 MOONLIGHT - Variable healing based on weather
- 🟡 SYNTHESIS - Variable healing based on weather
- 🟡 NASTYPLOT - Boosts Special Attack by 2 stages
- ❌ NORETREAT - Boosts all stats but prevents switching
- ❌ PAINSPLIT - Averages HP between user and target
- ❌ PARTINGSHOT - Lowers opponent's stats then switches
- ❌ PERISHSONG - Both Pokemon faint in 3 turns
- 🟡 PROTECT - Blocks most moves
- ❌ QUICKATTACK - Priority physical move
- ❌ RAPIDSPIN - Removes hazards from user's side
- 🟡 RECOVER - Restores 50% HP
- ❌ REFLECT - Reduces Physical damage
- ❌ REFRESH - Cures user's status condition
- ❌ REST - Full heal + sleep for 2-3 turns
- 🟡 ROOST - Restores HP, temporarily loses Flying type
- ❌ SHOREUP - Variable healing (enhanced in sand)
- 🟡 SLACKOFF - Restores 50% HP
- ❌ SLEEPTALK - Uses random move while asleep
- ❌ SPIKES - Entry hazard
- ❌ SPLASH - Does nothing
- ❌ SPORE - 100% sleep move
- ❌ STEALTHROCK - Entry hazard based on type effectiveness
- 🟡 SUBSTITUTE - Creates HP-costing decoy
- ❌ SUNNYDAY - Sets sun weather
- ❌ RAINDANCE - Sets rain weather
- ❌ SANDSTORM - Sets sand weather
- ❌ HAIL - Sets hail weather
- ❌ SNOWSCAPE - Sets snow weather
- ✅ SWORDSDANCE - Boosts Attack by 2 stages
- ❌ TAILWIND - Doubles Speed for 4 turns
- ❌ TAUNT - Prevents status moves
- ✅ THUNDERWAVE - Inflicts paralysis
- ❌ TIDYUP - Removes hazards and substitutes, boosts stats
- ✅ TOXIC - Badly poisons (100% accuracy for Poison types)
- ❌ TRICK - Swaps items
- ❌ SWITCHEROO - Swaps items
- ❌ TRICKROOM - Reverses speed priority
- ❌ WHIRLWIND - Forces opponent to switch
- 🟡 WILLOWISP - Inflicts burn
- ❌ WISH - Delayed healing
- ❌ YAWN - Causes sleep next turn

## Variable Power Moves

- ❌ ACROBATICS - Doubles power without item
- ❌ AVALANCHE - Doubles power if hit first
- ❌ BARBBARRAGE - Doubles power against poisoned targets
- ❌ BOLTBEAK - Doubles power if moving first
- ❌ FISHIOUSREND - Doubles power if moving first
- ❌ COLLISIONCOURSE - 1.3x power against super effective
- ❌ ELECTRODRIFT - 1.3x power against super effective
- ❌ ELECTROBALL - Power based on Speed ratio
- ❌ ERUPTION - Power based on current HP
- ❌ WATERSPOUT - Power based on current HP
- ❌ DRAGONENERGY - Power based on current HP
- ❌ FACADE - Doubles power with status condition
- ❌ FREEZEDRY - Super effective against Water types
- ❌ GRASSKNOT - Power based on target's weight
- ❌ LOWKICK - Power based on target's weight
- ❌ GYROBALL - Higher power with lower Speed
- ❌ HARDPRESS - Power based on target's remaining HP
- ❌ HEATCRASH - Power based on weight ratio
- ❌ HEAVYSLAM - Power based on weight ratio
- ❌ HEX - Doubles power against statused targets
- ❌ HYDROSTEAM - Boosted power in sun weather
- ❌ IVYCUDGEL - Type changes with mask items
- ❌ LASTRESPECTS - Power increases with fainted team members
- ❌ POLTERGEIST - Fails if target has no item
- ❌ PURSUIT - Doubles power against switching targets
- ❌ REVERSAL - Higher power at lower HP
- ❌ STOREDPOWER - Power increases with stat boosts
- ❌ POWERTRIP - Power increases with stat boosts
- ❌ STRENGTHSAP - Heals based on target's Attack stat
- ❌ SUCKERPUNCH - Priority move that fails against status moves
- ❌ THUNDERCLAP - Priority move that fails against status moves
- ❌ TERABLAST - Type and category change when Terastallized
- ❌ TERRAINPULSE - Type and power change based on terrain
- ❌ UPPERHAND - Priority counter to priority moves
- ❌ WEATHERBALL - Type and power change based on weather

## Multi-Hit and Charge Moves

- 🟡 DOUBLESLAP - Multi-hit move (functions implemented)
- 🟡 COMETPUNCH - Multi-hit move (functions implemented)
- 🟡 FURYATTACK - Multi-hit move (functions implemented)
- 🟡 PINMISSILE - Multi-hit move (functions implemented)
- 🟡 BARRAGE - Multi-hit move (functions implemented)
- 🟡 SPIKECANNON - Multi-hit move (functions implemented)
- 🟡 BONEMERANG - Multi-hit move (functions implemented)
- 🟡 BULLETSEED - Multi-hit move (functions implemented)
- 🟡 ICICLESHARD - Multi-hit move (functions implemented)
- 🟡 ROCKBLAST - Multi-hit move (functions implemented)
- 🟡 TAILSLAP - Multi-hit move (functions implemented)
- 🟡 BEATUP - Multi-hit move (functions implemented)
- 🟡 ARMTHRUST - Multi-hit move (functions implemented)
- ❌ BOUNCE - Two-turn move (semi-invulnerable)
- ❌ DIG - Two-turn move (underground)
- ❌ DIVE - Two-turn move (underwater)
- ❌ FLY - Two-turn move (airborne)
- ❌ FUTURESIGHT - Delayed damage after 3 turns
- ❌ METEORBEAM - Boosts Special Attack on charge turn
- ❌ ELECTROSHOT - Boosts Special Attack on charge turn
- ❌ PHANTOMFORCE - Two-turn Ghost move
- ❌ SHADOWFORCE - Two-turn Ghost move
- ❌ RAZORWIND - Two-turn Normal move
- ❌ SKULLBASH - Boosts Defense on charge turn
- ❌ SKYATTACK - Two-turn Flying move
- ❌ SKYDROP - Two-turn move that lifts target
- ❌ SOLARBEAM - No charge in sun, reduced power in other weather
- ❌ SOLARBLADE - No charge in sun, reduced power in other weather

## Fixed Damage Moves

- ❌ ENDEAVOR - Reduces target HP to user's HP
- ❌ FINALGAMBIT - Damage equals user's HP, user faints
- ❌ NATURESMADNESS - Halves target's HP
- ❌ RUINATION - Halves target's HP
- ❌ NIGHTSHADE - Damage equals user's level
- ❌ SEISMICTOSS - Damage equals user's level
- ❌ SUPERFANG - Halves target's HP

## Recoil and Self-Damage Moves

- 🟡 DOUBLEEDGE - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 TAKEDOWN - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 SUBMISSION - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 VOLTTACKLE - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 FLAREBLITZ - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 BRAVEBIRD - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 WILDCHARGE - Recoil move (functions implemented, recoil handled via PS data)
- 🟡 HEADSMASH - Recoil move (functions implemented, recoil handled via PS data)
- ❌ DOUBLESHOCK - Removes user's Electric typing
- ❌ BURNUP - Removes user's Fire typing
- ❌ CLANGOROUSSOUL - Boosts all stats, costs 1/3 HP
- ❌ EXPLOSION - User faints (doubled power in Gen 3/4)
- ❌ SELFDESTRUCT - User faints (doubled power in Gen 3/4)
- ❌ FILLETAWAY - Boosts offensive stats, costs 1/2 HP
- ❌ MINDBLOWN - Damages user for 1/2 max HP

## Drain Moves

- 🟡 GIGADRAIN - Drain move (functions implemented, drain handled via PS data)
- 🟡 MEGADRAIN - Drain move (functions implemented, drain handled via PS data)
- 🟡 DRAINPUNCH - Drain move (functions implemented, drain handled via PS data)
- 🟡 LEECHLIFE - Drain move (functions implemented, drain handled via PS data)
- 🟡 DREAMEATER - Drain move (functions implemented, drain handled via PS data)

## Item Interaction Moves

- ❌ FLING - Power and effect based on held item
- ❌ KNOCKOFF - Removes target's item (bonus damage in Gen 6+)
- ❌ THIEF - Steals target's item if user has none

## Counter Moves

- ❌ COMEUPPANCE - Returns 1.5x damage taken
- ❌ COUNTER - Returns 2x physical damage
- ❌ METALBURST - Returns 1.5x damage taken
- ❌ MIRRORCOAT - Returns 2x special damage

## Type-Changing Moves

- ❌ AURAWHEEL - Type changes with Morpeko form
- ❌ JUDGMENT - Type matches user's primary type
- ❌ MULTIATTACK - Type matches user's primary type
- ❌ RAGINGBULL - Type and effects change with Tauros form
- ❌ REVELATIONDANCE - Type matches user's primary type (or Tera type)

## Priority Moves

- ❌ ACCELEROCK - Rock-type priority
- ❌ AQUAJET - Water-type priority
- ❌ BULLETPUNCH - Steel-type priority
- ❌ EXTREMESPEED - +2 priority Normal move
- ❌ FAKEOUT - Flinches, only works on first turn
- ❌ FEINT - Breaks through protection
- ❌ FIRSTIMPRESSION - Bug-type priority, only works on first turn
- ❌ MACHPUNCH - Fighting-type priority

## Weather-Dependent Moves

- ❌ BLIZZARD - 100% accuracy in hail
- ❌ HURRICANE - 100% accuracy in rain, 50% in sun
- ❌ THUNDER - 100% accuracy in rain, 50% in sun
- ❌ EXPANDINGFORCE - Boosted power in Psychic Terrain
- ❌ MISTYEXPLOSION - Boosted power in Misty Terrain
- ❌ PSYBLADE - Boosted power in Electric Terrain
- ❌ RISINGVOLTAGE - Boosted power in Electric Terrain
- ❌ STEELROLLER - Fails without terrain

## Entry Hazard Clearing

- ❌ COURTCHANGE - Swaps all hazards between sides
- ❌ DEFOG - Removes all hazards and terrain
- ❌ MORTALSPIN - Rapid Spin + poison damage
- ❌ RAPIDSPIN - Removes hazards from user's side
- ❌ TIDYUP - Removes hazards and substitutes

## Special Mechanics

- ❌ FOCUSPUNCH - Fails if user takes direct damage
- ❌ ICESPINNER - Removes terrain after hitting
- ❌ RAGINGBULL - Breaks screens and barriers
- ❌ PHOTONGEYSER - Physical if Attack > Special Attack

---

**Total Count: 149+ moves with implemented effects**

This represents all moves with actual implemented logic in poke-engine that need to be ported to tapu-simu for complete parity. Each move has specific mechanics beyond basic damage calculation that must be faithfully reproduced in the new architecture.