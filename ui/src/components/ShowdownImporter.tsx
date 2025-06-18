import React, { useState } from 'react'
import { Pokemon } from '../types'
import { parseShowdownTeam, convertShowdownToTapuPokemon } from '../utils/showdownParser'

interface ShowdownImporterProps {
  onImport: (pokemon: Pokemon[]) => void
  onClose: () => void
  side: string
}

const ShowdownImporter: React.FC<ShowdownImporterProps> = ({ onImport, onClose, side }) => {
  const [pasteText, setPasteText] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [preview, setPreview] = useState<string[]>([])

  const handleTextChange = (text: string) => {
    setPasteText(text)
    setError(null)
    
    // Generate preview
    try {
      if (text.trim()) {
        const parsed = parseShowdownTeam(text)
        setPreview(parsed.map(p => `${p.species} ${p.ability ? `(${p.ability})` : ''}`))
      } else {
        setPreview([])
      }
    } catch (err) {
      setPreview([])
    }
  }

  const handleImport = async () => {
    if (!pasteText.trim()) {
      setError('Please paste a team first')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const parsed = parseShowdownTeam(pasteText)
      
      if (parsed.length === 0) {
        setError('No valid Pokemon found in the paste')
        return
      }

      const converted = await convertShowdownToTapuPokemon(parsed)
      
      if (converted.length === 0) {
        setError('Failed to convert any Pokemon. Check the console for detailed error messages.')
        return
      }

      if (converted.length < parsed.length) {
        console.warn(`Only ${converted.length} out of ${parsed.length} Pokemon were successfully imported`)
      }

      onImport(converted)
      onClose()
    } catch (err) {
      console.error('Import error:', err)
      setError('Failed to import team. Please check the format and try again.')
    } finally {
      setIsLoading(false)
    }
  }

  const samplePaste = `Dragonite @ Multiscale
Ability: Multiscale
Level: 50
EVs: 252 Atk / 4 HP / 252 Spe
Adamant Nature
- Dragon Dance
- Outrage
- Earthquake
- Extreme Speed

Toxapex @ Black Sludge
Ability: Regenerator
Level: 50
EVs: 252 HP / 252 Def / 4 SpD
Bold Nature
IVs: 0 Atk
- Scald
- Recover
- Haze
- Toxic Spikes

Garchomp @ Rocky Helmet
Ability: Rough Skin
Level: 50
EVs: 252 Atk / 4 HP / 252 Spe
Jolly Nature
- Earthquake
- Dragon Claw
- Stone Edge
- Stealth Rock

Rotom-Wash @ Leftovers
Ability: Levitate
Level: 50
EVs: 252 HP / 252 Def / 4 SpA
Bold Nature
IVs: 0 Atk
- Hydro Pump
- Volt Switch
- Will-O-Wisp
- Defog

Clefable @ Leftovers
Ability: Magic Guard
Level: 50
EVs: 252 HP / 252 Def / 4 SpA
Bold Nature
IVs: 0 Atk
- Moonblast
- Soft-Boiled
- Stealth Rock
- Thunder Wave

Heatran @ Air Balloon
Ability: Flash Fire
Level: 50
EVs: 4 HP / 252 SpA / 252 Spe
Modest Nature
IVs: 0 Atk
- Magma Storm
- Earth Power
- Flash Cannon
- Taunt`

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'rgba(0,0,0,0.8)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      padding: '20px'
    }}>
      <div style={{
        background: 'white',
        borderRadius: '12px',
        padding: '24px',
        maxWidth: '800px',
        width: '100%',
        maxHeight: '80vh',
        overflow: 'auto',
        boxShadow: '0 25px 50px -12px rgba(0,0,0,0.25)'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
          <h2 style={{ margin: 0, fontSize: '1.5rem', fontWeight: 'bold' }}>
            Import Pokemon Showdown Team - {side}
          </h2>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              fontSize: '24px',
              cursor: 'pointer',
              padding: '4px',
              color: '#6b7280'
            }}
          >
            ×
          </button>
        </div>

        <div style={{ marginBottom: '16px' }}>
          <p style={{ color: '#6b7280', marginBottom: '8px' }}>
            Paste your Pokemon Showdown team export below. You can get this by going to the teambuilder 
            on Pokemon Showdown and clicking "Import/Export" → "Copy to clipboard".
          </p>
          <div style={{
            background: '#dcfce7',
            border: '1px solid #16a34a',
            borderRadius: '6px',
            padding: '12px',
            fontSize: '14px',
            color: '#166534'
          }}>
            <strong>✓ Full Import Support:</strong> Imports species, level, IVs, EVs, nature, moves, abilities, items, and Tera types from your Pokemon Showdown teams.
          </div>
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label className="form-label">Team Paste</label>
          <textarea
            value={pasteText}
            onChange={(e) => handleTextChange(e.target.value)}
            placeholder={`Paste your Pokemon Showdown team here...\n\nExample format:\n${samplePaste}`}
            style={{
              width: '100%',
              height: '200px',
              padding: '12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              fontFamily: 'monospace',
              fontSize: '14px',
              resize: 'vertical'
            }}
          />
        </div>

        {preview.length > 0 && (
          <div style={{ marginBottom: '16px' }}>
            <label className="form-label">Import Preview ({preview.length} Pokemon)</label>
            <div style={{
              background: '#f9fafb',
              border: '1px solid #e5e7eb',
              borderRadius: '6px',
              padding: '12px',
              fontSize: '14px'
            }}>
              {preview.map((pokemon, index) => (
                <div key={index} style={{ padding: '4px 0', borderBottom: index < preview.length - 1 ? '1px solid #e5e7eb' : 'none' }}>
                  <div style={{ fontWeight: 'bold' }}>{index + 1}. {pokemon}</div>
                  <div style={{ fontSize: '12px', color: '#16a34a', marginLeft: '12px' }}>
                    ✓ Full customization will be imported
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {error && (
          <div style={{
            background: '#fee2e2',
            border: '1px solid #fecaca',
            borderRadius: '6px',
            padding: '12px',
            marginBottom: '16px',
            color: '#dc2626'
          }}>
            {error}
          </div>
        )}

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            className="btn btn-secondary"
            disabled={isLoading}
          >
            Cancel
          </button>
          <button
            onClick={handleImport}
            className="btn btn-primary"
            disabled={isLoading || !pasteText.trim()}
          >
            {isLoading ? 'Importing...' : `Import ${preview.length} Pokemon`}
          </button>
        </div>

        <details style={{ marginTop: '16px' }}>
          <summary style={{ cursor: 'pointer', color: '#6b7280', fontSize: '14px' }}>
            View sample team format
          </summary>
          <pre style={{
            background: '#f9fafb',
            border: '1px solid #e5e7eb',
            borderRadius: '6px',
            padding: '12px',
            marginTop: '8px',
            fontSize: '12px',
            overflow: 'auto'
          }}>
            {samplePaste}
          </pre>
        </details>
      </div>
    </div>
  )
}

export default ShowdownImporter