import React from 'react'
import { Pokemon } from '../types'

interface PokemonCardProps {
  pokemon: Pokemon
  onRemove?: () => void
  showMoves?: boolean
}

const PokemonCard: React.FC<PokemonCardProps> = ({ 
  pokemon, 
  onRemove, 
  showMoves = false 
}) => {
  const getTypeClass = (type: string) => {
    return `type-${type.toLowerCase()}`
  }

  const getHPBarColor = (hp: number, maxHP: number) => {
    const percentage = (hp / maxHP) * 100
    if (percentage > 50) return '#10b981'
    if (percentage > 25) return '#f59e0b'
    return '#ef4444'
  }

  return (
    <div className="pokemon-card">
      <div className="pokemon-header">
        <div>
          <div className="pokemon-name">{pokemon.species}</div>
          <div className="pokemon-level">Level {pokemon.level}</div>
        </div>
        {onRemove && (
          <button
            className="btn btn-secondary"
            onClick={onRemove}
            style={{ padding: '4px 8px', fontSize: '0.75rem' }}
          >
            ✕
          </button>
        )}
      </div>

      <div style={{ marginBottom: '12px' }}>
        <div style={{ 
          display: 'flex', 
          justifyContent: 'space-between', 
          alignItems: 'center',
          marginBottom: '4px' 
        }}>
          <span style={{ fontSize: '0.875rem', fontWeight: '500' }}>HP</span>
          <span style={{ fontSize: '0.875rem' }}>{pokemon.hp}/{pokemon.max_hp}</span>
        </div>
        <div style={{
          width: '100%',
          height: '8px',
          backgroundColor: '#e5e7eb',
          borderRadius: '4px',
          overflow: 'hidden'
        }}>
          <div
            style={{
              width: `${(pokemon.hp / pokemon.max_hp) * 100}%`,
              height: '100%',
              backgroundColor: getHPBarColor(pokemon.hp, pokemon.max_hp),
              transition: 'all 0.3s ease'
            }}
          />
        </div>
      </div>

      <div className="pokemon-stats">
        <div className="stat-item">
          <span className="stat-name">ATK</span>
          <span className="stat-value">{pokemon.stats.attack}</span>
        </div>
        <div className="stat-item">
          <span className="stat-name">DEF</span>
          <span className="stat-value">{pokemon.stats.defense}</span>
        </div>
        <div className="stat-item">
          <span className="stat-name">SPA</span>
          <span className="stat-value">{pokemon.stats.special_attack}</span>
        </div>
        <div className="stat-item">
          <span className="stat-name">SPD</span>
          <span className="stat-value">{pokemon.stats.special_defense}</span>
        </div>
        <div className="stat-item">
          <span className="stat-name">SPE</span>
          <span className="stat-value">{pokemon.stats.speed}</span>
        </div>
        <div className="stat-item">
          <span className="stat-name">Ability</span>
          <span className="stat-value" style={{ fontSize: '0.8rem' }}>{pokemon.ability}</span>
        </div>
      </div>

      <div style={{ marginBottom: '12px' }}>
        <div className="types">
          {pokemon.types.map((type, index) => (
            <span key={index} className={`type-badge ${getTypeClass(type)}`}>
              {type}
            </span>
          ))}
        </div>
      </div>

      {showMoves && pokemon.moves.length > 0 && (
        <div>
          <div style={{ 
            fontSize: '0.875rem', 
            fontWeight: '500', 
            marginBottom: '8px',
            color: '#374151'
          }}>
            Moves
          </div>
          <div style={{ display: 'grid', gap: '4px' }}>
            {pokemon.moves.slice(0, 4).map((move, index) => (
              <div 
                key={index}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '4px 8px',
                  backgroundColor: '#f9fafb',
                  borderRadius: '4px',
                  fontSize: '0.8rem'
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: '500' }}>{move.name}</span>
                  <span className={`type-badge ${getTypeClass(move.move_type)}`}>
                    {move.move_type}
                  </span>
                </div>
                <div style={{ display: 'flex', gap: '8px', fontSize: '0.75rem', color: '#6b7280' }}>
                  {move.base_power > 0 && <span>{move.base_power}</span>}
                  <span>{move.pp}/{move.max_pp}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {pokemon.item && (
        <div style={{ 
          marginTop: '8px', 
          fontSize: '0.8rem', 
          color: '#6b7280' 
        }}>
          Item: {pokemon.item}
        </div>
      )}
    </div>
  )
}

export default PokemonCard