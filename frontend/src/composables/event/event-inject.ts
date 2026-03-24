/**
 * Type for the event context provided by EventPage.vue via proxyRefs and
 * consumed by child section components via inject('eventCtx').
 *
 * proxyRefs unwraps Ref<T> → T at the type level, so all properties here
 * are their direct value types, not Ref-wrapped.
 */
import type { Event, EventPlayer, EventTeam, OverwatchRank, RoleRank, SignupRequest } from '../../types'

export interface EventCtxType {
  // ── Core event state ──
  event: Event | null
  eventIsFull: boolean
  loadingEvent: boolean

  // ── Busy flags ──
  creatingTeam: boolean
  creatingSoloTeams: boolean
  balancingTeams: boolean
  creatingMatch: boolean
  clearingBracket: boolean
  deletingEvent: boolean
  deletingMatchId: string | null
  addingPlayer: boolean
  deletingPlayers: Record<string, boolean>
  deletingTeams: Record<string, boolean>
  savingPlayerTeams: Record<string, boolean>
  savingPlayerEdits: Record<string, boolean>
  savingTeamEdits: Record<string, boolean>
  savingMatchups: Record<string, boolean>
  reportingWinners: Record<string, boolean>
  cancellingWinners: Record<string, boolean>
  updatingEvent: boolean
  updatingSignupVisibility: boolean
  rotatingSignupLink: boolean
  loadingSignupRequests: boolean
  endingEvent: boolean

  // ── Derived flags ──
  isTourneyEvent: boolean
  canCreateTeam: boolean
  canCreateMatch: boolean
  canAddPlayer: boolean
  canManageEvent: boolean
  teamsAreAlreadyBalanced: boolean
  canSaveEventMeta: boolean
  lastBalanceSummary: string

  // ── New-item form fields ──
  newTeamName: string
  newMatchTitle: string
  newMatchMap: string
  newMatchTeamAId: string
  newMatchTeamBId: string
  newMatchStartDate: string
  newPlayerName: string
  newPlayerRole: string
  newPlayerRank: string
  newPlayerRoles: RoleRank[]

  // ── Edit-team form fields ──
  editTeamName: string
  editingTeamId: string | null

  // ── Edit-player form fields ──
  editPlayerName: string
  editPlayerRole: string
  editPlayerRank: string
  editPlayerRoles: RoleRank[]
  editingPlayerId: string | null

  // ── Edit-event form fields ──
  editEventName: string
  editEventDescription: string
  editEventStartDate: string
  editEventFormat: string
  editEventMaxPlayers: number

  // ── Matchup selections ──
  matchupSelections: Record<string, { teamAId: string; teamBId: string }>

  // ── Signup data ──
  signupRequests: SignupRequest[]
  reviewingSignupRequests: Record<string, boolean>
  signupShareUrl: string
  signupToken: string

  // ── Utility data ──
  getRankIcon: (rank: string) => string
  overwatchRanks: OverwatchRank[]

  // ── Navigation ──
  openSection: (section: string) => void

  // ── Team actions ──
  createTeam: () => Promise<void>
  autoCreateSoloTeams: () => Promise<void>
  autoBalanceTeams: () => Promise<void>
  saveTeamEdit: (teamId: string) => Promise<void>
  deleteTeam: (team: EventTeam) => Promise<void>

  // ── Player actions ──
  addPlayer: () => Promise<void>
  savePlayerEdit: (playerId: string) => Promise<void>
  assignPlayerToTeam: (playerId: string, teamId: string | null) => Promise<void>
  assignPlayerToTeamWithRole: (playerId: string, teamId: string | null, role: string, rank: string) => Promise<void>
  removePlayerFromTeam: (playerId: string) => Promise<void>
  removePlayer: (player: EventPlayer) => Promise<void>

  // ── Match actions ──
  createMatch: () => Promise<void>
  updateMatchStartDate: (matchId: string, startDate: string) => Promise<void>
  generateTourneyBracket: (mode?: string) => Promise<void>
  clearTourneyBracket: () => Promise<void>
  deleteMatch: (matchId: string) => Promise<void>
  saveMatchup: (matchId: string) => Promise<boolean>
  reportMatchWinner: (matchId: string, winnerTeamId: string) => Promise<void>
  cancelMatchWinner: (matchId: string) => Promise<void>

  // ── Event settings actions ──
  syncEventEditDraftFromEvent: () => void
  saveEventEdit: () => Promise<void>
  deleteEvent: () => Promise<void>

  // ── Signup actions ──
  copySignupLink: () => Promise<void>
  rotateSignupLink: () => Promise<void>
  setSignupVisibility: (enabled: boolean) => Promise<void>
  setEventEnded: (ended: boolean) => Promise<void>
  acceptSignupRequest: (requestId: string) => Promise<void>
  declineSignupRequest: (requestId: string) => Promise<void>
}

