import { useState } from 'react'
import PokemonBuilder from './components/PokemonBuilder'
import BattleInterface from './components/BattleInterface'
import { Pokemon, BattleState } from './types'

interface Battle {
  sessionId: string
  state: BattleState
}

function App() {
  const [currentView, setCurrentView] = useState<'builder' | 'battle'>('builder')
  const [battle, setBattle] = useState<Battle | null>(null)
  const [sideOnePokemon, setSideOnePokemon] = useState<Pokemon[]>([])
  const [sideTwoPokemon, setSideTwoPokemon] = useState<Pokemon[]>([])

  const createBattle = async () => {
    if (sideOnePokemon.length === 0 || sideTwoPokemon.length === 0) {
      alert('Please add at least one Pokemon to each side')
      return
    }

    try {
      const response = await fetch('/api/battles', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          format: {
            name: 'Singles',
            format_type: 'Singles',
            generation: 'Gen9',
            active_pokemon_count: 1,
          },
          side_one: {
            pokemon: sideOnePokemon,
            active_pokemon_indices: [0],
            side_conditions: {},
          },
          side_two: {
            pokemon: sideTwoPokemon,
            active_pokemon_indices: [0],
            side_conditions: {},
          },
        }),
      })

      const data = await response.json()
      setBattle({
        sessionId: data.session_id,
        state: data.battle_state,
      })
      setCurrentView('battle')
    } catch (error) {
      console.error('Failed to create battle:', error)
      alert('Failed to create battle')
    }
  }

  const updateBattleState = (newState: BattleState) => {
    if (battle) {
      setBattle({
        ...battle,
        state: newState,
      })
    }
  }

  return (
    <div className="container">
      <div style={{ textAlign: 'center', marginBottom: '40px' }}>
        <h1 style={{ 
          fontSize: '3rem', 
          color: 'white', 
          marginBottom: '10px',
          textShadow: '2px 2px 4px rgba(0,0,0,0.3)'
        }}>
          🌺 Tapu Simu
        </h1>
        <p style={{ 
          fontSize: '1.2rem', 
          color: 'rgba(255,255,255,0.9)',
          textShadow: '1px 1px 2px rgba(0,0,0,0.3)'
        }}>
          Format-aware Pokemon Battle Simulator Testing UI
        </p>
      </div>

      <div style={{ marginBottom: '20px' }}>
        <div style={{ display: 'flex', gap: '10px', justifyContent: 'center' }}>
          <button 
            className={`btn ${currentView === 'builder' ? 'btn-primary' : 'btn-secondary'}`}
            onClick={() => setCurrentView('builder')}
          >
            🏗️ Team Builder
          </button>
          <button 
            className={`btn ${currentView === 'battle' ? 'btn-primary' : 'btn-secondary'}`}
            onClick={() => setCurrentView('battle')}
            disabled={!battle}
          >
            ⚔️ Battle Interface
          </button>
        </div>
      </div>

      {currentView === 'builder' && (
        <PokemonBuilder
          sideOnePokemon={sideOnePokemon}
          setSideOnePokemon={setSideOnePokemon}
          sideTwoPokemon={sideTwoPokemon}
          setSideTwoPokemon={setSideTwoPokemon}
          onCreateBattle={createBattle}
        />
      )}

      {currentView === 'battle' && battle && (
        <BattleInterface
          sessionId={battle.sessionId}
          battleState={battle.state}
          onStateUpdate={updateBattleState}
        />
      )}
    </div>
  )
}

export default App