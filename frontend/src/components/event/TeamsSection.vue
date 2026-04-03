<script setup lang="ts">
import { computed, inject, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { averagePlayersElo } from '../../lib/elo'
import { getLocale } from '../../i18n'
import { getRoleIcon, sortPlayersByRoleThenName } from '../../lib/roles'
import PlayerCard from '../player/PlayerCard.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import AppBadge from '../ui/AppBadge.vue'
import AppModal from '../ui/AppModal.vue'
import type { EventCtxType } from '../../composables/event/event-inject'
import type { EventPlayer, EventTeam, RoleRank } from '../../types'

const { t } = useI18n()
const ctx = inject<EventCtxType>('eventCtx')!
const assignmentSearchByTeam = reactive<Record<string, string>>({})
const teamPickerTeamId = ref<string>('')
const teamPickerBusyPlayerId = ref<string>('')

const isTeamPickerOpen = computed(() => Boolean(teamPickerTeamId.value))

const isOneVOneFormat = computed(() => {
  return String(ctx.event?.format || '').trim().toLowerCase() === '1v1'
})

const effectivePugFormat = computed(() => {
  const format = String(ctx.event?.format || '').trim().toLowerCase()
  if (format === '6v6') {
    return '6v6'
  }

  return '5v5'
})

const pugRoleTargets = computed(() => {
  if (effectivePugFormat.value === '6v6') {
    return { Tank: 2, DPS: 2, Support: 2 }
  }

  return { Tank: 1, DPS: 2, Support: 2 }
})

const pugTeamSize = computed(() => {
  const targets = pugRoleTargets.value
  return targets.Tank + targets.DPS + targets.Support
})

const rosterRoleCounts = computed(() => {
  if (!ctx.event) {
    return { Tank: 0, DPS: 0, Support: 0 }
  }

  const counts = { Tank: 0, DPS: 0, Support: 0 }
  for (const player of ctx.event.players) {
    if (player.role === 'Tank' || player.role === 'DPS' || player.role === 'Support') {
      counts[player.role] += 1
    }
  }

  return counts
})

const maxBalancedTeamsFromRoster = computed(() => {
  if (!ctx.event) {
    return 0
  }

  const targets = pugRoleTargets.value
  const roles = rosterRoleCounts.value
  const byRole = Math.min(
    Math.floor(roles.Tank / targets.Tank),
    Math.floor(roles.DPS / targets.DPS),
    Math.floor(roles.Support / targets.Support)
  )

  const byTotal = Math.floor(ctx.event.players.length / pugTeamSize.value)
  return Math.max(0, Math.min(byRole, byTotal))
})

const autoBalanceSupportsTeamCount = computed(() => {
  return Number(ctx.event?.teams?.length || 0) === 2
})

const autoBalanceDisabled = computed(() => {
  return (
    ctx.balancingTeams
    || ctx.teamsAreAlreadyBalanced
    || !ctx.event
    || ctx.event.teams.length === 0
    || !autoBalanceSupportsTeamCount.value
  )
})

function teamCreatedTimestamp(team: EventTeam): number {
  const raw = team?.created_at || team?.updated_at || ''
  const parsed = new Date(raw).getTime()
  return Number.isNaN(parsed) ? Number.POSITIVE_INFINITY : parsed
}

const orderedTeams = computed(() => {
  if (!ctx.event?.teams) {
    return []
  }

  return [...ctx.event.teams].sort((a, b) => {
    const diff = teamCreatedTimestamp(a) - teamCreatedTimestamp(b)
    if (diff !== 0) {
      return diff
    }

    return String(a?.name || '').localeCompare(String(b?.name || ''))
  })
})

const teamPickerTarget = computed(() => {
  const targetId = String(teamPickerTeamId.value || '')
  if (!targetId) {
    return null
  }

  return orderedTeams.value.find((team) => team.id === targetId) || null
})

const teamMatchCountById = computed(() => {
  const counts = Object.create(null)
  const matches = ctx.event?.matches || []

  for (const match of matches) {
    const teamAId = String(match?.team_a_id || '')
    const teamBId = String(match?.team_b_id || '')

    if (teamAId) {
      counts[teamAId] = (counts[teamAId] || 0) + 1
    }

    if (teamBId) {
      counts[teamBId] = (counts[teamBId] || 0) + 1
    }
  }

  return counts
})

function normalizeSearch(value: unknown): string {
  return String(value || '')
    .toLowerCase()
    .replace(/\s+/g, ' ')
    .trim()
}

function searchTokens(value: unknown): string[] {
  const normalized = normalizeSearch(value)
  return normalized ? normalized.split(' ') : []
}

function playerSearchBlob(player: EventPlayer): string {
  const roleBlob = Array.isArray(player?.roles) && player.roles.length > 0
    ? player.roles.map((rp: RoleRank) => `${rp.role || ''} ${rp.rank || ''}`).join(' ')
    : `${player?.role || ''} ${player?.rank || ''}`
  return normalizeSearch(`${player?.name || ''} ${roleBlob} ${player?.team || ''}`)
}

function playerRolesDisplay(player: EventPlayer): string {
  if (Array.isArray(player?.roles) && player.roles.length > 0) {
    return player.roles.map((rp: RoleRank) => `${rp.role} · ${rp.rank}`).join(' / ')
  }
  return `${player?.role || ''} · ${player?.rank || ''}`
}

function playerMatchesTokens(player: EventPlayer, tokens: string[]): boolean {
  if (tokens.length === 0) {
    return true
  }

  const blob = playerSearchBlob(player)
  return tokens.every((token: string) => blob.includes(token))
}

const unassignedPlayers = computed(() => {
  if (!ctx.event) {
    return []
  }

  return ctx.event.players
    .filter((player) => !player.team_id)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const unassignedPlayersCount = computed(() => {
  return unassignedPlayers.value.length
})

function playersForTeam(teamId: string | number): EventPlayer[] {
  if (!ctx.event) {
    return []
  }

  return sortPlayersByRoleThenName(
    ctx.event.players.filter((player) => player.team_id === teamId)
  )
}

function teamRoleCounts(teamId: string | number): { Tank: number; DPS: number; Support: number } {
  const counts = { Tank: 0, DPS: 0, Support: 0 }
  for (const player of playersForTeam(teamId)) {
    const displayRole = player.assigned_role || player.role
    if (displayRole === 'Tank' || displayRole === 'DPS' || displayRole === 'Support') {
      counts[displayRole] += 1
    }
  }

  return counts
}

function roleStatusClass(teamId: string | number, role: 'Tank' | 'DPS' | 'Support'): string {
  const count = teamRoleCounts(teamId)[role]
  const target = pugRoleTargets.value[role]
  if (count < target) {
    return 'missing'
  }
  if (count > target) {
    return 'excess'
  }

  return 'ok'
}

function playersAssignableToTeam(teamId: string | number): EventPlayer[] {
  if (!ctx.event) {
    return []
  }

  return ctx.event.players
    .filter((player) => player.team_id !== teamId)
    .sort((a, b) => {
      const aUnassigned = !a.team_id
      const bUnassigned = !b.team_id
      if (aUnassigned !== bUnassigned) {
        return aUnassigned ? -1 : 1
      }

      return a.name.localeCompare(b.name)
    })
}

function assignmentSearchValue(teamId: string | number): string {
  return String(assignmentSearchByTeam[teamId] || '')
}

function setAssignmentSearch(teamId: string | number, value: string) {
  assignmentSearchByTeam[teamId] = String(value || '')
}

function clearAllAssignmentSearches() {
  for (const teamId of Object.keys(assignmentSearchByTeam)) {
    assignmentSearchByTeam[teamId] = ''
  }
}

async function selectAssignResult(teamId: string | number, playerId: string | number) {
  await ctx.assignPlayerToTeam(String(playerId), String(teamId))
  setAssignmentSearch(teamId, '')
}

function handleDocumentPointerDown(event: PointerEvent) {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }

  if (target.closest('.team-assign-row')) {
    return
  }

  clearAllAssignmentSearches()
}

onMounted(() => {
  document.addEventListener('pointerdown', handleDocumentPointerDown)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', handleDocumentPointerDown)
})

function openTeamPicker(teamId: string | number) {
  teamPickerTeamId.value = String(teamId || '')
}

function closeTeamPicker() {
  teamPickerTeamId.value = ''
  teamPickerBusyPlayerId.value = ''
}

async function assignUnassignedPlayerToPickedTeam(playerId: string | number) {
  const teamId = String(teamPickerTeamId.value || '')
  if (!teamId) {
    return
  }

  teamPickerBusyPlayerId.value = String(playerId)
  try {
    await ctx.assignPlayerToTeam(String(playerId), teamId)
    closeTeamPicker()
  } finally {
    teamPickerBusyPlayerId.value = ''
  }
}

async function assignUnassignedPlayerToPickedTeamWithRole(playerId: string | number, role: string, rank: string) {
  const teamId = String(teamPickerTeamId.value || '')
  if (!teamId) {
    return
  }

  teamPickerBusyPlayerId.value = String(playerId)
  try {
    await ctx.assignPlayerToTeamWithRole(String(playerId), teamId, role, rank)
    closeTeamPicker()
  } finally {
    teamPickerBusyPlayerId.value = ''
  }
}

function filteredPlayersAssignableToTeam(teamId: string | number): EventPlayer[] {
  const players = playersAssignableToTeam(teamId)
  const tokens = searchTokens(assignmentSearchByTeam[teamId])
  if (tokens.length === 0) {
    return []
  }

  return players.filter((player) => {
    return playerMatchesTokens(player, tokens)
  })
}

function visibleTeamAssignResults(teamId: string | number): EventPlayer[] {
  return filteredPlayersAssignableToTeam(teamId).slice(0, 10)
}

function hasTeamAssignmentSearch(teamId: string | number): boolean {
  return searchTokens(assignmentSearchByTeam[teamId]).length > 0
}

function startEditTeam(team: EventTeam) {
  ctx.editingTeamId = team.id
  ctx.editTeamName = team.name
}

function cancelEditTeam() {
  ctx.editingTeamId = null
  ctx.editTeamName = ''
}

function formatTeamAverageElo(teamId: string | number): string {
  const rawAverage = averagePlayersElo(playersForTeam(teamId))
  if (rawAverage === null || rawAverage === undefined) return t('teams.naElo')
  const average = Number(rawAverage)
  if (!Number.isFinite(average)) return t('teams.naElo')
  return Math.round(average).toLocaleString(getLocale())
}

function assignmentNotice(player: EventPlayer): string {
  if (!player?.team_id || !player?.team) return ''
  return t('teams.playerInTeam', { team: player.team })
}

function playerInitials(name: string): string {
  const value = String(name || '').trim()
  if (!value) {
    return '?'
  }

  const parts = value.split(/\s+/).filter(Boolean)
  if (parts.length === 1) {
    return parts[0].slice(0, 1).toUpperCase()
  }

  return `${parts[0][0] || ''}${parts[1][0] || ''}`.toUpperCase()
}

function teamRankTierClass(rank: string): string {
  const normalized = String(rank || '').trim().toLowerCase()
  if (normalized.startsWith('bronze')) return 'rank-tier-bronze'
  if (normalized.startsWith('silver')) return 'rank-tier-silver'
  if (normalized.startsWith('gold')) return 'rank-tier-gold'
  if (normalized.startsWith('platinum')) return 'rank-tier-platinum'
  if (normalized.startsWith('diamond')) return 'rank-tier-diamond'
  if (normalized.startsWith('master')) return 'rank-tier-master'
  if (normalized.startsWith('grandmaster')) return 'rank-tier-grandmaster'
  if (normalized.startsWith('champion')) return 'rank-tier-champion'
  if (normalized.startsWith('top 500') || normalized.startsWith('top500')) return 'rank-tier-top500'
  return 'rank-tier-unranked'
}

function teamMatchesCount(teamId: string | number): number {
  return Number(teamMatchCountById.value[String(teamId)] || 0)
}

function formatTeamModified(team: EventTeam): string {
  const raw = team?.updated_at || team?.created_at
  if (!raw) return t('teams.justNow')
  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) return t('teams.recently')
  const diffMs = Date.now() - parsed.getTime()
  if (diffMs < 60 * 1000) return t('teams.justNow')
  const mins = Math.floor(diffMs / (60 * 1000))
  if (mins < 60) return t('teams.minsAgo', { n: mins })
  const hours = Math.floor(mins / 60)
  if (hours < 24) return t('teams.hoursAgo', { n: hours })
  const days = Math.floor(hours / 24)
  return t('teams.daysAgo', { n: days })
}

</script>

<template>
  <section>
    <EventSectionHeader icon="shield" :title="t('teams.sectionTitle')" />
    <div class="teams-layout" :class="{ 'is-readonly': !ctx.canManageEvent }">
      <aside v-if="ctx.canManageEvent" class="teams-sidebar">
        <p class="teams-sidebar-kicker">{{ t('teams.quickSetup') }}</p>
        <form class="grid-form" @submit.prevent="ctx.createTeam">
          <label>
            {{ t('teams.teamNameLabel') }}
            <input v-model="ctx.newTeamName" :placeholder="t('teams.teamNamePlaceholder')" />
          </label>
          <button type="submit" class="btn-primary teams-primary-cta" :disabled="!ctx.canCreateTeam || ctx.creatingTeam">
            {{ ctx.creatingTeam ? t('teams.creating') : t('teams.createTeamBtn') }}
          </button>
        </form>

        <div class="solo-team-action-row">
          <button
            v-if="isOneVOneFormat"
            class="btn-secondary sidebar-utility-btn"
            :disabled="ctx.creatingSoloTeams || unassignedPlayersCount === 0"
            @click="ctx.autoCreateSoloTeams"
          >
            <span class="material-symbols-rounded sidebar-utility-btn-icon" aria-hidden="true">group_add</span>
            {{ ctx.creatingSoloTeams ? t('teams.creatingAutoSolo') : t('teams.autoCreateSolo', { count: unassignedPlayersCount }) }}
          </button>
          <button
            v-if="!isOneVOneFormat"
            class="btn-secondary sidebar-utility-btn"
            :disabled="autoBalanceDisabled"
            @click="ctx.autoBalanceTeams"
          >
            <span class="material-symbols-rounded sidebar-utility-btn-icon" aria-hidden="true">auto_fix_high</span>
            {{ ctx.balancingTeams ? t('teams.balancing') : t('teams.autoBalance') }}
          </button>
          <p class="muted solo-team-help">
            <template v-if="isOneVOneFormat">
              {{ t('teams.helpAutoCreate') }}
            </template>
            <template v-else-if="!autoBalanceSupportsTeamCount">
              {{ t('teams.helpBalance2Teams') }}
            </template>
            <template v-else>
              {{ t('teams.helpBestSetup') }}
            </template>
          </p>
        </div>

        <div v-if="ctx.lastBalanceSummary" class="balance-report-box" role="status" aria-live="polite">
          <p class="balance-report-title">{{ t('teams.balanceReport') }}</p>
          <p class="balance-report-text">{{ ctx.lastBalanceSummary }}</p>
        </div>

        <div v-if="!ctx.isTourneyEvent && (ctx.event?.teams.length ?? 0) > 0" class="balance-helper-panel card">
          <div class="balance-helper-head">
            <span class="balance-helper-head-main">
              <span class="material-symbols-rounded balance-helper-info-icon" aria-hidden="true">info</span>
              <p class="balance-helper-title">{{ t('teams.balanceHelper') }}</p>
            </span>
            <AppBadge
              bg="color-mix(in srgb, var(--primary-300) 20%, var(--card) 80%)"
              color="var(--primary-200)"
              border="color-mix(in srgb, var(--primary-300) 70%, var(--line) 30%)"
              radius="pill"
              :label="`FORMAT: ${effectivePugFormat}`"
            />
          </div>
          <p class="balance-helper-summary">
            {{ t('teams.balanceRosterInfo', { max: maxBalancedTeamsFromRoster, format: effectivePugFormat }) }}
          </p>
          <div class="balance-roster-row">
            <span class="balance-roster-chip">
              <span class="material-symbols-rounded balance-roster-role-icon" aria-hidden="true">{{ getRoleIcon('Tank') }}</span>
              <span class="balance-roster-role-name">Tank</span>
              <span class="balance-roster-role-count">{{ rosterRoleCounts.Tank }}/{{ pugRoleTargets.Tank * Math.max(1, maxBalancedTeamsFromRoster) }}</span>
            </span>
            <span class="balance-roster-chip">
              <span class="material-symbols-rounded balance-roster-role-icon" aria-hidden="true">{{ getRoleIcon('DPS') }}</span>
              <span class="balance-roster-role-name">DPS</span>
              <span class="balance-roster-role-count">{{ rosterRoleCounts.DPS }}/{{ pugRoleTargets.DPS * Math.max(1, maxBalancedTeamsFromRoster) }}</span>
            </span>
            <span class="balance-roster-chip">
              <span class="material-symbols-rounded balance-roster-role-icon" aria-hidden="true">{{ getRoleIcon('Support') }}</span>
              <span class="balance-roster-role-name">Support</span>
              <span class="balance-roster-role-count">{{ rosterRoleCounts.Support }}/{{ pugRoleTargets.Support * Math.max(1, maxBalancedTeamsFromRoster) }}</span>
            </span>
          </div>
        </div>
      </aside>

      <div class="teams-board">
        <p v-if="(ctx.event?.teams.length ?? 0) === 0" class="muted">{{ t('teams.noTeams') }}</p>
        <ul v-else class="entry-list" :class="{ 'is-single': orderedTeams.length === 1 }">
          <li v-for="team in orderedTeams" :key="team.id" class="team-row">
            <div class="list-main">
              <div v-if="ctx.editingTeamId === team.id" class="inline-edit-row">
                <input v-model="ctx.editTeamName" :placeholder="t('teams.teamNameEditPlaceholder')" />
                <button
                  class="btn-primary icon-btn"
                  :disabled="Boolean(ctx.savingTeamEdits[team.id])"
                  :title="ctx.savingTeamEdits[team.id] ? t('teams.savingTeam') : t('teams.saveTeam')"
                  @click="ctx.saveTeamEdit(team.id)"
                >
                  <span class="material-symbols-rounded" aria-hidden="true">
                    {{ ctx.savingTeamEdits[team.id] ? 'hourglass_top' : 'save' }}
                  </span>
                  <span class="sr-only">{{ ctx.savingTeamEdits[team.id] ? t('teams.savingTeam') : t('teams.saveTeam') }}</span>
                </button>
                <button class="btn-secondary icon-btn" :title="t('teams.cancelTeam')" @click="cancelEditTeam">
                  <span class="material-symbols-rounded" aria-hidden="true">close</span>
                  <span class="sr-only">{{ t('teams.cancelTeam') }}</span>
                </button>
              </div>
              <span v-else class="entry-title-row">
                <span class="entry-title">{{ team.name }}</span>
              </span>
              <div class="team-meta-row muted">
                <span class="team-meta-count">{{ t('teams.teamPlayers', { count: team.player_ids?.length ?? 0 }) }}</span>
                <span class="team-meta-elo">{{ t('teams.avgElo', { elo: formatTeamAverageElo(team.id) }) }}</span>
              </div>
              <div v-if="ctx.canManageEvent && !ctx.isTourneyEvent" class="team-balance-row">
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'Tank')]" :label="`Tank ${teamRoleCounts(team.id).Tank}/${pugRoleTargets.Tank}`" />
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'DPS')]" :label="`DPS ${teamRoleCounts(team.id).DPS}/${pugRoleTargets.DPS}`" />
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'Support')]" :label="`Support ${teamRoleCounts(team.id).Support}/${pugRoleTargets.Support}`" />
              </div>
              <ul v-if="playersForTeam(team.id).length > 0" class="team-player-list">
                <li v-for="player in playersForTeam(team.id)" :key="player.id" class="team-player-item">
                  <span class="team-player-main">
                    <span class="team-player-avatar" :title="player.name" aria-hidden="true">{{ playerInitials(player.name) }}</span>
                    <span class="team-player-stack">
                      <span class="team-player-name">{{ player.name }}</span>
                      <span class="team-player-rank">
                        {{ t('teams.rankLabel') }}
                        <span class="team-player-rank-value" :class="teamRankTierClass(player.assigned_rank || player.rank)">
                          {{ player.assigned_rank || player.rank || 'Unranked' }}
                        </span>
                      </span>
                    </span>
                    <span class="team-player-role">
                      <span class="material-symbols-rounded team-role-icon" aria-hidden="true">{{ getRoleIcon(player.assigned_role || player.role) }}</span>
                      <span>{{ player.assigned_role || player.role }}</span>
                    </span>
                  </span>
                  <button
                    v-if="ctx.canManageEvent"
                    class="btn-secondary icon-btn team-player-remove"
                    :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
                    :title="ctx.savingPlayerTeams[player.id] ? t('teams.removingFromTeam') : t('teams.removeFromTeam')"
                    @click="ctx.removePlayerFromTeam(player.id)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">
                      {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'link_off' }}
                    </span>
                    <span class="sr-only">{{ ctx.savingPlayerTeams[player.id] ? t('teams.removingFromTeam') : t('teams.removeFromTeam') }}</span>
                  </button>
                </li>
              </ul>
              <span v-else class="muted team-player-empty">{{ t('teams.noPlayersAssigned') }}</span>
              <div v-if="ctx.canManageEvent" class="team-assign-grid">
                <p v-if="playersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">{{ t('teams.noAvailablePlayers') }}</p>
                <div v-else class="team-assign-row">
                  <label class="sr-only" :for="`assign-search-${team.id}`">{{ t('teams.pickerSearchAria', { name: team.name }) }}</label>
                  <input
                    :id="`assign-search-${team.id}`"
                    :value="assignmentSearchValue(team.id)"
                    type="search"
                    :placeholder="t('teams.searchPlaceholder')"
                    @input="setAssignmentSearch(team.id, ($event.target as HTMLInputElement).value)"
                  />
                  <p v-if="hasTeamAssignmentSearch(team.id)" class="muted team-assign-match-count">{{ t('teams.searchResultsCount', { count: filteredPlayersAssignableToTeam(team.id).length }) }}</p>

                  <div
                    v-if="hasTeamAssignmentSearch(team.id)"
                    class="team-assign-dropdown"
                    role="listbox"
                    :aria-label="t('teams.searchResultsAria', { name: team.name })"
                  >
                    <p v-if="filteredPlayersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">{{ t('teams.searchNoResults') }}</p>

                    <ul v-else class="team-assign-results">
                      <li v-for="player in visibleTeamAssignResults(team.id)" :key="`assign-result-${team.id}-${player.id}`">
                        <button
                          class="btn-secondary team-assign-result-btn"
                          :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
                          @click="selectAssignResult(team.id, player.id)"
                        >
                          <span class="material-symbols-rounded" aria-hidden="true">
                            {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'person_add' }}
                          </span>
                          <span class="team-assign-main">{{ player.name }} · {{ playerRolesDisplay(player) }}</span>
                          <span v-if="assignmentNotice(player)" class="team-assign-notice">{{ assignmentNotice(player) }}</span>
                        </button>
                      </li>
                    </ul>

                    <p v-if="filteredPlayersAssignableToTeam(team.id).length > 10" class="muted team-assign-limit-note">
                      {{ t('teams.showingFirst10') }}
                    </p>
                  </div>
                </div>
              </div>
              <div class="team-card-footer muted">
                <span>{{ t('teams.lastModified', { date: formatTeamModified(team) }) }}</span>
                <span>{{ t('teams.teamMatchCount', { count: teamMatchesCount(team.id) }) }}</span>
              </div>
            </div>
            <div class="team-actions">
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-secondary icon-btn"
                :title="t('teams.addUnassigned')"
                @click="openTeamPicker(team.id)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">add</span>
                <span class="sr-only">{{ t('teams.addUnassigned') }}</span>
              </button>
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-secondary icon-btn"
                :title="t('teams.editTeam')"
                @click="startEditTeam(team)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">edit</span>
                <span class="sr-only">{{ t('teams.editTeam') }}</span>
              </button>
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-danger icon-btn"
                :disabled="Boolean(ctx.deletingTeams[team.id])"
                :title="ctx.deletingTeams[team.id] ? t('teams.deletingTeam') : t('teams.deleteTeam')"
                @click="ctx.deleteTeam(team)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">
                  {{ ctx.deletingTeams[team.id] ? 'hourglass_top' : 'delete' }}
                </span>
                <span class="sr-only">{{ ctx.deletingTeams[team.id] ? t('teams.deletingTeam') : t('teams.deleteTeam') }}</span>
              </button>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <AppModal
      :open="isTeamPickerOpen"
      :title="t('teams.pickerTitle', { name: teamPickerTarget?.name || 'this team' })"
      max-width="min(42rem, 100%)"
      @update:open="!$event && closeTeamPicker()"
    >
      <p v-if="unassignedPlayers.length === 0" class="muted">{{ t('teams.allAssigned') }}</p>

      <template v-else>
        <p class="team-picker-hint muted">{{ t('teams.pickerHint') }}</p>

        <ul class="team-picker-list">
          <li v-for="player in unassignedPlayers" :key="`picker-player-${player.id}`">
            <PlayerCard
              class="team-picker-item"
              :class="{ 'is-disabled': Boolean(teamPickerBusyPlayerId) }"
              :player="player"
              :clickable="!teamPickerBusyPlayerId"
              :show-socials="ctx.canManageEvent"
              @select="assignUnassignedPlayerToPickedTeam(player.id)"
              @selectRole="(p, rp) => assignUnassignedPlayerToPickedTeamWithRole(p.id, rp.role, rp.rank)"
            />
          </li>
        </ul>
      </template>
    </AppModal>

  </section>
