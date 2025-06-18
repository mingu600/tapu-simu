import React, { useState, useEffect } from 'react'
import PokemonCard from './PokemonCard'
import PokemonEditor from './PokemonEditor'
import ShowdownImporter from './ShowdownImporter'
import { Pokemon } from '../types'

interface PokemonBuilderProps {
  sideOnePokemon: Pokemon[]
  setSideOnePokemon: (pokemon: Pokemon[]) => void
  sideTwoPokemon: Pokemon[]
  setSideTwoPokemon: (pokemon: Pokemon[]) => void
  onCreateBattle: () => void
}

interface SearchDropdown {
  isOpen: boolean
  searchTerm: string
}

const PokemonBuilder: React.FC<PokemonBuilderProps> = ({
  sideOnePokemon,
  setSideOnePokemon,
  sideTwoPokemon,
  setSideTwoPokemon,
  onCreateBattle,
}) => {
  const [availableSpecies, setAvailableSpecies] = useState<string[]>([])
  const [presets, setPresets] = useState<[string, string][]>([])
  const [editingPokemon, setEditingPokemon] = useState<{ pokemon: Pokemon; side: 'one' | 'two'; index: number } | null>(null)
  const [showdownImporter, setShowdownImporter] = useState<{ show: boolean; side: 'one' | 'two' }>({ show: false, side: 'one' })
  
  // Search dropdowns for each side
  const [sideOneSearch, setSideOneSearch] = useState<SearchDropdown>({ isOpen: false, searchTerm: '' })
  const [sideTwoSearch, setSideTwoSearch] = useState<SearchDropdown>({ isOpen: false, searchTerm: '' })

  useEffect(() => {
    // Fetch available Pokemon and presets
    Promise.all([
      fetch('/api/pokemon').then(r => r.json()),
      fetch('/api/presets/pokemon').then(r => r.json()),
    ]).then(([species, presetData]) => {
      setAvailableSpecies(species)
      setPresets(presetData)
    }).catch(console.error)

    // Close dropdowns on click outside
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Element
      if (!target.closest('.form-group')) {
        setSideOneSearch(prev => ({ ...prev, isOpen: false }))
        setSideTwoSearch(prev => ({ ...prev, isOpen: false }))
      }
    }

    document.addEventListener('click', handleClickOutside)
    return () => document.removeEventListener('click', handleClickOutside)
  }, [])

  const addPokemon = async (side: 'one' | 'two', species: string) => {
    try {
      const response = await fetch('/api/pokemon/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ species, level: 50 }),
      })
      const pokemon = await response.json()
      
      if (side === 'one') {
        setSideOnePokemon([...sideOnePokemon, pokemon])
        setSideOneSearch({ isOpen: false, searchTerm: '' })
      } else {
        setSideTwoPokemon([...sideTwoPokemon, pokemon])
        setSideTwoSearch({ isOpen: false, searchTerm: '' })
      }
    } catch (error) {
      console.error('Failed to create Pokemon:', error)
    }
  }

  const editPokemon = (side: 'one' | 'two', index: number) => {
    const pokemon = side === 'one' ? sideOnePokemon[index] : sideTwoPokemon[index]
    if (pokemon) {
      setEditingPokemon({ pokemon, side, index })
    }
  }

  const updatePokemon = (updatedPokemon: Pokemon) => {
    if (!editingPokemon) return
    
    const { side, index } = editingPokemon
    
    if (side === 'one') {
      const newTeam = [...sideOnePokemon]
      newTeam[index] = updatedPokemon
      setSideOnePokemon(newTeam)
    } else {
      const newTeam = [...sideTwoPokemon]
      newTeam[index] = updatedPokemon
      setSideTwoPokemon(newTeam)
    }
  }

  const getFilteredSpecies = (searchTerm: string) => {
    return availableSpecies.filter(species => 
      species.toLowerCase().includes(searchTerm.toLowerCase())
    ).slice(0, 20)
  }

  const removePokemon = (side: 'one' | 'two', index: number) => {
    if (side === 'one') {
      setSideOnePokemon(sideOnePokemon.filter((_, i) => i !== index))
    } else {
      setSideTwoPokemon(sideTwoPokemon.filter((_, i) => i !== index))
    }
  }

  const loadPresetTeam = async (side: 'one' | 'two', presetName: string) => {
    try {
      const response = await fetch(`/api/presets/teams/${presetName}`)
      const team = await response.json()
      
      if (side === 'one') {
        setSideOnePokemon(team)
      } else {
        setSideTwoPokemon(team)
      }
    } catch (error) {
      console.error('Failed to load preset team:', error)
    }
  }

  const handleShowdownImport = (side: 'one' | 'two', pokemon: Pokemon[]) => {
    if (side === 'one') {
      setSideOnePokemon(pokemon)
    } else {
      setSideTwoPokemon(pokemon)
    }
  }

  return (
    <div className="grid grid-2">
      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Side One Team</h2>
          <p className="card-description">Build your first team</p>
        </div>
        
        <div style={{ marginBottom: '20px' }}>
          <div className="form-group">
            <label className="form-label">Add Pokemon</label>
            <div style={{ position: 'relative' }}>
              <input
                type="text"
                className="form-control"
                placeholder="Search Pokemon..."
                value={sideOneSearch.searchTerm}
                onChange={(e) => setSideOneSearch({ searchTerm: e.target.value, isOpen: true })}
                onFocus={() => setSideOneSearch({ ...sideOneSearch, isOpen: true })}
              />
              {sideOneSearch.isOpen && sideOneSearch.searchTerm && (
                <div style={{
                  position: 'absolute', top: '100%', left: 0, right: 0,
                  background: 'white', border: '1px solid #ddd', borderRadius: '6px',
                  maxHeight: '200px', overflowY: 'auto', zIndex: 1000,
                  boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
                }}>
                  {getFilteredSpecies(sideOneSearch.searchTerm).map(species => (
                    <div
                      key={species}
                      onClick={() => addPokemon('one', species)}
                      style={{
                        padding: '8px 12px', cursor: 'pointer',
                        borderBottom: '1px solid #eee'
                      }}
                      onMouseEnter={(e) => e.currentTarget.style.background = '#f1f1f1'}
                      onMouseLeave={(e) => e.currentTarget.style.background = 'white'}
                    >
                      {species}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Quick Add Pokemon</label>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              {presets.slice(0, 5).map(([species]) => (
                <button
                  key={species}
                  className="btn btn-secondary"
                  onClick={() => addPokemon('one', species)}
                  style={{ fontSize: '0.75rem', padding: '4px 8px' }}
                >
                  {species}
                </button>
              ))}
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Load Preset Team</label>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                className="btn btn-secondary"
                onClick={() => loadPresetTeam('one', 'basic')}
              >
                Basic Team
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => loadPresetTeam('one', 'vgc2024')}
              >
                VGC 2024 Team
              </button>
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Import from Pokemon Showdown</label>
            <button
              className="btn btn-primary"
              onClick={() => setShowdownImporter({ show: true, side: 'one' })}
              style={{ width: '100%' }}
            >
              📋 Import Showdown Team
            </button>
          </div>
        </div>

        <div className="grid">
          {sideOnePokemon.map((pokemon, index) => (
            <div key={index} style={{ position: 'relative' }}>
              <PokemonCard
                pokemon={pokemon}
                onRemove={() => removePokemon('one', index)}
              />
              <button
                className="btn btn-primary"
                onClick={() => editPokemon('one', index)}
                style={{ 
                  position: 'absolute', 
                  top: '8px', 
                  right: '8px',
                  padding: '4px 8px',
                  fontSize: '0.75rem'
                }}
              >
                Edit
              </button>
            </div>
          ))}
          {sideOnePokemon.length === 0 && (
            <div style={{ 
              textAlign: 'center', 
              color: '#6b7280', 
              padding: '40px',
              border: '2px dashed #d1d5db',
              borderRadius: '8px'
            }}>
              Add Pokemon to start building your team
            </div>
          )}
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Side Two Team</h2>
          <p className="card-description">Build your opponent team</p>
        </div>
        
        <div style={{ marginBottom: '20px' }}>
          <div className="form-group">
            <label className="form-label">Add Pokemon</label>
            <div style={{ position: 'relative' }}>
              <input
                type="text"
                className="form-control"
                placeholder="Search Pokemon..."
                value={sideTwoSearch.searchTerm}
                onChange={(e) => setSideTwoSearch({ searchTerm: e.target.value, isOpen: true })}
                onFocus={() => setSideTwoSearch({ ...sideTwoSearch, isOpen: true })}
              />
              {sideTwoSearch.isOpen && sideTwoSearch.searchTerm && (
                <div style={{
                  position: 'absolute', top: '100%', left: 0, right: 0,
                  background: 'white', border: '1px solid #ddd', borderRadius: '6px',
                  maxHeight: '200px', overflowY: 'auto', zIndex: 1000,
                  boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
                }}>
                  {getFilteredSpecies(sideTwoSearch.searchTerm).map(species => (
                    <div
                      key={species}
                      onClick={() => addPokemon('two', species)}
                      style={{
                        padding: '8px 12px', cursor: 'pointer',
                        borderBottom: '1px solid #eee'
                      }}
                      onMouseEnter={(e) => e.currentTarget.style.background = '#f1f1f1'}
                      onMouseLeave={(e) => e.currentTarget.style.background = 'white'}
                    >
                      {species}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Quick Add Pokemon</label>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              {presets.slice(5, 10).map(([species]) => (
                <button
                  key={species}
                  className="btn btn-secondary"
                  onClick={() => addPokemon('two', species)}
                  style={{ fontSize: '0.75rem', padding: '4px 8px' }}
                >
                  {species}
                </button>
              ))}
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Load Preset Team</label>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                className="btn btn-secondary"
                onClick={() => loadPresetTeam('two', 'basic')}
              >
                Basic Team
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => loadPresetTeam('two', 'ou2024')}
              >
                OU 2024 Team
              </button>
            </div>
          </div>
          
          <div className="form-group">
            <label className="form-label">Import from Pokemon Showdown</label>
            <button
              className="btn btn-primary"
              onClick={() => setShowdownImporter({ show: true, side: 'two' })}
              style={{ width: '100%' }}
            >
              📋 Import Showdown Team
            </button>
          </div>
        </div>

        <div className="grid">
          {sideTwoPokemon.map((pokemon, index) => (
            <div key={index} style={{ position: 'relative' }}>
              <PokemonCard
                pokemon={pokemon}
                onRemove={() => removePokemon('two', index)}
              />
              <button
                className="btn btn-primary"
                onClick={() => editPokemon('two', index)}
                style={{ 
                  position: 'absolute', 
                  top: '8px', 
                  right: '8px',
                  padding: '4px 8px',
                  fontSize: '0.75rem'
                }}
              >
                Edit
              </button>
            </div>
          ))}
          {sideTwoPokemon.length === 0 && (
            <div style={{ 
              textAlign: 'center', 
              color: '#6b7280', 
              padding: '40px',
              border: '2px dashed #d1d5db',
              borderRadius: '8px'
            }}>
              Add Pokemon to start building your team
            </div>
          )}
        </div>
      </div>

      <div style={{ gridColumn: '1 / -1' }}>
        <div className="card">
          <div style={{ textAlign: 'center' }}>
            <button
              className="btn btn-primary"
              onClick={onCreateBattle}
              disabled={sideOnePokemon.length === 0 || sideTwoPokemon.length === 0}
              style={{ 
                fontSize: '1.1rem', 
                padding: '12px 32px',
                background: sideOnePokemon.length === 0 || sideTwoPokemon.length === 0 
                  ? '#9ca3af' : undefined
              }}
            >
              ⚔️ Create Battle
            </button>
            <p style={{ marginTop: '8px', color: '#6b7280', fontSize: '0.875rem' }}>
              Both sides need at least one Pokemon to start a battle
            </p>
          </div>
        </div>
      </div>

      {/* Pokemon Editor Modal */}
      {editingPokemon && (
        <PokemonEditor
          pokemon={editingPokemon.pokemon}
          onUpdate={updatePokemon}
          onClose={() => setEditingPokemon(null)}
        />
      )}

      {/* Showdown Importer Modal */}
      {showdownImporter.show && (
        <ShowdownImporter
          side={`Side ${showdownImporter.side === 'one' ? 'One' : 'Two'}`}
          onImport={(pokemon) => handleShowdownImport(showdownImporter.side, pokemon)}
          onClose={() => setShowdownImporter({ show: false, side: 'one' })}
        />
      )}
    </div>
  )
}

export default PokemonBuilder