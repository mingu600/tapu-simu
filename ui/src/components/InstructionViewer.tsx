import React, { useState } from 'react'
import { StateInstructions } from '../types'

interface InstructionViewerProps {
  instructions: StateInstructions[]
}

const InstructionViewer: React.FC<InstructionViewerProps> = ({ instructions }) => {
  const [selectedSet, setSelectedSet] = useState(0)

  const getInstructionIcon = (type: string) => {
    switch (type) {
      case 'PositionDamage': return '💥'
      case 'PositionHeal': return '💚'
      case 'ApplyStatus': return '🌡️'
      case 'BoostStats': return '📈'
      case 'ChangeWeather': return '🌦️'
      case 'ChangeTerrain': return '🗺️'
      default: return '⚡'
    }
  }

  const getInstructionColor = (type: string) => {
    switch (type) {
      case 'PositionDamage': return '#ef4444'
      case 'PositionHeal': return '#10b981'
      case 'ApplyStatus': return '#f59e0b'
      case 'BoostStats': return '#3b82f6'
      case 'ChangeWeather': return '#8b5cf6'
      case 'ChangeTerrain': return '#06b6d4'
      default: return '#6b7280'
    }
  }

  const formatPosition = (position: { side: string; slot: number } | null | undefined) => {
    if (!position) return 'N/A'
    return `${position.side === 'one' ? 'Side One' : 'Side Two'} Slot ${position.slot}`
  }

  return (
    <div>
      {instructions.length > 1 && (
        <div style={{ marginBottom: '20px' }}>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {instructions.map((instructionSet, index) => (
              <button
                key={index}
                className={`btn ${selectedSet === index ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setSelectedSet(index)}
                style={{ fontSize: '0.875rem' }}
              >
                Set {index + 1} ({instructionSet.percentage.toFixed(1)}%)
              </button>
            ))}
          </div>
        </div>
      )}

      {instructions.length > 0 && (
        <div>
          <div style={{ 
            marginBottom: '16px', 
            padding: '12px', 
            backgroundColor: '#f3f4f6', 
            borderRadius: '6px' 
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontWeight: '500' }}>
                Instruction Set {selectedSet + 1}
              </span>
              <span style={{ 
                color: '#6b7280', 
                fontSize: '0.875rem' 
              }}>
                Probability: {instructions[selectedSet].percentage.toFixed(1)}%
              </span>
            </div>
            <div style={{ fontSize: '0.875rem', color: '#6b7280', marginTop: '4px' }}>
              {instructions[selectedSet].instructions.length} instruction(s), 
              affects {instructions[selectedSet].affected_positions.length} position(s)
            </div>
          </div>

          <div style={{ display: 'grid', gap: '12px' }}>
            {instructions[selectedSet].instructions.map((instruction, index) => (
              <div
                key={index}
                style={{
                  padding: '16px',
                  border: '1px solid #e5e7eb',
                  borderRadius: '8px',
                  borderLeft: `4px solid ${getInstructionColor(instruction.instruction_type)}`,
                }}
              >
                <div style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: '8px',
                  marginBottom: '8px' 
                }}>
                  <span style={{ fontSize: '1.2rem' }}>
                    {getInstructionIcon(instruction.instruction_type)}
                  </span>
                  <span style={{ 
                    fontWeight: '500', 
                    color: getInstructionColor(instruction.instruction_type) 
                  }}>
                    {instruction.instruction_type}
                  </span>
                </div>

                <div style={{ 
                  fontSize: '0.95rem', 
                  color: '#374151',
                  marginBottom: '8px' 
                }}>
                  {instruction.description}
                </div>

                <div style={{ 
                  display: 'grid', 
                  gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
                  gap: '12px',
                  fontSize: '0.875rem',
                  color: '#6b7280'
                }}>
                  {instruction.target_position && (
                    <div>
                      <strong>Target:</strong> {formatPosition(instruction.target_position)}
                    </div>
                  )}
                  
                  {instruction.affected_positions.length > 0 && (
                    <div>
                      <strong>Affected:</strong> {instruction.affected_positions.map(formatPosition).join(', ')}
                    </div>
                  )}

                  {Object.keys(instruction.details).length > 0 && (
                    <div>
                      <strong>Details:</strong>
                      <div style={{ marginTop: '4px' }}>
                        {Object.entries(instruction.details).map(([key, value]) => (
                          <div key={key} style={{ fontSize: '0.8rem' }}>
                            {key}: {JSON.stringify(value)}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>

          {instructions[selectedSet].instructions.length === 0 && (
            <div style={{
              textAlign: 'center',
              padding: '40px',
              color: '#6b7280',
              backgroundColor: '#f9fafb',
              borderRadius: '8px',
            }}>
              No instructions generated for this set
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default InstructionViewer