</template>

<style scoped>
.teams-layout {
  display: grid;
  grid-template-columns: minmax(320px, 400px) minmax(0, 1fr);
  gap: 0.9rem;
  align-items: start;
}

.teams-layout.is-readonly {
  grid-template-columns: minmax(0, 1fr);
}

.teams-sidebar {
  padding: 0.92rem;
  display: grid;
  gap: 0.88rem;
}

.teams-sidebar-kicker {
  margin: 0;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--ink-2);
  font-weight: 700;
}

.teams-primary-cta {
  width: 100%;
}

.teams-board {
  min-width: 0;
}

.grid-form {
  display: grid;
  gap: 0.62rem;
  margin: 0;
}

.grid-form label {
  display: grid;
  gap: 0.28rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 0.72rem;
  font-weight: 700;
  color: var(--ink-2);
}

.solo-team-action-row {
  display: grid;
  gap: 0.45rem;
  margin: 0;
  padding-top: 0.52rem;
  border-top: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-1) 12%);
}

.solo-team-action-row .btn-secondary {
  width: 100%;
  justify-content: flex-start;
}

.sidebar-utility-btn {
  display: inline-flex;
  align-items: center;
  background: transparent;
  border-color: color-mix(in srgb, var(--line) 84%, var(--brand-1) 16%);
  color: color-mix(in srgb, white 95%, var(--ink-1) 5%);
  justify-content: flex-start;
  text-align: left;
  gap: 0.4rem;
}

