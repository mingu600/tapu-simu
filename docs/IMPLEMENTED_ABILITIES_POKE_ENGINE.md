# Implemented Abilities in Poke-Engine

This document lists all abilities that have actual code implementation in poke-engine, organized by trigger/effect type.

## Switch-In Abilities (ability_on_switch_in)
- **DROUGHT** - Sets Sun weather
- **SANDSTREAM** - Sets Sand weather  
- **INTIMIDATE** - Lowers opponent's Attack by 1
- **DRIZZLE** - Sets Rain weather
- **TRACE** - Copies opponent's ability

## End-of-Turn Abilities (ability_end_of_turn)
- **SPEEDBOOST** - Boosts Speed by 1 each turn
- **RAINDISH** - Heals 1/16 HP in Rain
- **DRYSKIN** - Heals 1/8 HP in Rain
- **SHEDSKIN** - Removes status conditions
- **HUNGERSWITCH** - Forme change for Morpeko
- **ICEFACE** - Forme restoration in Hail/Snow

## After Damage Hit Abilities (ability_after_damage_hit)
- **BATTLEBOND** - Boosts Attack/SpA/Speed when KOing opponent
- **MAGICIAN** - Steals opponent's item when dealing damage
- **PICKPOCKET** - Steals opponent's item when dealing damage
- **MOXIE** - Boosts Attack by 1 when KOing opponent
- **CHILLINGNEIGH** - Boosts Attack by 1 when KOing opponent
- **ASONEGLASTRIER** - Boosts Attack by 1 when KOing opponent
- **GRIMNEIGH** - Boosts Special Attack by 1 when KOing opponent  
- **ASONESPECTRIER** - Boosts Special Attack by 1 when KOing opponent
- **BEASTBOOST** - Boosts highest stat by 1 when KOing opponent
- **MUMMY** - Changes attacker's ability to Mummy on contact
- **LINGERINGAROMA** - Changes attacker's ability to Mummy on contact
- **WANDERINGSPIRIT** - Changes attacker's ability to Mummy on contact
- **GULPMISSILE** - Forme change and damage/effects on hit
- **COLORCHANGE** - Changes type to match move that hit
- **STAMINA** - Boosts Defense by 1 when hit
- **COTTONDOWN** - Lowers attacker's Speed by 1 when hit
- **SANDSPIT** - Sets Sand weather when hit
- **SEEDSOWER** - Sets Grassy Terrain when hit
- **TOXICDEBRIS** - Sets Toxic Spikes when hit by physical moves
- **BERSERK** - Boosts Special Attack when HP drops below 50%
- **ROUGHSKIN** - Damages attacker 1/8 HP on contact
- **IRONBARBS** - Damages attacker 1/8 HP on contact
- **AFTERMATH** - Damages attacker 1/4 HP when KO'd by contact
- **INNARDSOUT** - Damages attacker equal to damage taken when KO'd
- **PERISHBODY** - Applies Perish Song to both Pokemon on contact

## Switch-Out Abilities (ability_on_switch_out)
- **GULPMISSILE** - Forme change back to base form
- **ZEROTOHERO** - Forme change for Palafin
- **HUNGERSWITCH** - Forme change for Morpeko
- **NATURALCURE** - Removes status conditions
- **REGENERATOR** - Heals 1/3 HP

## Attack Modification Abilities (ability_modify_attack_being_used)
- **PROTEAN** - Changes type to match move being used (Gen 6-8 vs Gen 9 variants)
- **LIBERO** - Changes type to match move being used (Gen 6-8 vs Gen 9 variants)
- **GORILLATACTICS** - Disables other moves after using one
- **PRANKSTER** - Gives priority to status moves (with Dark-type immunity check)

## Defensive Abilities (ability_modify_attack_against)
- **TABLETSOFRUIN** - Reduces physical move power by 25%
- **SWORDOFRUIN** - Reduces physical move power by 25%
- **VESSELOFRUIN** - Reduces special move power by 25%
- **BEADSOFRUIN** - Reduces special move power by 25%

