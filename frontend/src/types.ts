// ── Overwatch domain primitives ────────────────────────────────────────
export type OverwatchRank =
  | 'Unranked'
  | 'Bronze'
  | 'Silver'
  | 'Gold'
  | 'Platinum'
  | 'Diamond'
  | 'Master'
  | 'Grandmaster'
  | 'Champion'

export type OverwatchRole = 'Tank' | 'DPS' | 'Support' | 'Flex'

// ── Event domain ───────────────────────────────────────────────────────
export type EventType = 'PUG' | 'TOURNEY'
export type EventFormat = '5v5' | '6v6' | '1v1'

export interface RoleRank {
  role: OverwatchRole
  rank: OverwatchRank
}

export interface EventTeam {
  id: number | string
  name: string
  player_ids?: (number | string)[]
  created_at?: string
  updated_at?: string
}

export interface EventPlayer {
  id: number | string
  name: string
  role: OverwatchRole
  rank: OverwatchRank
  team_id: number | string | null
  team?: EventTeam | null
  assigned_role: OverwatchRole | null
  assigned_rank: OverwatchRank | null
  roles: RoleRank[]
}

export interface EventMatch {
  id: number | string
  title: string
  map: string
  start_date: string | null
  team_a_id: number | string | null
  team_b_id: number | string | null
  team_a_name: string | null
  team_b_name: string | null
  winner_team_id: number | string | null
  winner_team_name: string | null
  max_players?: number | null
  round?: number | null
  position?: number | null
  status?: string
  isPlaceholder?: boolean
  next_match_id?: string | number | null
  players: EventPlayer[]
}

export interface Event {
  id: number | string
  name: string
  description: string
  start_date: string | null
  event_type: EventType
  format: EventFormat
  max_players: number
  can_manage: boolean
  public_signup_enabled: boolean
  public_signup_token: string | null
  is_featured: boolean
  is_ended: boolean
  creator_id?: number | string
  creator_name?: string
  players: EventPlayer[]
  teams: EventTeam[]
  matches: EventMatch[]
}

// ── API response shapes ────────────────────────────────────────────────
export interface AutoBalanceResponse {
  event: Event
  summary: string
}

export interface SignupLink {
  signup_token: string
}

export interface SignupRequest {
  id: number | string
  status: string
  name: string
  created_at?: string
  updated_at?: string
  roles?: Array<{ role: string; rank: string }>
  [key: string]: unknown
}

// ── Auth domain ────────────────────────────────────────────────────────
export interface AuthUser {
  id: number | string
  username: string
  email: string
  role: string
  [key: string]: unknown
}

export interface AuthSession {
  user: AuthUser
  access_token: string
  refresh_token: string
}

export interface PublicSignupInfo {
  event_name: string
  public_signup_enabled: boolean
  current_players: number
  max_players: number
  current_signup_requests: number
  start_date?: string | null
  [key: string]: unknown
}
