//! # Battle Environment Integration Tests
//! 
//! Tests for the complete battle environment functionality

use tapu_simu::{
    BattleFormat, State, RandomPlayer, FirstMovePlayer, 
    BattleEnvironment, run_battle_from_state
};
use tapu_simu::core::move_choice::PokemonIndex;

#[test]
fn test_battle_environment_creation() {
    let player_one = Box::new(RandomPlayer::new("Player1".to_string()));
    let player_two = Box::new(FirstMovePlayer::new("Player2".to_string()));
    
    let env = BattleEnvironment::new(player_one, player_two, 100, false);
    
    assert_eq!(env.max_turns, 100);
    assert!(!env.verbose);
}

#[test]
fn test_battle_from_state() {
    let state = State::new(BattleFormat::gen9_ou());
    let player_one = Box::new(RandomPlayer::new("RandomBot".to_string()));
    let player_two = Box::new(FirstMovePlayer::new("FirstBot".to_string()));
    
    let result = run_battle_from_state(state, player_one, player_two, 50, false);
    
    // The battle should complete (either with a winner or reaching turn limit)
    assert!(result.turn_count <= 50);
    assert!(result.turn_history.len() <= 50);
}

#[test]
fn test_battle_result_structure() {
    let state = State::new(BattleFormat::gen9_ou());
    let player_one = Box::new(RandomPlayer::new("P1".to_string()));
    let player_two = Box::new(RandomPlayer::new("P2".to_string()));
    
    let result = run_battle_from_state(state, player_one, player_two, 10, false);
    
    // Verify the result structure is properly populated
    assert!(result.turn_count > 0);
    assert_eq!(result.turn_history.len(), result.turn_count);
    
    // Each turn should have proper information
    for turn_info in &result.turn_history {
        assert!(turn_info.turn_number > 0);
        assert!(turn_info.turn_number <= result.turn_count);
    }
}

#[test]
fn test_different_player_types() {
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test RandomPlayer vs FirstMovePlayer
    let player_one = Box::new(RandomPlayer::new("Random".to_string()));
    let player_two = Box::new(FirstMovePlayer::new("First".to_string()));
    
    let result = run_battle_from_state(state, player_one, player_two, 20, false);
    
    // Should complete without panicking
    assert!(result.turn_count <= 20);
}

#[cfg(test)]
mod player_tests {
    use super::*;
    use tapu_simu::{DamageMaximizer, MoveChoice, SideReference};

    #[test]
    fn test_player_names() {
        let random = RandomPlayer::new("RandomBot".to_string());
        let first = FirstMovePlayer::new("FirstBot".to_string());
        let damage = DamageMaximizer::new("DamageBot".to_string());
        
        assert_eq!(random.name(), "RandomBot");
        assert_eq!(first.name(), "FirstBot");
        assert_eq!(damage.name(), "DamageBot");
    }

    #[test]
    fn test_first_move_player_deterministic() {
        let player = FirstMovePlayer::new("Test".to_string());
        let state = State::new(BattleFormat::gen9_ou());
        let options = vec![MoveChoice::None, MoveChoice::Switch(PokemonIndex::P0)];
        
        let choice1 = player.choose_move(&state, SideReference::SideOne, &options);
        let choice2 = player.choose_move(&state, SideReference::SideOne, &options);
        
        // FirstMovePlayer should always choose the same (first) option
        assert_eq!(choice1, choice2);
        assert_eq!(choice1, MoveChoice::None); // First option
    }
}