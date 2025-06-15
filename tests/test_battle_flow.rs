//! Integration tests for basic battle flow
//! 
//! Tests battle initialization, choice making, and turn execution

use tapu_simu::battle::Battle;
use tapu_simu::side::{SideId, ChosenAction};
use tapu_simu::dex::ShowdownDex;

#[test]
fn test_battle_initialization() {
    let battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    assert_eq!(battle.state().turn, 0);
    assert_eq!(battle.state().sides.len(), 2);
    assert!(!battle.state().ended);
}

#[test]
fn test_battle_choice_making() {
    let mut battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    // Add choices using factory methods
    battle.add_choice(SideId::P1, vec![ChosenAction::attack()]).expect("Failed to add P1 choice");
    battle.add_choice(SideId::P2, vec![ChosenAction::attack()]).expect("Failed to add P2 choice");
    
    assert!(battle.state().all_choices_made());
}

#[test]
fn test_battle_turn_execution() {
    let mut battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    // Initial state
    assert_eq!(battle.state().turn, 0);
    assert!(battle.state().queue.is_empty());
    
    // Add choices using factory methods
    battle.add_choice(SideId::P1, vec![ChosenAction::attack()]).expect("Failed to add P1 choice");
    battle.add_choice(SideId::P2, vec![ChosenAction::attack()]).expect("Failed to add P2 choice");
    
    // Execute a step - should start turn 1
    let ended = battle.step().expect("Failed to execute step");
    assert!(!ended);
    assert_eq!(battle.state().turn, 1);
}

#[test]
fn test_battle_state_serialization() {
    let battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    // Test binary serialization
    let binary = battle.serialize_state().expect("Failed to serialize state");
    assert!(binary.len() > 0);
    
    // Create new battle and deserialize
    let mut new_battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    new_battle.deserialize_state(&binary).expect("Failed to deserialize state");
    
    // States should match
    assert_eq!(battle.state().turn, new_battle.state().turn);
    assert_eq!(battle.state().sides.len(), new_battle.state().sides.len());
}

#[test]
fn test_battle_undo_functionality() {
    let mut battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    // Execute at least one turn to have something to undo
    battle.add_choice(SideId::P1, vec![ChosenAction::attack()]).expect("Failed to add P1 choice");
    battle.add_choice(SideId::P2, vec![ChosenAction::attack()]).expect("Failed to add P2 choice");
    battle.step().expect("Failed to execute step");
    
    // The turn should have advanced
    let current_turn = battle.state().turn;
    assert!(current_turn >= 1);
    
    // Try to undo to turn 0 (before any moves)
    if current_turn > 0 {
        battle.undo_to_turn(0).expect("Failed to undo to turn 0");
        assert_eq!(battle.state().turn, 0);
    }
    
    // Try to undo to non-existent turn
    assert!(battle.undo_to_turn(99).is_err());
}

#[test]
fn test_battle_end_detection() {
    let battle = Battle::quick_test_battle(ShowdownDex::test_dex()).expect("Failed to create battle");
    
    // For now, let's just verify the battle hasn't ended yet
    assert!(!battle.state().ended);
    assert_eq!(battle.state().winner, None);
    
    // In a real implementation, we would execute moves that cause fainting
    // This test demonstrates the structure is in place
}

