import React, { useState, useEffect } from 'react'
import PokemonCard from './PokemonCard'
import InstructionViewer from './InstructionViewer'
import { 
  BattleState, 
  MoveChoice, 
  InstructionResponse, 
  StateInstructions, 
  LegalOption,
  TurnHistory,
  BattleSession
} from '../types'

interface BattleInterfaceProps {
  sessionId: string
  battleState: BattleState
  onStateUpdate: (newState: BattleState) => void
}

const BattleInterface: React.FC<BattleInterfaceProps> = ({
  sessionId,
  battleState,
  onStateUpdate,
}) => {
  const [sideOneChoice, setSideOneChoice] = useState<MoveChoice>({
    choice_type: 'move',
    move_index: 0,
    target_positions: [{ side: 'two', slot: 0 }],
  })
  
  const [sideTwoChoice, setSideTwoChoice] = useState<MoveChoice>({
    choice_type: 'move',
    move_index: 0,
    target_positions: [{ side: 'one', slot: 0 }],
  })

  const [instructions, setInstructions] = useState<StateInstructions[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [showDebugState, setShowDebugState] = useState(false)
  const [previewState, setPreviewState] = useState<BattleState | null>(null)
  const [selectedInstructionSet, setSelectedInstructionSet] = useState<number | null>(null)
  const [turnHistory, setTurnHistory] = useState<TurnHistory[]>([])
  const [sideOneLegalOptions, setSideOneLegalOptions] = useState<LegalOption[]>([])
  const [sideTwoLegalOptions, setSideTwoLegalOptions] = useState<LegalOption[]>([])
  const [isLoadingOptions, setIsLoadingOptions] = useState(false)

  // Load legal options for both sides
  const loadLegalOptions = async () => {
    setIsLoadingOptions(true)
    try {
      const response = await fetch(`/api/battles/${sessionId}/legal-options`)
      const data = await response.json()
      
      if (data.success) {
        setSideOneLegalOptions(data.side_one_options || [])
        setSideTwoLegalOptions(data.side_two_options || [])
      }
    } catch (error) {
      console.error('Failed to load legal options:', error)
    } finally {
      setIsLoadingOptions(false)
    }
  }

  const generateInstructions = async () => {
    setIsLoading(true)
    try {
      const response = await fetch(`/api/battles/${sessionId}/instructions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          side_one_choice: sideOneChoice,
          side_two_choice: sideTwoChoice,
        }),
      })

      const data: InstructionResponse = await response.json()
      
      if (data.success) {
        setInstructions(data.instructions)
        setSelectedInstructionSet(null)
        setPreviewState(null)
        // Don't auto-update state - let user choose which instruction set to apply
      } else {
        alert(`Failed to generate instructions: ${data.error}`)
      }
    } catch (error) {
      console.error('Failed to generate instructions:', error)
      alert('Failed to generate instructions')
    } finally {
      setIsLoading(false)
    }
  }

  const previewInstructionSet = async (instructionSetIndex: number) => {
    try {
      const response = await fetch(`/api/battles/${sessionId}/preview`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ instruction_set_index: instructionSetIndex }),
      })
      
      const previewState: BattleState = await response.json()
      setPreviewState(previewState)
      setSelectedInstructionSet(instructionSetIndex)
    } catch (error) {
      console.error('Failed to preview instruction set:', error)
    }
  }

  const applyInstructionSet = async (instructionSetIndex: number) => {
    try {
      // Calculate expected turn number: current turn + 1, but account for turn history position
      const expectedTurnNumber = battleState.turn + 1;
      
      const response = await fetch(`/api/battles/${sessionId}/apply`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          instruction_set_index: instructionSetIndex,
          expected_turn_number: expectedTurnNumber
        }),
      })
      
      const newState: BattleState = await response.json()
      
      // Add to turn history
      const turnRecord: TurnHistory = {
        turn_number: battleState.turn,
        initial_state: battleState,
        side_one_choice: sideOneChoice,
        side_two_choice: sideTwoChoice,
        instructions: instructions,
        final_state: newState
      }
      
      setTurnHistory(prev => [...prev, turnRecord])
      onStateUpdate(newState)
      setInstructions([])
      setPreviewState(null)
      setSelectedInstructionSet(null)
      
      // Load new legal options for the next turn
      loadLegalOptions()
    } catch (error) {
      console.error('Failed to apply instruction set:', error)
    }
  }

  const goToPreviousTurn = async () => {
    if (turnHistory.length > 0) {
      const previousTurn = turnHistory[turnHistory.length - 1]
      
      try {
        // First update backend state to match the previous turn
        await fetch(`/api/battles/${sessionId}/state`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ new_state: previousTurn.initial_state })
        })
        
        // Then update frontend state
        onStateUpdate(previousTurn.initial_state)
        setTurnHistory(prev => prev.slice(0, -1))
        setInstructions([])
        setPreviewState(null)
        setSelectedInstructionSet(null)
        
        // Legal options will be refreshed by useEffect when battleState changes
      } catch (error) {
        console.error('Failed to sync backend state:', error)
        // Fallback to just updating frontend
        onStateUpdate(previousTurn.initial_state)
        setTurnHistory(prev => prev.slice(0, -1))
        setInstructions([])
        setPreviewState(null)
        setSelectedInstructionSet(null)
        loadLegalOptions()
      }
    }
  }

  const resetToTurn = async (turnIndex: number) => {
    if (turnIndex < turnHistory.length) {
      const targetTurn = turnHistory[turnIndex]
      
      try {
        // First update backend state to match the target turn
        await fetch(`/api/battles/${sessionId}/state`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ new_state: targetTurn.initial_state })
        })
        
        // Then update frontend state
        onStateUpdate(targetTurn.initial_state)
        setTurnHistory(prev => prev.slice(0, turnIndex))
        setInstructions([])
        setPreviewState(null)
        setSelectedInstructionSet(null)
        
        // Legal options will be refreshed by useEffect when battleState changes
      } catch (error) {
        console.error('Failed to sync backend state:', error)
        // Fallback to just updating frontend
        onStateUpdate(targetTurn.initial_state)
        setTurnHistory(prev => prev.slice(0, turnIndex))
        setInstructions([])
        setPreviewState(null)
        setSelectedInstructionSet(null)
        loadLegalOptions()
      }
    }
  }

  const sideOnePokemon = battleState.side_one.pokemon[battleState.side_one.active_pokemon_indices[0] || 0]
  const sideTwoPokemon = battleState.side_two.pokemon[battleState.side_two.active_pokemon_indices[0] || 0]
  
  const currentState = previewState || battleState
  const sideOnePreviewPokemon = currentState.side_one.pokemon[currentState.side_one.active_pokemon_indices[0] || 0]
  const sideTwoPreviewPokemon = currentState.side_two.pokemon[currentState.side_two.active_pokemon_indices[0] || 0]

  // Load legal options when component mounts or battle state changes
  useEffect(() => {
    loadLegalOptions()
  }, [sessionId, battleState.turn])

  const updateSideOneChoice = (option: LegalOption) => {
    setSideOneChoice(option.move_choice)
  }

  const updateSideTwoChoice = (option: LegalOption) => {
    setSideTwoChoice(option.move_choice)
  }

  return (
    <div className="grid">
      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Battle - Turn {currentState.turn}</h2>
          <div style={{ display: 'flex', gap: '20px', fontSize: '0.875rem', color: '#6b7280' }}>
            {currentState.weather !== 'NONE' && (
              <span>Weather: {currentState.weather}</span>
            )}
            {currentState.terrain !== 'NONE' && (
              <span>Terrain: {currentState.terrain}</span>
            )}
            {currentState.trick_room_active && (
              <span>Trick Room Active</span>
            )}
          </div>
        </div>

        <div style={{ marginBottom: '20px', display: 'flex', gap: '10px', justifyContent: 'center' }}>
          <button
            className="btn btn-secondary"
            onClick={goToPreviousTurn}
            disabled={turnHistory.length === 0}
          >
            ← Previous Turn
          </button>
          <span style={{ padding: '8px 16px', background: '#f3f4f6', borderRadius: '4px' }}>
            Turn {currentState.turn} {previewState && '(Preview)'}
          </span>
          <button
            className="btn btn-secondary"
            onClick={() => setPreviewState(null)}
            disabled={!previewState}
          >
            Clear Preview
          </button>
        </div>

        <div className="grid grid-2">
          <div>
            <h3 style={{ marginBottom: '16px', color: '#374151' }}>Side One</h3>
            {sideOnePreviewPokemon && (
              <PokemonCard pokemon={sideOnePreviewPokemon} showMoves={true} />
            )}
            {previewState && sideOnePokemon && (
              <div style={{ marginTop: '10px', padding: '10px', background: '#fef3c7', borderRadius: '6px' }}>
                <small><strong>Original:</strong> {sideOnePokemon.hp}/{sideOnePokemon.max_hp} HP</small>
              </div>
            )}
          </div>

          <div>
            <h3 style={{ marginBottom: '16px', color: '#374151' }}>Side Two</h3>
            {sideTwoPreviewPokemon && (
              <PokemonCard pokemon={sideTwoPreviewPokemon} showMoves={true} />
            )}
            {previewState && sideTwoPokemon && (
              <div style={{ marginTop: '10px', padding: '10px', background: '#fef3c7', borderRadius: '6px' }}>
                <small><strong>Original:</strong> {sideTwoPokemon.hp}/{sideTwoPokemon.max_hp} HP</small>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <h3 className="card-title">Move Selection</h3>
          <p className="card-description">Choose moves for both sides</p>
        </div>

        <div className="grid grid-2">
          <div>
            <h4 style={{ marginBottom: '12px', color: '#374151' }}>Side One Move</h4>
            {isLoadingOptions ? (
              <div>Loading legal options...</div>
            ) : (
              <div className="form-group">
                <label className="form-label">Legal Options</label>
                <select
                  className="form-select"
                  value={sideOneLegalOptions.findIndex(opt => 
                    opt.move_choice.choice_type === sideOneChoice.choice_type &&
                    (opt.move_choice.choice_type === 'switch' 
                      ? opt.move_choice.pokemon_index === sideOneChoice.pokemon_index
                      : opt.move_choice.move_index === sideOneChoice.move_index)
                  )}
                  onChange={(e) => {
                    const optionIndex = parseInt(e.target.value)
                    if (optionIndex >= 0 && optionIndex < sideOneLegalOptions.length) {
                      updateSideOneChoice(sideOneLegalOptions[optionIndex])
                    }
                  }}
                >
                  {sideOneLegalOptions.map((option, index) => (
                    <option key={index} value={index} disabled={option.is_disabled}>
                      {option.display_name}
                      {option.is_disabled && ` (${option.disabled_reason})`}
                    </option>
                  ))}
                </select>
                {sideOneLegalOptions.length === 0 && (
                  <div style={{ padding: '10px', background: '#fee2e2', borderRadius: '4px', marginTop: '8px' }}>
                    <small>No legal options available. Using fallback moves.</small>
                  </div>
                )}
              </div>
            )}
          </div>

          <div>
            <h4 style={{ marginBottom: '12px', color: '#374151' }}>Side Two Move</h4>
            {isLoadingOptions ? (
              <div>Loading legal options...</div>
            ) : (
              <div className="form-group">
                <label className="form-label">Legal Options</label>
                <select
                  className="form-select"
                  value={sideTwoLegalOptions.findIndex(opt => 
                    opt.move_choice.choice_type === sideTwoChoice.choice_type &&
                    (opt.move_choice.choice_type === 'switch' 
                      ? opt.move_choice.pokemon_index === sideTwoChoice.pokemon_index
                      : opt.move_choice.move_index === sideTwoChoice.move_index)
                  )}
                  onChange={(e) => {
                    const optionIndex = parseInt(e.target.value)
                    if (optionIndex >= 0 && optionIndex < sideTwoLegalOptions.length) {
                      updateSideTwoChoice(sideTwoLegalOptions[optionIndex])
                    }
                  }}
                >
                  {sideTwoLegalOptions.map((option, index) => (
                    <option key={index} value={index} disabled={option.is_disabled}>
                      {option.display_name}
                      {option.is_disabled && ` (${option.disabled_reason})`}
                    </option>
                  ))}
                </select>
                {sideTwoLegalOptions.length === 0 && (
                  <div style={{ padding: '10px', background: '#fee2e2', borderRadius: '4px', marginTop: '8px' }}>
                    <small>No legal options available. Using fallback moves.</small>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        <div style={{ textAlign: 'center', marginTop: '20px' }}>
          <button
            className="btn btn-primary"
            onClick={generateInstructions}
            disabled={isLoading}
            style={{ 
              fontSize: '1.1rem', 
              padding: '12px 32px',
              opacity: isLoading ? 0.6 : 1 
            }}
          >
            {isLoading ? '⏳ Generating...' : '⚡ Generate Instructions'}
          </button>
        </div>
      </div>

      {instructions.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3 className="card-title">Generated Instructions</h3>
            <p className="card-description">
              {instructions.length} instruction set(s) generated - Preview and apply
            </p>
          </div>
          
          <div style={{ marginBottom: '20px' }}>
            <div style={{ marginBottom: '15px' }}>
              <h4 style={{ marginBottom: '10px', color: '#374151' }}>Select Instruction Set</h4>
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                {instructions.map((instructionSet, index) => (
                  <button
                    key={index}
                    className={`btn ${selectedInstructionSet === index ? 'btn-primary' : 'btn-secondary'}`}
                    onClick={() => previewInstructionSet(index)}
                    style={{ fontSize: '0.875rem' }}
                  >
                    Set {index + 1} ({instructionSet.percentage.toFixed(1)}%)
                  </button>
                ))}
              </div>
            </div>
            
            {selectedInstructionSet !== null && (
              <div style={{ textAlign: 'center' }}>
                <button
                  className="btn btn-success"
                  onClick={() => applyInstructionSet(selectedInstructionSet)}
                  style={{ fontSize: '1rem', padding: '10px 24px' }}
                >
                  Apply Set {selectedInstructionSet + 1}
                </button>
              </div>
            )}
          </div>
          
          <InstructionViewer instructions={instructions} />
          
          {selectedInstructionSet !== null && instructions[selectedInstructionSet] && (
            <div style={{ marginTop: '20px', padding: '15px', background: '#f0f9ff', borderRadius: '8px', border: '1px solid #0ea5e9' }}>
              <h4>Selected Instruction Set {selectedInstructionSet + 1} Preview</h4>
              <InstructionViewer instructions={[instructions[selectedInstructionSet]]} />
            </div>
          )}
        </div>
      )}

      {turnHistory.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3 className="card-title">Turn History</h3>
            <p className="card-description">
              {turnHistory.length} turn(s) completed
            </p>
          </div>
          <div style={{ maxHeight: '200px', overflowY: 'auto' }}>
            {turnHistory.map((turn, index) => (
              <div key={index} style={{
                padding: '10px',
                margin: '5px 0',
                background: '#f9fafb',
                borderRadius: '6px',
                border: '1px solid #e5e7eb'
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span><strong>Turn {turn.turn_number}</strong></span>
                  <button
                    className="btn btn-secondary"
                    onClick={() => resetToTurn(index)}
                    style={{ fontSize: '0.75rem', padding: '4px 8px' }}
                  >
                    Reset to here
                  </button>
                </div>
                <div style={{ fontSize: '0.875rem', color: '#6b7280', marginTop: '5px' }}>
                  Side 1: {turn.side_one_choice.choice_type} | 
                  Side 2: {turn.side_two_choice.choice_type} | 
                  {turn.instructions.length} instruction set(s)
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-header">
          <h3 className="card-title">Debug: Battle State</h3>
          <button
            className="btn btn-secondary"
            onClick={() => setShowDebugState(!showDebugState)}
            style={{ fontSize: '0.875rem' }}
          >
            {showDebugState ? 'Hide' : 'Show'} Raw State
          </button>
        </div>
        {showDebugState && (
          <div style={{
            background: '#f9fafb',
            border: '1px solid #e5e7eb',
            borderRadius: '6px',
            padding: '16px',
            overflow: 'auto'
          }}>
            <pre style={{
              fontSize: '12px',
              lineHeight: '1.4',
              margin: 0,
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word'
            }}>
              {JSON.stringify(currentState, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  )
}

export default BattleInterface