- **PUREPOWER** - Doubles physical move power
- **TORRENT** - Boosts Water moves by 1.5x when at low HP
- **SERENEGRACE** - Doubles secondary effect chances
- **HUGEPOWER** - Doubles physical move power
- **COMPOUNDEYES** - Increases accuracy by 1.3x
- **STENCH** - Adds 10% flinch chance to moves
- **SWARM** - Boosts Bug moves by 1.5x when at low HP
- **BLAZE** - Boosts Fire moves by 1.5x when at low HP
- **OVERGROW** - Boosts Grass moves by 1.5x when at low HP
- **HUSTLE** - Boosts physical move power by 1.5x, reduces accuracy to 80%
- **GUTS** - Boosts move power by 1.5x when statused (with Burn interaction)
- **SOUNDPROOF** - Blocks sound-based moves
- **POISONPOINT** - 33% chance to poison on contact
- **LIGHTNINGROD** - Redirects Electric moves and boosts Special Attack
- **MARVELSCALE** - Reduces physical damage by 1.5x when statused
- **EFFECTSPORE** - Chance to inflict Poison/Paralysis/Sleep on contact
- **FLAMEBODY** - 30% chance to burn on contact
- **SUCTIONCUPS** - Prevents forced switching
- **WONDERGUARD** - Only super effective moves deal damage
- **LEVITATE** - Immune to Ground moves (except Thousand Arrows)
- **STATIC** - 30% chance to paralyze on contact
- **THICKFAT** - Halves Fire and Ice move damage
- **FLASHFIRE** - Absorbs Fire moves and gains Flash Fire boost
- **LIQUIDOOZE** - Reverses HP drain effects
- **SHIELDDUST** - Prevents secondary effects on the user
- **WATERABSORB** - Absorbs Water moves and heals 1/4 HP
- **DRYSKIN** - Absorbs Water moves, takes 1.25x Fire damage
- **DAMP** - Prevents explosion moves
- **VOLTABSORB** - Absorbs Electric moves and heals 1/4 HP

## Damage Calculation Abilities
- **SCRAPPY** - Normal/Fighting moves hit Ghost types
- **MINDSEYE** - Normal/Fighting moves hit Ghost types
- **UNAWARE** - Ignores stat changes when calculating damage
- **MOLDBREAKER** - Ignores certain defensive abilities
- **CLOUDNINE** - Negates weather effects
- **AIRLOCK** - Negates weather effects
- **INFILTRATOR** - Bypasses screens and substitutes

## Status Protection Abilities
- **SHIELDSDOWN** - Protection when above 50% HP
- **PURIFYINGSALT** - Status immunity
- **COMATOSE** - Sleep immunity (always asleep)
- **LEAFGUARD** - Status immunity in Sun
- **WATERVEIL** - Burn immunity
- **WATERBUBBLE** - Burn immunity  
- **THERMALEXCHANGE** - Burn immunity
- **MAGMAARMOR** - Freeze immunity
- **INSOMNIA** - Sleep immunity
- **SWEETVEIL** - Sleep immunity
- **VITALSPIRIT** - Sleep immunity
- **LIMBER** - Paralysis immunity
- **IMMUNITY** - Poison immunity
- **PASTELVEIL** - Poison immunity
- **STURDY** - Prevents OHKO from full HP
- **PRESSURE** - Increases PP usage

## Special Mechanics
- **CONTRARY** - Reverses stat changes
- **CORROSION** - Can poison Steel and Poison types
- **MAGICGUARD** - Prevents indirect damage
- **NEUTRALIZINGGAS** - Suppresses all abilities

---

**Total: 100+ abilities with actual code implementation**

This represents the comprehensive set of abilities that have functional code in poke-engine, ranging from simple stat modifications to complex forme changes and interaction mechanics. The abilities are implemented across different battle phases including switch-in effects, end-of-turn effects, damage modifications, status protections, and special battle mechanics.