.sidebar-utility-btn:hover:not(:disabled) {
  background: transparent;
  border-color: color-mix(in srgb, var(--brand-1) 46%, var(--line) 54%);
}

.sidebar-utility-btn-icon {
  font-size: 1rem;
  line-height: 1;
  color: #ffd25f;
}

.solo-team-help {
  margin: 0.15rem 0 0;
  font-style: italic;
  font-size: 0.72rem;
  line-height: 1.35;
}

.balance-report-box {
  border: 1px solid color-mix(in srgb, var(--line-strong) 82%, var(--line) 18%);
  background: var(--card);
  border-radius: var(--radius-md);
  padding: 0.52rem 0.6rem;
  margin: 0;
}

.balance-report-title {
  margin: 0;
  font-weight: 760;
  color: var(--ink-1);
}

.balance-report-text {
  margin: 0.2rem 0 0;
  color: var(--ink-2);
  font-size: 0.9rem;
}

.balance-helper-panel {
  padding: 0.74rem;
  margin: 0;
  display: grid;
  gap: 0.46rem;
}

.balance-helper-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.balance-helper-head-main {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
}

.balance-helper-info-icon {
  font-size: 1rem;
  line-height: 1;
  color: #ffd25f;
}

.balance-helper-title {
  margin: 0;
  font-weight: 760;
}

