<script setup>
import { computed, inject, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { averagePlayersElo } from '../../lib/elo'
import { getRoleIcon, sortPlayersByRoleThenName } from '../../lib/roles'
import PlayerCard from '../player/PlayerCard.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import AppBadge from '../ui/AppBadge.vue'

const ctx = inject('eventCtx')
const assignmentSearchByTeam = reactive({})
const teamPickerTeamId = ref('')
const teamPickerBusyPlayerId = ref('')
const teamPickerDialogRef = ref(null)
const teamPickerCloseButtonRef = ref(null)
let previouslyFocusedElement = null

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

function teamCreatedTimestamp(team) {
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

function normalizeSearch(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/\s+/g, ' ')
    .trim()
}

function searchTokens(value) {
  const normalized = normalizeSearch(value)
  return normalized ? normalized.split(' ') : []
}

function playerSearchBlob(player) {
  return normalizeSearch(`${player?.name || ''} ${player?.role || ''} ${player?.rank || ''} ${player?.team || ''}`)
}

function playerMatchesTokens(player, tokens) {
  if (tokens.length === 0) {
    return true
  }

  const blob = playerSearchBlob(player)
  return tokens.every((token) => blob.includes(token))
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

function playersForTeam(teamId) {
  if (!ctx.event) {
    return []
  }

  return sortPlayersByRoleThenName(
    ctx.event.players.filter((player) => player.team_id === teamId)
  )
}

function teamRoleCounts(teamId) {
  const counts = { Tank: 0, DPS: 0, Support: 0 }
  for (const player of playersForTeam(teamId)) {
    if (player.role === 'Tank' || player.role === 'DPS' || player.role === 'Support') {
      counts[player.role] += 1
    }
  }

  return counts
}

function roleStatusClass(teamId, role) {
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

function teamBalanceNeeds(teamId) {
  const counts = teamRoleCounts(teamId)
  const targets = pugRoleTargets.value
  const needs = []

  for (const role of ['Tank', 'DPS', 'Support']) {
    const missing = targets[role] - counts[role]
    if (missing > 0) {
      needs.push(`${role} x${missing}`)
    }
  }

  return needs.join(', ')
}

function teamBalanceExcess(teamId) {
  const counts = teamRoleCounts(teamId)
  const targets = pugRoleTargets.value
  const extra = []

  for (const role of ['Tank', 'DPS', 'Support']) {
    const overflow = counts[role] - targets[role]
    if (overflow > 0) {
      extra.push(`${role} x${overflow}`)
    }
  }

  return extra.join(', ')
}

function playersAssignableToTeam(teamId) {
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

function assignmentSearchValue(teamId) {
  return String(assignmentSearchByTeam[teamId] || '')
}

function setAssignmentSearch(teamId, value) {
  assignmentSearchByTeam[teamId] = String(value || '')
}

function clearAllAssignmentSearches() {
  for (const teamId of Object.keys(assignmentSearchByTeam)) {
    assignmentSearchByTeam[teamId] = ''
  }
}

async function selectAssignResult(teamId, playerId) {
  await ctx.assignPlayerToTeam(playerId, teamId)
  setAssignmentSearch(teamId, '')
}

function handleDocumentPointerDown(event) {
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

  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }

  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', onTeamPickerKeydown)
  }

  restoreTeamPickerFocus()
})

function openTeamPicker(teamId) {
  teamPickerTeamId.value = String(teamId || '')
}

function closeTeamPicker() {
  teamPickerTeamId.value = ''
  teamPickerBusyPlayerId.value = ''
}

async function assignUnassignedPlayerToPickedTeam(playerId) {
  const teamId = String(teamPickerTeamId.value || '')
  if (!teamId) {
    return
  }

  teamPickerBusyPlayerId.value = String(playerId)
  try {
    await ctx.assignPlayerToTeam(playerId, teamId)
    closeTeamPicker()
  } finally {
    teamPickerBusyPlayerId.value = ''
  }
}

function teamPickerFocusableElements() {
  if (!teamPickerDialogRef.value) {
    return []
  }

  const selectors = [
    'button:not([disabled])',
    '[href]',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ]

  return Array.from(teamPickerDialogRef.value.querySelectorAll(selectors.join(', '))).filter((el) => {
    return el.getAttribute('aria-hidden') !== 'true'
  })
}

function focusInitialTeamPickerElement() {
  nextTick(() => {
    if (teamPickerCloseButtonRef.value) {
      teamPickerCloseButtonRef.value.focus()
      return
    }

    if (teamPickerDialogRef.value) {
      teamPickerDialogRef.value.focus()
    }
  })
}

function restoreTeamPickerFocus() {
  if (previouslyFocusedElement && typeof previouslyFocusedElement.focus === 'function') {
    previouslyFocusedElement.focus()
  }

  previouslyFocusedElement = null
}

function onTeamPickerKeydown(event) {
  if (!isTeamPickerOpen.value) {
    return
  }

  if (event.key === 'Escape') {
    event.preventDefault()
    closeTeamPicker()
    return
  }

  if (event.key !== 'Tab') {
    return
  }

  const focusableElements = teamPickerFocusableElements()
  if (focusableElements.length === 0) {
    event.preventDefault()
    if (teamPickerDialogRef.value) {
      teamPickerDialogRef.value.focus()
    }
    return
  }

  const first = focusableElements[0]
  const last = focusableElements[focusableElements.length - 1]
  const active = document.activeElement

  if (!teamPickerDialogRef.value?.contains(active)) {
    event.preventDefault()
    first.focus()
    return
  }

  if (event.shiftKey && active === first) {
    event.preventDefault()
    last.focus()
    return
  }

  if (!event.shiftKey && active === last) {
    event.preventDefault()
    first.focus()
  }
}

watch(isTeamPickerOpen, (open) => {
  if (typeof document === 'undefined') {
    return
  }

  document.body.style.overflow = open ? 'hidden' : ''

  if (typeof window !== 'undefined') {
    if (open) {
      const active = document.activeElement
      if (active instanceof HTMLElement) {
        previouslyFocusedElement = active
      } else {
        previouslyFocusedElement = null
      }

      window.addEventListener('keydown', onTeamPickerKeydown)
      focusInitialTeamPickerElement()
    } else {
      window.removeEventListener('keydown', onTeamPickerKeydown)
      restoreTeamPickerFocus()
    }
  }
})

function filteredPlayersAssignableToTeam(teamId) {
  const players = playersAssignableToTeam(teamId)
  const tokens = searchTokens(assignmentSearchByTeam[teamId])
  if (tokens.length === 0) {
    return []
  }

  return players.filter((player) => {
    return playerMatchesTokens(player, tokens)
  })
}

function visibleTeamAssignResults(teamId) {
  return filteredPlayersAssignableToTeam(teamId).slice(0, 10)
}

function hasTeamAssignmentSearch(teamId) {
  return searchTokens(assignmentSearchByTeam[teamId]).length > 0
}

function startEditTeam(team) {
  ctx.editingTeamId = team.id
  ctx.editTeamName = team.name
}

function cancelEditTeam() {
  ctx.editingTeamId = null
  ctx.editTeamName = ''
}

function formatTeamAverageElo(teamId) {
  const rawAverage = averagePlayersElo(playersForTeam(teamId))
  if (rawAverage === null || rawAverage === undefined) {
    return 'N/A'
  }

  const average = Number(rawAverage)
  if (!Number.isFinite(average)) {
    return 'N/A'
  }

  return Math.round(average).toLocaleString('en-US')
}

function assignmentNotice(player) {
  if (!player?.team_id || !player?.team) {
    return ''
  }

  return `Currently in ${player.team}`
}

function playerInitials(name) {
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

function teamRankTierClass(rank) {
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

function teamMatchesCount(teamId) {
  return Number(teamMatchCountById.value[String(teamId)] || 0)
}

function formatTeamModified(team) {
  const raw = team?.updated_at || team?.created_at
  if (!raw) {
    return 'Just now'
  }

  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) {
    return 'Recently'
  }

  const diffMs = Date.now() - parsed.getTime()
  if (diffMs < 60 * 1000) {
    return 'Just now'
  }

  const mins = Math.floor(diffMs / (60 * 1000))
  if (mins < 60) {
    return `${mins} min${mins === 1 ? '' : 's'} ago`
  }

  const hours = Math.floor(mins / 60)
  if (hours < 24) {
    return `${hours}h ago`
  }

  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

</script>

<template>
  <section>
    <EventSectionHeader icon="shield" title="Team Management" />
    <div class="teams-layout" :class="{ 'is-readonly': !ctx.canManageEvent }">
      <aside v-if="ctx.canManageEvent" class="teams-sidebar">
        <p class="teams-sidebar-kicker">Quick setup</p>
        <form class="grid-form" @submit.prevent="ctx.createTeam">
          <label>
            Team name
            <input v-model="ctx.newTeamName" placeholder="e.g. Team Alpha, Phoenix Squad..." />
          </label>
          <button type="submit" class="btn-primary teams-primary-cta" :disabled="!ctx.canCreateTeam || ctx.creatingTeam">
            {{ ctx.creatingTeam ? 'Creating team...' : '+ Create Team' }}
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
            {{ ctx.creatingSoloTeams ? 'Creating solo teams...' : `Auto-create solo teams (${unassignedPlayersCount})` }}
          </button>
          <button
            v-if="!isOneVOneFormat"
            class="btn-secondary sidebar-utility-btn"
            :disabled="ctx.balancingTeams || ctx.event.teams.length === 0"
            @click="ctx.autoBalanceTeams"
          >
            <span class="material-symbols-rounded sidebar-utility-btn-icon" aria-hidden="true">auto_fix_high</span>
            {{ ctx.balancingTeams ? 'Balancing teams...' : 'Best team setup (ELO)' }}
          </button>
          <p class="muted solo-team-help">
            <template v-if="isOneVOneFormat">
              "Auto-create" creates one solo team per unassigned player.
            </template>
            <template v-else>
              "Best setup" rebalances existing teams using rank ELO calculations.
            </template>
          </p>
        </div>

        <div v-if="ctx.lastBalanceSummary" class="balance-report-box" role="status" aria-live="polite">
          <p class="balance-report-title">Last auto-balance report</p>
          <p class="balance-report-text">{{ ctx.lastBalanceSummary }}</p>
        </div>

        <div v-if="!ctx.isTourneyEvent && ctx.event.teams.length > 0" class="balance-helper-panel card">
          <div class="balance-helper-head">
            <span class="balance-helper-head-main">
              <span class="material-symbols-rounded balance-helper-info-icon" aria-hidden="true">info</span>
              <p class="balance-helper-title">PUG balance assistant</p>
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
            Current roster supports <span class="balance-helper-summary-highlight">{{ maxBalancedTeamsFromRoster }} fully balanced teams</span> for standard competitive {{ effectivePugFormat }}.
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
        <p v-if="ctx.event.teams.length === 0" class="muted">No teams yet. Create teams first.</p>
        <ul v-else class="entry-list" :class="{ 'is-single': orderedTeams.length === 1 }">
          <li v-for="team in orderedTeams" :key="team.id" class="team-row">
            <div class="list-main">
              <div v-if="ctx.editingTeamId === team.id" class="inline-edit-row">
                <input v-model="ctx.editTeamName" placeholder="Team name" />
                <button
                  class="btn-primary icon-btn"
                  :disabled="Boolean(ctx.savingTeamEdits[team.id])"
                  :title="ctx.savingTeamEdits[team.id] ? 'Saving team' : 'Save team'"
                  @click="ctx.saveTeamEdit(team.id)"
                >
                  <span class="material-symbols-rounded" aria-hidden="true">
                    {{ ctx.savingTeamEdits[team.id] ? 'hourglass_top' : 'save' }}
                  </span>
                  <span class="sr-only">{{ ctx.savingTeamEdits[team.id] ? 'Saving team' : 'Save team' }}</span>
                </button>
                <button class="btn-secondary icon-btn" title="Cancel editing team" @click="cancelEditTeam">
                  <span class="material-symbols-rounded" aria-hidden="true">close</span>
                  <span class="sr-only">Cancel editing team</span>
                </button>
              </div>
              <span v-else class="entry-title-row">
                <span class="entry-title">{{ team.name }}</span>
              </span>
              <div class="team-meta-row muted">
                <span class="team-meta-count">{{ team.player_ids.length }} players</span>
                <span class="team-meta-elo">AVG ELO: {{ formatTeamAverageElo(team.id) }}</span>
              </div>
              <div v-if="ctx.canManageEvent && !ctx.isTourneyEvent" class="team-balance-row">
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'Tank')]" :label="`Tank ${teamRoleCounts(team.id).Tank}/${pugRoleTargets.Tank}`" />
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'DPS')]" :label="`DPS ${teamRoleCounts(team.id).DPS}/${pugRoleTargets.DPS}`" />
                <AppBadge :variant="{ ok: 'ok', missing: 'warning', excess: 'danger' }[roleStatusClass(team.id, 'Support')]" :label="`Support ${teamRoleCounts(team.id).Support}/${pugRoleTargets.Support}`" />
              </div>
              <p v-if="ctx.canManageEvent && !ctx.isTourneyEvent && teamBalanceNeeds(team.id)" class="muted team-balance-note">Needs: {{ teamBalanceNeeds(team.id) }}</p>
              <p v-if="ctx.canManageEvent && !ctx.isTourneyEvent && teamBalanceExcess(team.id)" class="muted team-balance-note">Over target: {{ teamBalanceExcess(team.id) }}</p>
              <ul v-if="playersForTeam(team.id).length > 0" class="team-player-list">
                <li v-for="player in playersForTeam(team.id)" :key="player.id" class="team-player-item">
                  <span class="team-player-main">
                    <span class="team-player-avatar" :title="player.name" aria-hidden="true">{{ playerInitials(player.name) }}</span>
                    <span class="team-player-stack">
                      <span class="team-player-name">{{ player.name }}</span>
                      <span class="team-player-rank">
                        Rank:
                        <span class="team-player-rank-value" :class="teamRankTierClass(player.rank)">
                          {{ player.rank || 'Unranked' }}
                        </span>
                      </span>
                    </span>
                    <span class="team-player-role">
                      <span class="material-symbols-rounded team-role-icon" aria-hidden="true">{{ getRoleIcon(player.role) }}</span>
                      <span>{{ player.role }}</span>
                    </span>
                  </span>
                  <button
                    v-if="ctx.canManageEvent"
                    class="btn-secondary icon-btn team-player-remove"
                    :disabled="Boolean(ctx.savingPlayerTeams[player.id])"
                    :title="ctx.savingPlayerTeams[player.id] ? 'Removing from team' : 'Remove from team'"
                    @click="ctx.removePlayerFromTeam(player.id)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">
                      {{ ctx.savingPlayerTeams[player.id] ? 'hourglass_top' : 'link_off' }}
                    </span>
                    <span class="sr-only">{{ ctx.savingPlayerTeams[player.id] ? 'Removing from team' : 'Remove from team' }}</span>
                  </button>
                </li>
              </ul>
              <span v-else class="muted team-player-empty">No players assigned</span>
              <div v-if="ctx.canManageEvent" class="team-assign-grid">
                <p v-if="playersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">No available players to assign</p>
                <div v-else class="team-assign-row">
                  <label class="sr-only" :for="`assign-search-${team.id}`">Search assignable players for {{ team.name }}</label>
                  <input
                    :id="`assign-search-${team.id}`"
                    :value="assignmentSearchValue(team.id)"
                    type="search"
                    placeholder="Search player, role, rank..."
                    @input="setAssignmentSearch(team.id, $event.target.value)"
                  />
                  <p v-if="hasTeamAssignmentSearch(team.id)" class="muted team-assign-match-count">{{ filteredPlayersAssignableToTeam(team.id).length }} matches</p>

                  <div
                    v-if="hasTeamAssignmentSearch(team.id)"
                    class="team-assign-dropdown"
                    role="listbox"
                    :aria-label="`Search results for ${team.name}`"
                  >
                    <p v-if="filteredPlayersAssignableToTeam(team.id).length === 0" class="muted team-player-empty">No players match this search.</p>

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
                          <span class="team-assign-main">{{ player.name }} · {{ player.role }} · {{ player.rank }}</span>
                          <span v-if="assignmentNotice(player)" class="team-assign-notice">{{ assignmentNotice(player) }}</span>
                        </button>
                      </li>
                    </ul>

                    <p v-if="filteredPlayersAssignableToTeam(team.id).length > 10" class="muted team-assign-limit-note">
                      Showing first 10 matches. Refine search to narrow results.
                    </p>
                  </div>
                </div>
              </div>
              <div class="team-card-footer muted">
                <span>Last modified: {{ formatTeamModified(team) }}</span>
                <span>{{ teamMatchesCount(team.id) }} matches</span>
              </div>
            </div>
            <div class="team-actions">
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-secondary icon-btn"
                title="Add unassigned player"
                @click="openTeamPicker(team.id)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">add</span>
                <span class="sr-only">Add unassigned player</span>
              </button>
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-secondary icon-btn"
                title="Edit team"
                @click="startEditTeam(team)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">edit</span>
                <span class="sr-only">Edit team</span>
              </button>
              <button
                v-if="ctx.canManageEvent && ctx.editingTeamId !== team.id"
                class="btn-danger icon-btn"
                :disabled="Boolean(ctx.deletingTeams[team.id])"
                :title="ctx.deletingTeams[team.id] ? 'Deleting team' : 'Delete team'"
                @click="ctx.deleteTeam(team)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">
                  {{ ctx.deletingTeams[team.id] ? 'hourglass_top' : 'delete' }}
                </span>
                <span class="sr-only">{{ ctx.deletingTeams[team.id] ? 'Deleting team' : 'Delete team' }}</span>
              </button>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <div v-if="teamPickerTeamId" class="team-picker-backdrop" @click.self="closeTeamPicker">
      <section
        ref="teamPickerDialogRef"
        class="team-picker-modal"
        role="dialog"
        aria-modal="true"
        aria-label="Assign unassigned player"
        tabindex="-1"
      >
        <div class="team-picker-header">
          <h4>Add unassigned player</h4>
          <button ref="teamPickerCloseButtonRef" class="btn-secondary icon-btn" title="Close picker" @click="closeTeamPicker">
            <span class="material-symbols-rounded" aria-hidden="true">close</span>
            <span class="sr-only">Close picker</span>
          </button>
        </div>

        <p class="muted team-picker-subtitle">
          Assign an available player to {{ teamPickerTarget?.name || 'this team' }}.
        </p>

        <p v-if="unassignedPlayers.length === 0" class="muted">All players are already assigned.</p>

        <ul v-else class="team-picker-list">
          <li v-for="player in unassignedPlayers" :key="`picker-player-${player.id}`">
            <PlayerCard
              class="team-picker-item"
              :class="{ 'is-disabled': Boolean(teamPickerBusyPlayerId) }"
              :player="player"
              :clickable="!teamPickerBusyPlayerId"
              @select="assignUnassignedPlayerToPickedTeam(player.id)"
            />
          </li>
        </ul>
      </section>
    </div>

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
  border-radius: var(--radius-item);
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

.team-balance-note {
  margin: 0;
  font-size: 0.84rem;
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
  border-radius: var(--radius-item);
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

.team-picker-backdrop {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(7, 12, 22, 0.64);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.team-picker-modal {
  width: min(42rem, calc(100vw - 2rem));
  max-height: min(80vh, 54rem);
  overflow: auto;
  border: 1px solid color-mix(in srgb, var(--line-strong) 72%, var(--brand-1) 28%);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--card) 95%, #101a2c 5%);
  padding: 0.82rem;
  display: grid;
  gap: 0.55rem;
}

.team-picker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.team-picker-header h4 {
  margin: 0;
}

.team-picker-subtitle {
  margin: 0;
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
  border-radius: var(--radius-item);
  box-shadow: 0 var(--radius-item) 24px rgba(16, 39, 82, 0.18);
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
