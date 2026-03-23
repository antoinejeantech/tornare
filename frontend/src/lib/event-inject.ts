/**
 * Type for the event context provided by EventPage.vue via proxyRefs and
 * consumed by child section components via inject('eventCtx').
 *
 * proxyRefs unwraps Ref<T> → T at the type level, so all properties here
 * are their direct value types, not Ref-wrapped.
 */
import type { InjectionKey } from 'vue'
import type { Event, EventPlayer, EventTeam, OverwatchRank, RoleRank, SignupRequest } from '../types'

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
  deletingMatchId: string | number | null
  addingPlayer: boolean
  deletingPlayers: Record<string | number, boolean>
  deletingTeams: Record<string | number, boolean>
  savingPlayerTeams: Record<string | number, boolean>
  savingPlayerEdits: Record<string | number, boolean>
  savingTeamEdits: Record<string | number, boolean>
  savingMatchups: Record<string | number, boolean>
  reportingWinners: Record<string | number, boolean>
  cancellingWinners: Record<string | number, boolean>
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
  editingTeamId: string | number | null

  // ── Edit-player form fields ──
  editPlayerName: string
  editPlayerRole: string
  editPlayerRank: string
  editPlayerRoles: RoleRank[]
  editingPlayerId: string | number | null

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
  saveTeamEdit: (teamId: string | number) => Promise<void>
  deleteTeam: (team: EventTeam) => Promise<void>

  // ── Player actions ──
  addPlayer: () => Promise<void>
  savePlayerEdit: (playerId: string | number) => Promise<void>
  assignPlayerToTeam: (playerId: string | number, teamId: string | number | null) => Promise<void>
  assignPlayerToTeamWithRole: (playerId: string | number, teamId: string | number | null, role: string, rank: string) => Promise<void>
  removePlayerFromTeam: (playerId: string | number) => Promise<void>
  removePlayer: (player: EventPlayer) => Promise<void>

  // ── Match actions ──
  createMatch: () => Promise<void>
  updateMatchStartDate: (matchId: string | number, startDate: string) => Promise<void>
  generateTourneyBracket: (mode?: string) => Promise<void>
  clearTourneyBracket: () => Promise<void>
  deleteMatch: (matchId: string | number) => Promise<void>
  saveMatchup: (matchId: string | number) => Promise<boolean>
  reportMatchWinner: (matchId: string | number, winnerTeamId: string | number) => Promise<void>
  cancelMatchWinner: (matchId: string | number) => Promise<void>

  // ── Event settings actions ──
  syncEventEditDraftFromEvent: () => void
  saveEventEdit: () => Promise<void>
  deleteEvent: () => Promise<void>

  // ── Signup actions ──
  copySignupLink: () => Promise<void>
  rotateSignupLink: () => Promise<void>
  setSignupVisibility: (enabled: boolean) => Promise<void>
  setEventEnded: (ended: boolean) => Promise<void>
  acceptSignupRequest: (requestId: string | number) => Promise<void>
  declineSignupRequest: (requestId: string | number) => Promise<void>
}

export const eventCtxKey: InjectionKey<EventCtxType> = Symbol('eventCtx')