.balance-helper-summary {
  margin: 0;
  color: var(--ink-2);
}

.balance-helper-summary-highlight {
  color: color-mix(in srgb, white 96%, var(--ink-1) 4%);
}

.balance-roster-row {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.38rem;
}

.balance-roster-chip {
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 92%, #1b2840 8%);
  border-radius: var(--radius-pill);
  padding: 0.16rem 0.5rem;
  font-size: 0.82rem;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  gap: 0.26rem;
}

.balance-roster-role-icon {
  font-size: 0.95rem;
  color: var(--ink-muted);
}

.balance-roster-role-name {
  color: color-mix(in srgb, white 96%, var(--ink-1) 4%);
}

.balance-roster-role-count {
  color: var(--ink-muted);
}

.inline-edit-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  align-items: center;
  gap: 0.38rem;
}

.entry-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: start;
  gap: 0.62rem;
}

.entry-list.is-single {
  grid-template-columns: 1fr;
}

@media (max-width: 1280px) {
  .entry-list {
    grid-template-columns: 1fr;
  }
}

.team-row {
  border: 1px solid color-mix(in srgb, var(--line) 96%, var(--brand-1) 4%);
  background: transparent;
  border-radius: var(--radius-md);
  padding: 0.92rem 0.96rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: start;
  gap: 0.7rem;
  box-shadow: none;
  position: relative;
}

