import React, { useState, useEffect } from 'react'
import { Pokemon } from '../types'

interface PokemonEditorProps {
  pokemon: Pokemon | null
  onUpdate: (pokemon: Pokemon) => void
  onClose: () => void
}

interface Dropdown {
  isOpen: boolean
  searchTerm: string
}

const PokemonEditor: React.FC<PokemonEditorProps> = ({ pokemon, onUpdate, onClose }) => {
  const [activeTab, setActiveTab] = useState<'general' | 'stats' | 'moves'>('general')
  const [editedPokemon, setEditedPokemon] = useState<Pokemon | null>(pokemon)
  const [availableSpecies, setAvailableSpecies] = useState<string[]>([])
  const [availableMoves, setAvailableMoves] = useState<string[]>([])
  const [availableItems, setAvailableItems] = useState<string[]>([])
  const [availableAbilities, setAvailableAbilities] = useState<string[]>([])
  
  // Dropdown states
  const [speciesDropdown, setSpeciesDropdown] = useState<Dropdown>({ isOpen: false, searchTerm: '' })
  const [itemDropdown, setItemDropdown] = useState<Dropdown>({ isOpen: false, searchTerm: '' })
  const [moveDropdowns, setMoveDropdowns] = useState<Dropdown[]>([
    { isOpen: false, searchTerm: '' },
    { isOpen: false, searchTerm: '' },
    { isOpen: false, searchTerm: '' },
    { isOpen: false, searchTerm: '' }
  ])

  // IVs and EVs (default to 31 IVs, 0 EVs)
  const [ivs, setIvs] = useState<number[]>([31, 31, 31, 31, 31, 31])
  const [evs, setEvs] = useState<number[]>([0, 0, 0, 0, 0, 0])

  useEffect(() => {
    setEditedPokemon(pokemon)
    if (pokemon) {
      loadSpeciesData(pokemon.species)
      // Initialize IVs and EVs from Pokemon data or defaults
      setIvs(pokemon.ivs || [31, 31, 31, 31, 31, 31])
      setEvs(pokemon.evs || [0, 0, 0, 0, 0, 0])
    }
    
    // Load initial data
    Promise.all([
      fetch('/api/pokemon').then(r => r.json()),
      fetch('/api/moves').then(r => r.json()),
      fetch('/api/items').then(r => r.json())
    ]).then(([species, moves, items]) => {
      setAvailableSpecies(species)
      setAvailableMoves(moves)
      setAvailableItems(items)
    }).catch(console.error)
  }, [pokemon])

  const loadSpeciesData = async (species: string) => {
    try {
      const [movesResponse, abilitiesResponse] = await Promise.all([
        fetch(`/api/pokemon/${encodeURIComponent(species)}/moves`),
        fetch(`/api/pokemon/${encodeURIComponent(species)}/abilities`)
      ])
      
      const [moves, abilities] = await Promise.all([
        movesResponse.json(),
        abilitiesResponse.json()
      ])
      
      setAvailableMoves(moves)
      setAvailableAbilities(abilities)
    } catch (error) {
      console.error('Failed to load species data:', error)
    }
  }

  const handleSpeciesChange = async (species: string) => {
    try {
      const response = await fetch('/api/pokemon/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ species, level: editedPokemon?.level || 50 })
      })
      const newPokemon = await response.json()
      setEditedPokemon(newPokemon)
      loadSpeciesData(species)
      setSpeciesDropdown({ isOpen: false, searchTerm: species })
    } catch (error) {
      console.error('Failed to create Pokemon:', error)
    }
  }

  const updatePokemonField = (field: keyof Pokemon, value: any) => {
    if (!editedPokemon) return
    
    const updated = { ...editedPokemon, [field]: value }
    setEditedPokemon(updated)
    
    // If level changes, recalculate stats
    if (field === 'level') {
      recalculateStats(updated.species, value, ivs, evs, editedPokemon.nature || 'Hardy')
    }
  }

  const recalculateStats = async (species: string, level: number, newIvs: number[], newEvs: number[], nature: string) => {
    try {
      const response = await fetch('/api/pokemon/recalculate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          species,
          level,
          ivs: newIvs,
          evs: newEvs,
          nature
        })
      })
      const updatedPokemon = await response.json()
      
      if (editedPokemon) {
        setEditedPokemon({
          ...editedPokemon,
          stats: updatedPokemon.stats,
          hp: updatedPokemon.hp,
          max_hp: updatedPokemon.max_hp,
          level,
          nature,
          ivs: newIvs,
          evs: newEvs
        })
      }
    } catch (error) {
      console.error('Failed to recalculate stats:', error)
    }
  }

  const updateMove = (index: number, moveData: any) => {
    if (!editedPokemon) return
    
    const updatedMoves = [...editedPokemon.moves]
    
    // Ensure we have enough move slots
    while (updatedMoves.length <= index) {
      updatedMoves.push({
        name: '',
        move_type: 'Normal',
        category: 'Physical',
        base_power: 0,
        accuracy: 100,
        pp: 5,
        max_pp: 5,
        priority: 0,
        target: 'Normal'
      })
    }
    
    updatedMoves[index] = moveData
    updatePokemonField('moves', updatedMoves)
  }

  const handleSave = () => {
    if (editedPokemon) {
      onUpdate(editedPokemon)
      onClose()
    }
  }

  const calculateEvRemaining = () => {
    const total = evs.reduce((sum, ev) => sum + ev, 0)
    return Math.max(0, 508 - total)
  }

  const filteredSpecies = availableSpecies.filter(s => 
    s.toLowerCase().includes(speciesDropdown.searchTerm.toLowerCase())
  ).slice(0, 20)

  const filteredItems = availableItems.filter(i => 
    i.toLowerCase().includes(itemDropdown.searchTerm.toLowerCase())
  ).slice(0, 20)

  const getFilteredMovesForSlot = (slotIndex: number) => {
    return availableMoves.filter(m => 
      m.toLowerCase().includes(moveDropdowns[slotIndex].searchTerm.toLowerCase())
    ).slice(0, 20)
  }

  if (!editedPokemon) {
    return (
      <div style={{ textAlign: 'center', padding: '40px', color: '#666' }}>
        Select a Pokemon to edit
      </div>
    )
  }

  return (
    <div style={{ 
      position: 'fixed', top: '0', left: '0', right: '0', bottom: '0',
      background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center',
      zIndex: 1000
    }}>
      <div style={{
        background: 'white', borderRadius: '12px', padding: '24px',
        maxWidth: '800px', width: '90%', maxHeight: '90%', overflow: 'auto'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
          <h2 style={{ margin: 0 }}>Edit {editedPokemon.species}</h2>
          <button 
            className="btn btn-secondary" 
            onClick={onClose}
            style={{ padding: '8px 12px' }}
          >
            ✕
          </button>
        </div>

        {/* Tabs */}
        <div style={{ display: 'flex', borderBottom: '2px solid #e0e0e0', marginBottom: '20px' }}>
          {[
            { id: 'general', label: 'General' },
            { id: 'stats', label: 'Stats & EVs' },
            { id: 'moves', label: 'Moves' }
          ].map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              style={{
                padding: '10px 20px',
                border: 'none',
                background: 'none',
                borderBottom: activeTab === tab.id ? '2px solid #4a90e2' : '2px solid transparent',
                color: activeTab === tab.id ? '#4a90e2' : '#666',
                fontWeight: activeTab === tab.id ? '600' : 'normal',
                cursor: 'pointer'
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* General Tab */}
        {activeTab === 'general' && (
          <div>
            <div className="form-group">
              <label className="form-label">Pokemon Species</label>
              <div style={{ position: 'relative' }}>
                <input
                  type="text"
                  className="form-control"
                  value={speciesDropdown.searchTerm || editedPokemon.species}
                  onChange={(e) => setSpeciesDropdown({ ...speciesDropdown, searchTerm: e.target.value, isOpen: true })}
                  onFocus={() => setSpeciesDropdown({ ...speciesDropdown, isOpen: true })}
                  placeholder="Search Pokemon..."
                />
                {speciesDropdown.isOpen && filteredSpecies.length > 0 && (
                  <div style={{
                    position: 'absolute', top: '100%', left: 0, right: 0,
                    background: 'white', border: '1px solid #ddd', borderRadius: '6px',
                    maxHeight: '200px', overflowY: 'auto', zIndex: 1000,
                    boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
                  }}>
                    {filteredSpecies.map(species => (
                      <div
                        key={species}
                        onClick={() => handleSpeciesChange(species)}
                        style={{
                          padding: '10px', cursor: 'pointer',
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

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
              <div className="form-group">
                <label className="form-label">Level</label>
                <input
                  type="number"
                  className="form-control"
                  value={editedPokemon.level}
                  onChange={(e) => updatePokemonField('level', parseInt(e.target.value))}
                  min="1"
                  max="100"
                />
              </div>

              <div className="form-group">
                <label className="form-label">Gender</label>
                <select
                  className="form-control"
                  value={editedPokemon.gender}
                  onChange={(e) => updatePokemonField('gender', e.target.value)}
                >
                  <option value="Unknown">Unknown</option>
                  <option value="Male">Male</option>
                  <option value="Female">Female</option>
                </select>
              </div>
            </div>

            <div className="form-group">
              <label className="form-label">Item</label>
              <div style={{ position: 'relative' }}>
                <input
                  type="text"
                  className="form-control"
                  value={itemDropdown.searchTerm || editedPokemon.item || ''}
                  onChange={(e) => setItemDropdown({ ...itemDropdown, searchTerm: e.target.value, isOpen: true })}
                  onFocus={() => setItemDropdown({ ...itemDropdown, isOpen: true })}
                  placeholder="Search items..."
                />
                {itemDropdown.isOpen && filteredItems.length > 0 && (
                  <div style={{
                    position: 'absolute', top: '100%', left: 0, right: 0,
                    background: 'white', border: '1px solid #ddd', borderRadius: '6px',
                    maxHeight: '200px', overflowY: 'auto', zIndex: 1000,
                    boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
                  }}>
                    {filteredItems.map(item => (
                      <div
                        key={item}
                        onClick={() => {
                          updatePokemonField('item', item)
                          setItemDropdown({ isOpen: false, searchTerm: item })
                        }}
                        style={{
                          padding: '10px', cursor: 'pointer',
                          borderBottom: '1px solid #eee'
                        }}
                        onMouseEnter={(e) => e.currentTarget.style.background = '#f1f1f1'}
                        onMouseLeave={(e) => e.currentTarget.style.background = 'white'}
                      >
                        {item}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>

            <div className="form-group">
              <label className="form-label">Ability</label>
              <select
                className="form-control"
                value={editedPokemon.ability}
                onChange={(e) => updatePokemonField('ability', e.target.value)}
              >
                <option>Select ability...</option>
                {availableAbilities.map(ability => (
                  <option key={ability} value={ability}>{ability}</option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label className="form-label">Nature</label>
              <select
                className="form-control"
                value={editedPokemon.nature || 'Hardy'}
                onChange={(e) => {
                  updatePokemonField('nature', e.target.value)
                  recalculateStats(editedPokemon.species, editedPokemon.level, ivs, evs, e.target.value)
                }}
              >
                <option value="Hardy">Hardy</option>
                <option value="Lonely">Lonely (+Atk -Def)</option>
                <option value="Brave">Brave (+Atk -Spe)</option>
                <option value="Adamant">Adamant (+Atk -SpA)</option>
                <option value="Naughty">Naughty (+Atk -SpD)</option>
                <option value="Bold">Bold (+Def -Atk)</option>
                <option value="Docile">Docile</option>
                <option value="Relaxed">Relaxed (+Def -Spe)</option>
                <option value="Impish">Impish (+Def -SpA)</option>
                <option value="Lax">Lax (+Def -SpD)</option>
                <option value="Timid">Timid (+Spe -Atk)</option>
                <option value="Hasty">Hasty (+Spe -Def)</option>
                <option value="Serious">Serious</option>
                <option value="Jolly">Jolly (+Spe -SpA)</option>
                <option value="Naive">Naive (+Spe -SpD)</option>
                <option value="Modest">Modest (+SpA -Atk)</option>
                <option value="Mild">Mild (+SpA -Def)</option>
                <option value="Quiet">Quiet (+SpA -Spe)</option>
                <option value="Bashful">Bashful</option>
                <option value="Rash">Rash (+SpA -SpD)</option>
                <option value="Calm">Calm (+SpD -Atk)</option>
                <option value="Gentle">Gentle (+SpD -Def)</option>
                <option value="Sassy">Sassy (+SpD -Spe)</option>
                <option value="Careful">Careful (+SpD -SpA)</option>
                <option value="Quirky">Quirky</option>
              </select>
            </div>
          </div>
        )}

        {/* Stats Tab */}
        {activeTab === 'stats' && (
          <div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '20px' }}>
              {/* Base Stats */}
              <div style={{ border: '1px solid #e0e0e0', borderRadius: '6px', padding: '12px' }}>
                <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: '#666' }}>BASE STATS</h4>
                {['HP', 'ATK', 'DEF', 'SPA', 'SPD', 'SPE'].map((stat, index) => {
                  const statValue = Object.values(editedPokemon.stats)[index]
                  return (
                    <div key={stat} style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
                      <span style={{ width: '40px', fontSize: '12px', fontWeight: '600' }}>{stat}</span>
                      <div style={{ 
                        flex: 1, height: '8px', background: '#e0e0e0', 
                        borderRadius: '4px', margin: '0 8px' 
                      }}>
                        <div style={{
                          width: `${(statValue / 255) * 100}%`,
                          height: '100%',
                          background: ['#50c878', '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#feca57'][index],
                          borderRadius: '4px'
                        }} />
                      </div>
                      <span style={{ width: '30px', fontSize: '12px', textAlign: 'center' }}>{statValue}</span>
                    </div>
                  )
                })}
              </div>

              {/* EVs */}
              <div style={{ border: '1px solid #e0e0e0', borderRadius: '6px', padding: '12px' }}>
                <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: '#666' }}>EVS (EFFORT VALUES)</h4>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                  {['HP', 'ATK', 'DEF', 'SPA', 'SPD', 'SPE'].map((stat, index) => (
                    <input
                      key={stat}
                      type="number"
                      placeholder={stat}
                      value={evs[index]}
                      onChange={(e) => {
                        const newEvs = [...evs]
                        newEvs[index] = Math.min(252, Math.max(0, parseInt(e.target.value) || 0))
                        setEvs(newEvs)
                        
                        // Recalculate stats when EVs change
                        if (editedPokemon) {
                          recalculateStats(editedPokemon.species, editedPokemon.level, ivs, newEvs, editedPokemon.nature || 'Hardy')
                        }
                      }}
                      min="0"
                      max="252"
                      style={{
                        textAlign: 'center', padding: '6px', border: '1px solid #ddd',
                        borderRadius: '4px', fontSize: '12px'
                      }}
                    />
                  ))}
                </div>
                <p style={{ fontSize: '12px', color: '#666', marginTop: '8px', margin: '8px 0 0 0' }}>
                  Remaining: {calculateEvRemaining()}/508
                </p>
              </div>

              {/* IVs */}
              <div style={{ border: '1px solid #e0e0e0', borderRadius: '6px', padding: '12px' }}>
                <h4 style={{ margin: '0 0 12px 0', fontSize: '14px', color: '#666' }}>IVS (INDIVIDUAL VALUES)</h4>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                  {['HP', 'ATK', 'DEF', 'SPA', 'SPD', 'SPE'].map((stat, index) => (
                    <input
                      key={stat}
                      type="number"
                      placeholder={stat}
                      value={ivs[index]}
                      onChange={(e) => {
                        const newIvs = [...ivs]
                        newIvs[index] = Math.min(31, Math.max(0, parseInt(e.target.value) || 0))
                        setIvs(newIvs)
                        
                        // Recalculate stats when IVs change
                        if (editedPokemon) {
                          recalculateStats(editedPokemon.species, editedPokemon.level, newIvs, evs, editedPokemon.nature || 'Hardy')
                        }
                      }}
                      min="0"
                      max="31"
                      style={{
                        textAlign: 'center', padding: '6px', border: '1px solid #ddd',
                        borderRadius: '4px', fontSize: '12px'
                      }}
                    />
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Moves Tab */}
        {activeTab === 'moves' && (
          <div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '16px' }}>
              {[0, 1, 2, 3].map(moveIndex => (
                <div key={moveIndex} className="form-group">
                  <label className="form-label">Move {moveIndex + 1}</label>
                  <div style={{ position: 'relative' }}>
                    <input
                      type="text"
                      className="form-control"
                      value={moveDropdowns[moveIndex].searchTerm || editedPokemon.moves[moveIndex]?.name || ''}
                      onChange={(e) => {
                        const newDropdowns = [...moveDropdowns]
                        newDropdowns[moveIndex] = { searchTerm: e.target.value, isOpen: true }
                        setMoveDropdowns(newDropdowns)
                      }}
                      onFocus={() => {
                        const newDropdowns = [...moveDropdowns]
                        newDropdowns[moveIndex] = { ...newDropdowns[moveIndex], isOpen: true }
                        setMoveDropdowns(newDropdowns)
                      }}
                      placeholder="Search moves..."
                    />
                    {moveDropdowns[moveIndex].isOpen && (
                      <div style={{
                        position: 'absolute', top: '100%', left: 0, right: 0,
                        background: 'white', border: '1px solid #ddd', borderRadius: '6px',
                        maxHeight: '200px', overflowY: 'auto', zIndex: 1000,
                        boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
                      }}>
                        {getFilteredMovesForSlot(moveIndex).map(move => (
                          <div
                            key={move}
                            onClick={async () => {
                              try {
                                const response = await fetch(`/api/moves/${encodeURIComponent(move)}`)
                                const moveData = await response.json()
                                updateMove(moveIndex, moveData)
                                
                                const newDropdowns = [...moveDropdowns]
                                newDropdowns[moveIndex] = { searchTerm: move, isOpen: false }
                                setMoveDropdowns(newDropdowns)
                              } catch (error) {
                                console.error('Failed to load move data:', error)
                              }
                            }}
                            style={{
                              padding: '10px', cursor: 'pointer',
                              borderBottom: '1px solid #eee'
                            }}
                            onMouseEnter={(e) => e.currentTarget.style.background = '#f1f1f1'}
                            onMouseLeave={(e) => e.currentTarget.style.background = 'white'}
                          >
                            {move}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Save/Cancel buttons */}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px', marginTop: '24px' }}>
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            Save Changes
          </button>
        </div>
      </div>
    </div>
  )
}

export default PokemonEditor