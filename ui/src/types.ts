export interface Pokemon {
  species: string
  level: number
  hp: number
  max_hp: number
  stats: {
    hp: number
    attack: number
    defense: number
    special_attack: number
    special_defense: number
    speed: number
  }
  moves: Move[]
  ability: string
  item: string | null
  types: string[]
  gender: string
  nature?: string
  ivs?: number[]
  evs?: number[]
  tera_type?: string | null
  is_terastallized?: boolean
}

export interface Move {
  name: string
  move_type: string
  category: string
  base_power: number
  accuracy: number
  pp: number
  max_pp: number
  priority: number
  target: string
}

export interface BattleSide {
  pokemon: Pokemon[]
  active_pokemon_indices: (number | null)[]
  side_conditions: Record<string, number>
}

export interface BattleFormat {
  name: string
  format_type: string
  generation: string
  active_pokemon_count: number
}

export interface BattleState {
  format: BattleFormat
  side_one: BattleSide
  side_two: BattleSide
  weather: string
  weather_turns_remaining: number | null
  terrain: string
  terrain_turns_remaining: number | null
  turn: number
  trick_room_active: boolean
  trick_room_turns_remaining: number | null
}

export interface BattlePosition {
  side: string
  slot: number
}

export interface MoveChoice {
  choice_type: string
  move_index?: number
  target_positions: BattlePosition[]
  pokemon_index?: number
  tera_type?: string | null
  is_forced?: boolean
}

export interface Instruction {
  instruction_type: string
  description: string
  target_position?: BattlePosition | null
  affected_positions: BattlePosition[]
  details: Record<string, any>
}

export interface StateInstructions {
  percentage: number
  instructions: Instruction[]
  affected_positions: BattlePosition[]
}

export interface InstructionResponse {
  success: boolean
  error?: string | null
  instructions: StateInstructions[]
  updated_state?: BattleState | null
}

export interface LegalOption {
  choice_type: string
  display_name: string
  move_choice: MoveChoice
  is_disabled?: boolean
  disabled_reason?: string
}

export interface TurnHistory {
  turn_number: number
  initial_state: BattleState
  side_one_choice: MoveChoice
  side_two_choice: MoveChoice
  instructions: StateInstructions[]
  final_state: BattleState
}

export interface BattleSession {
  session_id: string
  current_turn: number
  turn_history: TurnHistory[]
  current_state: BattleState
  preview_state?: BattleState
  selected_instruction_set?: number
}