.list-main {
  min-width: 0;
  display: grid;
  gap: 0.34rem;
}

.entry-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
  font-size: 1rem;
}

.entry-title-row {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
  flex-wrap: wrap;
  padding-right: 4.4rem;
}

.team-meta-row {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 0.74rem;
  letter-spacing: 0.01em;
  padding-right: 4.4rem;
  justify-content: flex-start;
  flex-wrap: wrap;
  margin-bottom: 0.42rem;
}

.team-meta-count {
  min-width: 0;
}

.team-meta-elo {
  font-family: "Space Mono", ui-monospace, monospace;
  color: var(--ink-2);
  text-align: left;
  white-space: nowrap;
}

.team-balance-row {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.34rem;
  margin-bottom: 0.5rem;
}

.team-actions {
  position: absolute;
  top: 0.72rem;
  right: 0.76rem;
  display: flex;
  gap: 0.26rem;
  align-self: start;
}

.team-actions .icon-btn {
  min-width: 1.92rem;
  min-height: 1.92rem;
  width: 1.92rem;
  height: 1.92rem;
  padding: 0.18rem;
  background: transparent;
  border-color: transparent;
  color: var(--ink-muted);
}

.team-actions .icon-btn:hover:not(:disabled) {
  background: transparent;
  border-color: transparent;
  color: var(--ink-2);
}

