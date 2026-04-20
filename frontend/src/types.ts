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
export type EventStatus = 'DRAFT' | 'ACTIVE' | 'ENDED'

export interface LinkedUserInfo {
  id: string
  username: string
  display_name: string
  discord_username: string | null
  battletag: string | null
  avatar_url: string | null
}

export interface ParticipatedEventSummary {
  id: string
  name: string
  start_date: string | null
  event_type: EventType
  format: EventFormat
  status: EventStatus
}

export interface RoleRank {
  role: OverwatchRole
  rank: OverwatchRank
}

export interface EventTeam {
  id: string
  name: string
  player_ids?: string[]
  created_at?: string
  updated_at?: string
}

export interface EventPlayer {
  id: string
  name: string
  role: OverwatchRole
  rank: OverwatchRank
  team_id: string | null
  team?: EventTeam | null
  assigned_role: OverwatchRole | null
  assigned_rank: OverwatchRank | null
  roles: RoleRank[]
  linked_user?: LinkedUserInfo | null
  reported_discord?: string | null
  reported_battletag?: string | null
}

export interface EventMatch {
  id: string
  title: string
  map: string
  start_date: string | null
  team_a_id: string | null
  team_b_id: string | null
  team_a_name: string | null
  team_b_name: string | null
  winner_team_id: string | null
  winner_team_name: string | null
  max_players?: number | null
  round?: number | null
  position?: number | null
  status?: string
  isPlaceholder?: boolean
  next_match_id?: string | null
  next_match_slot?: string | null
  players: EventPlayer[]
}

export interface Event {
  id: string
  name: string
  description: string
  start_date: string | null
  event_type: EventType
  format: EventFormat
  max_players: number
  can_manage: boolean
  public_signup_enabled: boolean
  public_signup_token: string | null
  require_discord: boolean
  require_battletag: boolean
  discord_announce: boolean
  is_featured: boolean
  status: EventStatus
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

export interface PublicSignupInfo {
  event_id?: string | number
  event_name: string
  public_signup_enabled: boolean
  current_players: number
  max_players: number
  current_signup_requests: number
  start_date?: string | null
  already_joined: boolean
  require_discord: boolean
  require_battletag: boolean
  [key: string]: unknown
}

export interface SignupRequest {
  id: string
  status: string
  name: string
  created_at?: string
  updated_at?: string
  roles?: Array<{ role: string; rank: string }>
  linked_user?: LinkedUserInfo | null
  reported_discord?: string | null
  reported_battletag?: string | null
  [key: string]: unknown
}

// ── Auth domain ────────────────────────────────────────────────────────
export interface AuthUser {
  id: string
  username: string
  email: string
  role: string
  display_name: string
  battletag: string | null
  rank_tank: OverwatchRank
  rank_dps: OverwatchRank
  rank_support: OverwatchRank
  can_edit_battletag: boolean
  has_password: boolean
  has_discord_identity: boolean
  discord_username: string | null
  avatar_url: string | null
}

export interface AuthSession {
  user: AuthUser
  access_token: string
  refresh_token: string
}

export interface PendingVerificationResponse {
  message: string
}


