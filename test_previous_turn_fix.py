#!/usr/bin/env python3
"""
Test script to verify the previous turn fix is working.
This tests that when going to previous turn, the move options properly sync with the battle state.
"""

import requests
import json

# Server URL
SERVER_URL = "http://localhost:3001"

def create_test_pokemon(species, moves):
    """Create a Pokemon using the API"""
    request = {
        "species": species,
        "level": 50,
        "nature": "Hardy",
        "ability": "",
        "item": "",
        "ivs": [31, 31, 31, 31, 31, 31],
        "evs": [0, 0, 0, 0, 0, 0],
        "moves": moves
    }
    
    response = requests.post(f"{SERVER_URL}/api/pokemon/create-custom", json=request, timeout=10)
    response.raise_for_status()
    return response.json()

def test_previous_turn_fix():
    """Test that previous turn properly syncs move options"""
    
    print("🧪 Testing Previous Turn Move Options Sync")
    print("=" * 50)
    
    try:
        # Create Pokemon
        print("📦 Creating Pokemon...")
        flutter_mane = create_test_pokemon("Flutter Mane", ["Icy Wind", "Moonblast", "Shadow Ball", "Protect"])
        dragapult = create_test_pokemon("Dragapult", ["Dragon Darts", "U-turn", "Will-O-Wisp", "Protect"])
        heatran = create_test_pokemon("Heatran", ["Lava Plume", "Earth Power", "Flash Cannon", "Protect"])
        
        # Create battle
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
        
        response = requests.post(f"{SERVER_URL}/api/battles", json=battle_state, timeout=10)
        response.raise_for_status()
        session_data = response.json()
        session_id = session_data["session_id"]
        initial_battle_state = session_data["battle_state"]  # Use the actual returned state
        print(f"✅ Battle session created: {session_id}")
        
        # Get initial legal options to verify Dragapult moves are shown
        print("\n📋 Step 1: Getting initial legal options...")
        response = requests.get(f"{SERVER_URL}/api/battles/{session_id}/legal-options", timeout=10)
        response.raise_for_status()
        initial_options = response.json()
        
        side_two_moves = [opt for opt in initial_options["side_two_options"] if opt["choice_type"] == "move"]
        print(f"   Side Two Move Options: {[opt['display_name'] for opt in side_two_moves]}")
        
        # Should see Dragapult's moves (Dragon Darts, U-turn, Will-O-Wisp, Protect)
        expected_dragapult_moves = ["Dragon Darts", "U-turn", "Will-O-Wisp", "Protect"]
        actual_moves = [opt['display_name'] for opt in side_two_moves]
        
        # Extract just the move names (before the arrow)
        move_names = [move.split(' →')[0] for move in actual_moves]
        
        if all(move in move_names for move in expected_dragapult_moves):
            print("   ✅ Initial state: Dragapult moves correctly shown")
        else:
            print(f"   ❌ Initial state: Expected {expected_dragapult_moves}, got {move_names}")
            return
            
        # Execute a turn: Flutter Mane uses Icy Wind, Player 2 switches to Heatran
        print("\n⚔️ Step 2: Executing turn (Icy Wind vs Switch to Heatran)...")
        instruction_request = {
            "side_one_choice": {
                "choice_type": "move",
                "move_index": 0,
                "target_positions": [{"side": "two", "slot": 0}],
                "pokemon_index": None
            },
            "side_two_choice": {
                "choice_type": "switch",
                "move_index": None,
                "target_positions": [],
                "pokemon_index": 1
            }
        }
        
        response = requests.post(f"{SERVER_URL}/api/battles/{session_id}/instructions", 
                                json=instruction_request, timeout=10)
        response.raise_for_status()
        
        # Apply the first instruction set (should have switch + damage)
        apply_request = {"instruction_set_index": 0}
        response = requests.post(f"{SERVER_URL}/api/battles/{session_id}/apply",
                                json=apply_request, timeout=10)
        response.raise_for_status()
        print("   ✅ Turn executed, Heatran should now be active")
        
        # Get legal options after the turn - should show Heatran's moves
        print("\n📋 Step 3: Getting legal options after turn...")
        response = requests.get(f"{SERVER_URL}/api/battles/{session_id}/legal-options", timeout=10)
        response.raise_for_status()
        after_turn_options = response.json()
        
        side_two_moves_after = [opt for opt in after_turn_options["side_two_options"] if opt["choice_type"] == "move"]
        print(f"   Side Two Move Options: {[opt['display_name'] for opt in side_two_moves_after]}")
        
        # Should see Heatran's moves (Lava Plume, Earth Power, Flash Cannon, Protect)
        expected_heatran_moves = ["Lava Plume", "Earth Power", "Flash Cannon", "Protect"]
        actual_moves_after = [opt['display_name'] for opt in side_two_moves_after]
        
        # Extract just the move names (before the arrow)
        move_names_after = [move.split(' →')[0] for move in actual_moves_after]
        
        if all(move in move_names_after for move in expected_heatran_moves):
            print("   ✅ After turn: Heatran moves correctly shown")
        else:
            print(f"   ❌ After turn: Expected {expected_heatran_moves}, got {move_names_after}")
            return
            
        # Now test the fix: Use new PUT endpoint to go back to previous turn
        print("\n⏪ Step 4: Going to previous turn using PUT endpoint...")
        
        # Update backend state to initial state
        put_request = {"new_state": initial_battle_state}
        response = requests.put(f"{SERVER_URL}/api/battles/{session_id}/state",
                               json=put_request, timeout=10)
        response.raise_for_status()
        print("   ✅ Backend state updated to previous turn")
        
        # Get legal options after going back - should show Dragapult's moves again
        print("\n📋 Step 5: Getting legal options after going back...")
        response = requests.get(f"{SERVER_URL}/api/battles/{session_id}/legal-options", timeout=10)
        response.raise_for_status()
        final_options = response.json()
        
        side_two_moves_final = [opt for opt in final_options["side_two_options"] if opt["choice_type"] == "move"]
        print(f"   Side Two Move Options: {[opt['display_name'] for opt in side_two_moves_final]}")
        
        # Should see Dragapult's moves again (Dragon Darts, U-turn, Will-O-Wisp, Protect)
        actual_moves_final = [opt['display_name'] for opt in side_two_moves_final]
        
        # Extract just the move names (before the arrow)
        move_names_final = [move.split(' →')[0] for move in actual_moves_final]
        
        if all(move in move_names_final for move in expected_dragapult_moves):
            print("   ✅ After going back: Dragapult moves correctly shown")
            print("\n🎯 TEST RESULT: ✅ SUCCESS - Previous turn fix is working!")
            print("   - Backend and frontend states are properly synced")
            print("   - Move options correctly update when going to previous turn")
        else:
            print(f"   ❌ After going back: Expected {expected_dragapult_moves}, got {move_names_final}")
            print("\n🎯 TEST RESULT: ❌ FAIL - Move options still desync")
        
    except Exception as e:
        print(f"❌ Test failed with error: {e}")

if __name__ == "__main__":
    test_previous_turn_fix()