.team-player-list {
  list-style: none;
  margin: 0;
  margin-bottom: 0.48rem;
  padding: 0;
  width: 100%;
  min-width: 0;
  display: grid;
  gap: 0;
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: transparent;
}

.team-player-item {
  display: flex;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  align-items: center;
  justify-content: flex-start;
  gap: 0.58rem;
  padding: 0.5rem 0.56rem;
  border: 0;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-2) 8%);
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.team-player-item:last-child {
  border-bottom: 0;
}

.team-player-item:hover {
  background: transparent;
}

.team-player-main {
  min-width: 0;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.42rem;
  flex: 1;
  padding-block: 0.08rem;
  border-right: 1px solid color-mix(in srgb, var(--line) 82%, var(--brand-2) 18%);
  padding-right: 0.42rem;
}

.team-player-stack {
  min-width: 0;
  display: grid;
  gap: 0.12rem;
}

.team-player-rank {
  font-size: 0.68rem;
  color: var(--ink-muted);
}

.team-player-rank-value {
  font-weight: 700;
}

.team-player-rank-value.rank-tier-bronze {
  color: #d39a63;
}

.team-player-rank-value.rank-tier-silver {
  color: #d1d7e0;
}

.team-player-rank-value.rank-tier-gold {
  color: #efd08a;
}

