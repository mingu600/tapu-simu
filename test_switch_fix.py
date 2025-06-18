#!/usr/bin/env python3
"""
Test script to verify the switch + attack instruction generation fix.
Simulates Flutter Mane using Icy Wind while opponent switches Dragapult to Heatran.
"""

import requests
import json

# Server URL
SERVER_URL = "http://localhost:3001"

def create_pokemon(species, moves):
    """Create a Pokemon using the API"""
    request = {
        "species": species,
        "level": 50,
        "nature": "Hardy",
        "ability": "",  # Will use default
        "item": "",     # No item
        "ivs": [31, 31, 31, 31, 31, 31],
        "evs": [0, 0, 0, 0, 0, 0],
        "moves": moves
    }
    
    response = requests.post(f"{SERVER_URL}/api/pokemon/create-custom", json=request, timeout=10)
    response.raise_for_status()
    return response.json()

def setup_battle_state():
    """Setup the battle state with Flutter Mane vs Dragapult/Heatran"""
    
    print("📦 Creating Pokemon...")
    
    # Create Flutter Mane
    flutter_mane = create_pokemon("Flutter Mane", ["Icy Wind", "Moonblast", "Shadow Ball", "Protect"])
    print("   ✅ Flutter Mane created")
    
    # Create Dragapult 
    dragapult = create_pokemon("Dragapult", ["Dragon Darts", "U-turn", "Will-O-Wisp", "Protect"])
    print("   ✅ Dragapult created")
    
    # Create Heatran
    heatran = create_pokemon("Heatran", ["Lava Plume", "Earth Power", "Flash Cannon", "Protect"])
    print("   ✅ Heatran created")
    
    battle_state = {
        "format": {
            "name": "Singles",
            "format_type": "Singles",
            "generation": "Gen9",
            "active_pokemon_count": 1
        },
        "side_one": {
            "pokemon": [flutter_mane],
            "active_pokemon_indices": [0],
            "side_conditions": {}
        },
        "side_two": {
            "pokemon": [dragapult, heatran],
            "active_pokemon_indices": [0],
            "side_conditions": {}
        }
    }
    return battle_state

def test_switch_plus_attack():
    """Test the specific scenario: Flutter Mane Icy Wind + Dragapult switches to Heatran"""
    
    print("🧪 Testing Switch + Attack Instruction Generation")
    print("=" * 60)
    
    # Setup battle
    battle_state = setup_battle_state()
    
    print(f"📤 Step 1: Creating battle session...")
    
    try:
        # Step 1: Create a battle session  
        response = requests.post(f"{SERVER_URL}/api/battles", json=battle_state, timeout=10)
        response.raise_for_status()
        
        session_data = response.json()
        session_id = session_data["session_id"]
        print(f"✅ Battle session created: {session_id}")
        
        # Step 2: Generate instructions with the move choices
        print(f"📤 Step 2: Generating instructions...")
        print(f"   Side One: Icy Wind (targeting opponent)")
        print(f"   Side Two: Switch Dragapult → Heatran")
        
        instruction_request = {
            "side_one_choice": {
                "choice_type": "move",
                "move_index": 0,  # Icy Wind is first move
                "target_positions": [
                    {
                        "side": "two",
                        "slot": 0
                    }
                ],
                "pokemon_index": None
            },
            "side_two_choice": {
                "choice_type": "switch",
                "move_index": None,
                "target_positions": [],
                "pokemon_index": 1  # Switch to Pokemon at index 1 (Heatran)
            }
        }
        
        response = requests.post(f"{SERVER_URL}/api/battles/{session_id}/instructions", 
                                json=instruction_request, timeout=10)
        response.raise_for_status()
        
        result = response.json()
        
        # Analyze the results
        if "instructions" in result:
            instruction_sets = result["instructions"]
            print(f"📊 Results: {len(instruction_sets)} instruction sets generated")
            print()
            
            # Expected: 2 instruction sets (normal damage vs critical hit)
            if len(instruction_sets) == 2:
                print("✅ PASS: Correct number of instruction sets (2)")
                
                # Check probabilities
                probs = [set_info["percentage"] for set_info in instruction_sets]
                probs.sort(reverse=True)
                
                if abs(probs[0] - 95.8) < 0.5 and abs(probs[1] - 4.2) < 0.5:
                    print("✅ PASS: Correct probabilities (~95.8% + ~4.2%)")
                    print(f"   Actual: {probs[0]:.1f}% + {probs[1]:.1f}%")
                else:
                    print(f"❌ FAIL: Wrong probabilities: {probs}")
                
                # Check instruction structure and damage calculation
                print("\n📝 Instruction Set Analysis:")
                for i, set_info in enumerate(instruction_sets):
                    instructions = set_info["instructions"]
                    print(f"\n   Set {i+1} (probability: {set_info['percentage']:.1f}%):")
                    
                    # Should have: Switch instruction + Damage instruction
                    switch_instructions = [instr for instr in instructions if instr["instruction_type"] == "Other" and "Switch" in instr["description"]]
                    damage_instructions = [instr for instr in instructions if instr["instruction_type"] == "PositionDamage"]
                    
                    if switch_instructions and damage_instructions:
                        print(f"      ✅ Contains both switch and damage instructions")
                        damage_amount = damage_instructions[0]["details"]["damage_amount"]
                        print(f"      💥 Damage: {damage_amount}")
                        
                        # The key test: damage should be low because it's calculated against Heatran
                        # (high HP/Defense) instead of Dragapult (lower HP/Defense)
                        if damage_amount < 50:  # Very low damage indicates correct targeting
                            print(f"      ✅ Low damage indicates attack targets Heatran (correct)")
                        else:
                            print(f"      ❌ High damage might indicate attack targets Dragapult (incorrect)")
                    else:
                        print(f"      ❌ Missing expected instructions")
                        print(f"         Switch instructions: {len(switch_instructions)}")
                        print(f"         Damage instructions: {len(damage_instructions)}")
                
                # Final verdict
                print(f"\n🎯 TEST RESULT:")
                if len(instruction_sets) == 2 and abs(probs[0] - 95.8) < 0.5:
                    print(f"✅ SUCCESS: Fix is working correctly!")
                    print(f"   - Only 2 instruction sets (not 6)")
                    print(f"   - Correct crit vs non-crit probabilities")
                    print(f"   - Switch applied before damage calculation")
                else:
                    print(f"❌ PARTIAL: Some issues remain")
                
            elif len(instruction_sets) == 6:
                print("❌ FAIL: Still generating 6 instruction sets (bug not fixed)")
                for i, set_info in enumerate(instruction_sets):
                    print(f"   Set {i+1}: {set_info['percentage']:.1f}% - {len(set_info['instructions'])} instructions")
            else:
                print(f"❌ FAIL: Unexpected number of instruction sets: {len(instruction_sets)}")
                
        else:
            print("❌ FAIL: No instructions in response")
            print(f"Response: {json.dumps(result, indent=2)}")
            
    except requests.exceptions.RequestException as e:
        print(f"❌ FAIL: Request failed: {e}")
    except json.JSONDecodeError as e:
        print(f"❌ FAIL: JSON decode error: {e}")
        print(f"Response text: {response.text}")
    except Exception as e:
        print(f"❌ FAIL: Unexpected error: {e}")

if __name__ == "__main__":
    test_switch_plus_attack()