.team-player-rank-value.rank-tier-platinum {
  color: #8be1e5;
}

.team-player-rank-value.rank-tier-diamond {
  color: #b4cbff;
}

.team-player-rank-value.rank-tier-master {
  color: #d9b9ff;
}

.team-player-rank-value.rank-tier-grandmaster {
  color: #ffb0b0;
}

.team-player-rank-value.rank-tier-champion {
  color: #ffc4e3;
}

.team-player-rank-value.rank-tier-top500 {
  color: #ffb29c;
}

.team-player-rank-value.rank-tier-unranked {
  color: var(--ink-2);
}

.team-player-name {
  font-size: 0.86rem;
  font-weight: 700;
  line-height: 1;
  overflow-wrap: anywhere;
}

.team-player-role {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  justify-self: end;
  min-width: 78px;
  font-size: 0.76rem;
  line-height: 1;
  color: var(--ink-2);
}

.team-card-footer {
  margin-top: 0.26rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  font-size: 0.68rem;
  letter-spacing: 0.02em;
  border-top: 0;
  padding-top: 0;
}

.team-role-icon {
  font-size: 0.95rem;
}

.team-player-avatar {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-pill);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.62rem;
  font-weight: 800;
  letter-spacing: 0.02em;
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
  border: 1px solid color-mix(in srgb, var(--line) 78%, var(--brand-1) 22%);
  background: color-mix(in srgb, var(--card) 80%, #2f4463 20%);
}

.team-player-remove {
  margin-left: auto;
}

.team-player-remove.icon-btn {
  min-width: 2rem;
  min-height: 2rem;
  width: 2rem;
  height: 2rem;
  padding: 0.22rem;
  border-color: transparent;
  background: transparent;
  color: var(--ink-muted);
}

.team-player-remove.icon-btn:hover:not(:disabled) {
  border-color: transparent;
  background: transparent;
  color: var(--ink-2);
}

.team-player-remove .material-symbols-rounded {
  font-size: 1.05rem;
}

.team-player-empty {
  font-size: 0.9rem;
}

.team-picker-subtitle {
  margin: 0;
}

.team-picker-hint {
  margin: 0;
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--ink-2);
}

.team-picker-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.44rem;
}

.team-picker-item {
  width: 100%;
  color: var(--ink-1);
}

.team-picker-item.is-disabled {
  opacity: 0.62;
  cursor: progress;
  pointer-events: none;
}

.team-assign-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.4rem;
}

.team-assign-row {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.4rem;
  align-items: center;
}

.team-assign-row input[type="search"] {
  background: transparent;
}

.team-assign-match-count {
  margin: 0;
}

.team-assign-dropdown {
  position: absolute;
  top: calc(100% + 0.34rem);
  left: 0;
  right: auto;
  z-index: 24;
  width: max-content;
  min-width: min(26rem, 100%);
  max-width: min(42rem, calc(100vw - 2rem));
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 96%, #19253a 4%);
  border-radius: var(--radius-md);
  box-shadow: 0 12px 24px rgba(16, 39, 82, 0.18);
  padding: 0.45rem;
  display: grid;
  gap: 0.35rem;
}

.team-assign-results {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.32rem;
  justify-items: start;
  max-height: 14rem;
  overflow: auto;
}

/* Reset nested list item styles inherited from .entry-list li. */
.team-assign-results li {
  border: 0;
  background: transparent;
  border-radius: 0;
  padding: 0;
  display: block;
}

.team-assign-result-btn {
  width: auto;
  max-width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.team-assign-main {
  font-weight: 700;
}

.team-assign-notice {
  color: var(--ink-2);
  font-size: 0.82rem;
  margin-left: 1.58rem;
}

.team-assign-limit-note {
  margin: 0;
}

.team-assign-row .team-player-empty {
  margin: 0;
}

@media (max-width: 1100px) {
  .team-assign-row {
    grid-template-columns: 1fr;
  }

  .team-assign-dropdown {
    left: 0;
    right: 0;
    min-width: 0;
    max-width: none;
  }
}

@media (max-width: 900px) {
  .teams-layout {
    grid-template-columns: 1fr;
  }

  .team-row {
    grid-template-columns: 1fr;
  }

  .entry-title-row,
  .team-meta-row {
    padding-right: 0;
  }

  .team-actions {
    position: static;
    top: auto;
    right: auto;
    justify-content: flex-start;
  }

  .team-player-main {
    grid-template-columns: 24px minmax(0, 1fr);
    gap: 0.3rem 0.5rem;
    border-right: 0;
    padding-right: 0;
  }

  .team-player-role {
    grid-column: 2;
    justify-self: start;
    min-width: 0;
  }

}
</style>
