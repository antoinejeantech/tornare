<script setup lang="ts">
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { EventCtxType } from '../../composables/event/event-inject'
import type { EventMatch } from '../../types'

const ctx = inject<EventCtxType>('eventCtx')!
const { t } = useI18n()
const editingMatchups = ref<Record<string | number, boolean>>({})
const bracketWrapEl = ref<HTMLElement | null>(null)
const measuredCardHeight = ref(104)
const bracketLinks = ref<Array<{ id: string; d: string; status: string; isPlayIn: boolean }>>([])
let resizeObserver: ResizeObserver | null = null

function nextPowerOfTwo(value: number): number {
  let size = 1
  while (size < value) {
    size *= 2
  }
  return size
}

function bracketRoundsCount(size: number): number {
  let rounds = 0
  let current = size
  while (current > 1) {
    current /= 2
    rounds += 1
  }
  return Math.max(1, rounds)
}

function mainBracketSize(teamCount: number): number {
  const nextPow2 = nextPowerOfTwo(teamCount)
  if (teamCount === nextPow2) {
    return nextPow2
  }
  return nextPow2 / 2
}

function knockoutLabel(matchesInRound: number): string {
  if (matchesInRound <= 1) {
    return t('tourneyMatches.roundFinal')
  }
  if (matchesInRound === 2) {
    return t('tourneyMatches.roundSemifinals')
  }
  if (matchesInRound === 4) {
    return t('tourneyMatches.roundQuarterfinals')
  }
  return t('tourneyMatches.roundOf', { n: matchesInRound * 2 })
}

function buildPreviewRounds(teamCount: number) {
  const safeCount = Math.max(2, teamCount)
  const mainSize = mainBracketSize(safeCount)
  const playInCount = safeCount - mainSize
  const mainRoundStart = playInCount > 0 ? 2 : 1
  const mainRounds = bracketRoundsCount(mainSize)
  const rounds = []

  if (playInCount > 0) {
    rounds.push({
      round: 1,
      label: 'Play-In',
      slots: playInCount,
    })
  }

  for (let idx = 0; idx < mainRounds; idx += 1) {
    const round = mainRoundStart + idx
    const slots = Math.max(1, mainSize >> (idx + 1))
    rounds.push({
      round,
      label: knockoutLabel(slots),
      slots,
    })
  }

  return rounds
}

function roundLabelFromMatches(round: number, cards: EventMatch[]): string {
  const hasPlayInTitles = cards.some((card) => String(card.title || '').toLowerCase().startsWith('play-in'))
  if (hasPlayInTitles || round === 1 && cards.length > 0 && cards.every((card) => String(card.title || '').toLowerCase().startsWith('play-in'))) {
    return t('tourneyMatches.roundPlayIn')
  }

  return knockoutLabel(cards.length)
}

function availableTeamsForMatch(matchId: string | number) {
  const mid = String(matchId)
  const matches = ctx.event?.matches ?? []
  const excluded = new Set<string>()
  const preserved = new Set<string>()
  const currentMatch = matches.find((m) => String(m.id) === mid)
  const currentSelection = ctx.matchupSelections[mid]

  if (currentMatch?.team_a_id) preserved.add(String(currentMatch.team_a_id))
  if (currentMatch?.team_b_id) preserved.add(String(currentMatch.team_b_id))
  if (currentSelection?.teamAId) preserved.add(String(currentSelection.teamAId))
  if (currentSelection?.teamBId) preserved.add(String(currentSelection.teamBId))

  for (const m of matches) {
    if (String(m.id) === mid) continue
    if (!m.winner_team_id) {
      // Active (unfinished) match — its assigned teams are busy
      if (m.team_a_id) excluded.add(String(m.team_a_id))
      if (m.team_b_id) excluded.add(String(m.team_b_id))
    } else {
      // Completed match — loser is eliminated; winner only belongs to its next match
      const loserId = m.team_a_id === m.winner_team_id ? m.team_b_id : m.team_a_id
      if (loserId) excluded.add(String(loserId))
      const dest = m.next_match_id ? String(m.next_match_id) : null
      if (dest !== mid) excluded.add(String(m.winner_team_id))
    }
  }

  return (ctx.event?.teams ?? []).filter((t) => {
    const teamId = String(t.id)
    return preserved.has(teamId) || !excluded.has(teamId)
  })
}

function displayTeamName(match: EventMatch, slot: 'A' | 'B'): string {
  if (slot === 'A') {
    return match.team_a_name || 'TBD'
  }
  return match.team_b_name || 'TBD'
}

function nextMatchIsFull(match: EventMatch): boolean {
  if (!match.next_match_id) return false
  const nextMatch = ctx.event?.matches?.find((m) => String(m.id) === String(match.next_match_id))
  return Boolean(nextMatch && nextMatch.team_a_id && nextMatch.team_b_id)
}

function canReportWinner(match: EventMatch, teamId: string | number | null | undefined): boolean {
  if (!ctx.canManageEvent || !teamId) {
    return false
  }
  if (match.winner_team_id) {
    return false
  }
  if (nextMatchIsFull(match)) {
    return false
  }
  return Boolean(match.team_a_id && match.team_b_id)
}

function canCancelWinner(match: EventMatch): boolean {
  if (!ctx.canManageEvent) {
    return false
  }
  return Boolean(match.winner_team_id)
}

function isEditingMatchup(matchId: string | number): boolean {
  return Boolean(editingMatchups.value[matchId])
}

function toggleMatchupEditor(matchId: string | number) {
  editingMatchups.value = {
    ...editingMatchups.value,
    [matchId]: !editingMatchups.value[matchId],
  }
}

async function saveMatchupAndClose(matchId: string | number) {
  const saved = await ctx.saveMatchup(String(matchId))
  if (saved) {
    editingMatchups.value = {
      ...editingMatchups.value,
      [matchId]: false,
    }
  }
}

function roundListStyle(roundIndex: number) {
  const cardHeight = measuredCardHeight.value
  const baseGap = 16
  const rounds = bracketRounds.value
  const cardsCount = rounds[roundIndex]?.cards?.length || 1
  const maxCards = maxRoundCards.value
  const columnHeight = (maxCards * cardHeight) + ((maxCards - 1) * baseGap)

  // Distribute card centers evenly in the column and use symmetric top/bottom padding.
  const centerStep = columnHeight / Math.max(1, cardsCount)
  const gap = Math.max(0, centerStep - cardHeight)
  const edgePadding = Math.max(0, (centerStep - cardHeight) / 2)

  return {
    '--round-gap': `${gap}px`,
    '--round-pad': `${edgePadding}px`,
  }
}

const bracketRounds = computed(() => {
  const matches = Array.isArray(ctx.event?.matches) ? ctx.event.matches : []
  const roundMatches = matches.filter((match) => Number.isInteger(match.round) && Number.isInteger(match.position))

  // Use real generated structure when bracket exists; otherwise show a play-in aware preview.
  if (roundMatches.length > 0) {
    const uniqueRounds = [...new Set(roundMatches.map((match) => Number(match.round)))].sort((a, b) => a - b)

    return uniqueRounds.map((round) => {
      const roundExisting = roundMatches.filter((match) => Number(match.round) === round)
      const maxExistingPos = roundExisting.reduce((max, match) => Math.max(max, Number(match.position || 0)), 0)
      const slots = Math.max(1, maxExistingPos)

      const cards = []
      for (let position = 1; position <= slots; position += 1) {
        const found = roundExisting.find((match) => Number(match.position) === position)
        if (found) {
          cards.push(found)
        } else {
          cards.push({
            id: `placeholder-${round}-${position}`,
            title: `Round ${round} Match ${position}`,
            round,
            position,
            map: '',
            start_date: null,
            players: [],
            team_a_id: null,
            team_b_id: null,
            team_a_name: null,
            team_b_name: null,
            winner_team_id: null,
            winner_team_name: null,
            status: 'OPEN',
            isPlaceholder: true,
          })
        }
      }

      return {
        id: `round-${round}`,
        label: roundLabelFromMatches(round, cards),
        cards,
      }
    })
  }

  const preview = buildPreviewRounds(ctx.event?.teams?.length || 0)
  return preview.map((entry) => {
    const cards = []
    for (let position = 1; position <= entry.slots; position += 1) {
      cards.push({
        id: `placeholder-${entry.round}-${position}`,
        title: entry.round === 1 && entry.label === 'Play-In'
          ? `Play-In Match ${position}`
          : `Round ${entry.round} Match ${position}`,
        round: entry.round,
        position,
        map: '',
        start_date: null,
        players: [],
        team_a_id: null,
        team_b_id: null,
        team_a_name: null,
        team_b_name: null,
        winner_team_id: null,
        winner_team_name: null,
        status: 'OPEN',
        isPlaceholder: true,
      })
    }

    return {
      id: `round-${entry.round}`,
      label: entry.label,
      cards,
    }
  })
})

const maxRoundCards = computed(() => {
  if (!bracketRounds.value.length) {
    return 1
  }
  return Math.max(1, ...bracketRounds.value.map((round) => round.cards.length))
})

const hasGeneratedMatches = computed(() => {
  return Array.isArray(ctx.event?.matches) && ctx.event.matches.length > 0
})

const teamCount = computed(() => {
  return Array.isArray(ctx.event?.teams) ? ctx.event.teams.length : 0
})

const hasEnoughTeamsForBracket = computed(() => {
  return teamCount.value >= 2
})

const hasPlayedMatches = computed(() => {
  return Array.isArray(ctx.event?.matches) && ctx.event.matches.some((match) => Boolean(match.winner_team_id))
})

async function refreshMeasuredCardHeight() {
  const hasEditing = Object.values(editingMatchups.value).some(Boolean)
  if (hasEditing) {
    return
  }

  await nextTick()

  const root = bracketWrapEl.value
  if (!root) {
    return
  }

  const cards = root.querySelectorAll('.bracket-match')
  if (!cards.length) {
    return
  }

  let maxHeight = 0
  cards.forEach((card) => {
    maxHeight = Math.max(maxHeight, (card as HTMLElement).offsetHeight)
  })

  if (maxHeight > 0) {
    measuredCardHeight.value = maxHeight
  }
}

async function refreshBracketLinks() {
  await nextTick()
  const container = bracketWrapEl.value
  if (!container) { bracketLinks.value = []; return }
  const containerRect = container.getBoundingClientRect()
  const matches = Array.isArray(ctx.event?.matches) ? ctx.event.matches : []
  // Round 1 is a play-in round only when higher rounds also exist.
  const hasBracketRounds = matches.some(m => (m.round ?? 0) > 1)
  const links: Array<{ id: string; d: string; status: string; isPlayIn: boolean }> = []

  for (const match of matches) {
    if (!match.next_match_id || match.isPlaceholder) continue
    const fromEl = container.querySelector(`[data-match-id="${match.id}"]`) as HTMLElement | null
    const toEl   = container.querySelector(`[data-match-id="${match.next_match_id}"]`) as HTMLElement | null
    if (!fromEl || !toEl) continue

    const fromRect = fromEl.getBoundingClientRect()
    const toRect   = toEl.getBoundingClientRect()

    const x1 = fromRect.right - containerRect.left
    const y1 = fromRect.top + fromRect.height / 2 - containerRect.top
    const x2 = toRect.left - containerRect.left

    // Target the vertical centre of the actual team-row slot in the destination card.
    let y2: number
    if (match.next_match_slot === 'A' || match.next_match_slot === 'B') {
      const rowIndex = match.next_match_slot === 'A' ? 0 : 1
      const row = toEl.querySelectorAll<HTMLElement>('.match-team-row')[rowIndex]
      y2 = row
        ? row.getBoundingClientRect().top + row.getBoundingClientRect().height / 2 - containerRect.top
        : toRect.top + toRect.height / 2 - containerRect.top
    } else {
      y2 = toRect.top + toRect.height / 2 - containerRect.top
    }

    const isPlayIn = hasBracketRounds && match.round === 1
    let d: string
    if (isPlayIn) {
      // Bezier curve for play-in links: avoids crossing artefacts that occur
      // when multiple play-in vertical segments share the same midX column.
      const dx = Math.abs(x2 - x1) * 0.45
      d = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
    } else {
      // Right-angle path for bracket-internal links (QF → SF → F).
      const midX = (x1 + x2) / 2
      d = `M ${x1} ${y1} H ${midX} V ${y2} H ${x2}`
    }

    const status = match.winner_team_id ? 'completed'
                 : match.status === 'READY' ? 'ready'
                 : match.status === 'COMPLETED' ? 'completed'
                 : 'open'
    links.push({ id: String(match.id), d, status, isPlayIn })
  }

  bracketLinks.value = links
}

onMounted(async () => {
  await refreshMeasuredCardHeight()

  if (typeof ResizeObserver !== 'undefined' && bracketWrapEl.value) {
    resizeObserver = new ResizeObserver(() => {
      refreshMeasuredCardHeight()
      refreshBracketLinks()
    })
    resizeObserver.observe(bracketWrapEl.value)
  }
})

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
})

watch(bracketRounds, () => {
  refreshMeasuredCardHeight()
}, { deep: true })

watch(() => ctx.event?.matches, () => {
  refreshBracketLinks()
}, { deep: true, immediate: true })

watch(editingMatchups, () => {
  refreshMeasuredCardHeight()
  refreshBracketLinks()
}, { deep: true })
</script>

<template>
  <div class="tourney-root">
    <!-- Toolbar -->
    <div v-if="ctx.canManageEvent" class="tourney-toolbar">
      <div class="tourney-toolbar-actions">
        <button
          class="btn-primary toolbar-btn-random"
          type="button"
          :disabled="ctx.creatingMatch || ctx.clearingBracket || hasPlayedMatches || !hasEnoughTeamsForBracket"
          @click="ctx.generateTourneyBracket('random')"
        >
          <span class="material-symbols-rounded btn-icon" aria-hidden="true">shuffle</span>
          {{ ctx.creatingMatch ? t('tourneyMatches.generating') : t('tourneyMatches.generateRandom') }}
        </button>
        <button
          class="btn-secondary"
          type="button"
          :disabled="ctx.creatingMatch || ctx.clearingBracket || hasPlayedMatches || !hasEnoughTeamsForBracket"
          @click="ctx.generateTourneyBracket('empty')"
        >
          {{ ctx.creatingMatch ? t('tourneyMatches.generating') : t('tourneyMatches.generateEmpty') }}
        </button>
        <button
          class="btn-danger toolbar-btn-delete"
          type="button"
          :disabled="ctx.creatingMatch || ctx.clearingBracket || hasPlayedMatches || !hasGeneratedMatches"
          @click="ctx.clearTourneyBracket"
        >
          <span class="material-symbols-rounded btn-icon" aria-hidden="true">delete</span>
          {{ ctx.clearingBracket ? t('tourneyMatches.clearing') : t('tourneyMatches.deleteBracket') }}
        </button>
      </div>
      <p class="tourney-toolbar-hint muted">
        {{ hasPlayedMatches
          ? t('tourneyMatches.hintResultsLocked')
          : (!hasEnoughTeamsForBracket
            ? t('tourneyMatches.hintNotEnoughTeams')
            : ((ctx.event?.matches?.length ?? 0) > 0
              ? t('tourneyMatches.hintCanRegenerate')
              : t('tourneyMatches.hintInfo'))) }}
      </p>
    </div>

    <!-- Stats bar -->
    <div v-if="hasGeneratedMatches || teamCount > 0" class="bracket-stats">
      <div class="stat-item">
        <span class="stat-value">{{ ctx.event?.players?.length ?? 0 }}</span>
        <span class="stat-label">{{ t('tourneyMatches.totalPlayers') }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ teamCount }}</span>
        <span class="stat-label">{{ t('tourneyMatches.teamsRegistered') }}</span>
        <span v-if="hasGeneratedMatches" class="stat-sub muted">{{ t('tourneyMatches.fullBracket') }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ ctx.event?.matches?.length ?? 0 }}</span>
        <span class="stat-label">{{ t('tourneyMatches.totalMatches') }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">
          {{ ctx.event?.matches?.filter(m => m.winner_team_id).length ?? 0 }}
          <span class="stat-value-of muted">/ {{ ctx.event?.matches?.length ?? 0 }}</span>
        </span>
        <span class="stat-label">{{ t('tourneyMatches.matchesPlayed') }}</span>
      </div>
    </div>

    <!-- Empty state -->
    <p v-if="!hasGeneratedMatches && teamCount === 0" class="muted bracket-empty-message">
      {{ t('tourneyMatches.noTeams') }}
    </p>

    <!-- Bracket -->
    <div v-else class="tourney-bracket-wrap">
      <div
        ref="bracketWrapEl"
        class="tourney-bracket"
        :style="{
          '--rounds': bracketRounds.length,
          '--max-round-cards': maxRoundCards,
          '--match-height': measuredCardHeight + 'px',
        }"
      >
        <section
          v-for="(round, roundIndex) in bracketRounds"
          :key="round.id"
          class="bracket-round"
        >
          <h4 class="bracket-round-title">{{ round.label }}</h4>
          <div class="bracket-round-list" :style="roundListStyle(roundIndex)">
            <article
              v-for="match in round.cards"
              :key="match.id"
              :data-match-id="String(match.id)"
              class="bracket-match"
              :class="{
                'is-ready': match.status === 'READY',
                'is-completed': match.status === 'COMPLETED',
                'is-placeholder': match.isPlaceholder,
                'is-editing-card': ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id),
              }"
            >
              <!-- Card header: title + status badge -->
              <div class="match-header">
                <span
                  v-if="!match.isPlaceholder"
                  class="match-title-static"
                >
                  {{ match.title }}
                </span>
                <span v-else class="match-title-static">{{ match.title }}</span>
                <span
                  class="match-status-badge"
                  :class="`badge-${(match.winner_team_name ? 'completed' : match.status || 'open').toLowerCase()}`"
                >
                  {{ match.winner_team_name ? t('tourneyMatches.statusDone') : t('tourneyMatches.statusOpen') }}
                </span>
                <button
                  v-if="ctx.canManageEvent && !match.isPlaceholder && !match.winner_team_id && match.team_a_id && match.team_b_id && nextMatchIsFull(match)"
                  class="match-warning-icon"
                  type="button"
                  :data-tooltip="t('tourneyMatches.nextMatchFull')"
                  :aria-label="t('tourneyMatches.nextMatchFullAria')"
                >⚠️</button>
              </div>

              <!-- Team rows -->
              <div class="match-teams">
                <!-- Team A -->
                <div
                  class="match-team-row"
                  :class="{
                    'is-winner': match.winner_team_id === match.team_a_id && match.team_a_id,
                    'is-editing': ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id),
                  }"
                >
                  <select
                    v-if="ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id)"
                    v-model="ctx.matchupSelections[match.id].teamAId"
                    class="bracket-team-select"
                    :disabled="Boolean(ctx.savingMatchups[match.id]) || Boolean(ctx.reportingWinners[match.id]) || Boolean(ctx.cancellingWinners[match.id])"
                    @click.stop
                  >
                    <option value="">{{ t('tourneyMatches.chooseTeam') }}</option>
                    <option v-for="team in availableTeamsForMatch(match.id)" :key="`t-a-${match.id}-${team.id}`" :value="String(team.id)">
                      {{ team.name }}
                    </option>
                  </select>
                  <span v-else class="team-name">{{ displayTeamName(match, 'A') }}</span>
                  <button
                    v-if="canReportWinner(match, match.team_a_id) && !isEditingMatchup(match.id)"
                    class="btn-secondary win-btn"
                    type="button"
                    :disabled="Boolean(ctx.reportingWinners[match.id])"
                    @click="ctx.reportMatchWinner(match.id, match.team_a_id || '')"
                  >{{ t('tourneyMatches.win') }}</button>
                </div>

                <div class="match-teams-divider" aria-hidden="true"></div>

                <!-- Team B -->
                <div
                  class="match-team-row"
                  :class="{
                    'is-winner': match.winner_team_id === match.team_b_id && match.team_b_id,
                    'is-editing': ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id),
                  }"
                >
                  <select
                    v-if="ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id)"
                    v-model="ctx.matchupSelections[match.id].teamBId"
                    class="bracket-team-select"
                    :disabled="Boolean(ctx.savingMatchups[match.id]) || Boolean(ctx.reportingWinners[match.id]) || Boolean(ctx.cancellingWinners[match.id])"
                    @click.stop
                  >
                    <option value="">{{ t('tourneyMatches.chooseTeam') }}</option>
                    <option v-for="team in availableTeamsForMatch(match.id)" :key="`t-b-${match.id}-${team.id}`" :value="String(team.id)">
                      {{ team.name }}
                    </option>
                  </select>
                  <span v-else class="team-name">{{ displayTeamName(match, 'B') }}</span>
                  <button
                    v-if="canReportWinner(match, match.team_b_id) && !isEditingMatchup(match.id)"
                    class="btn-secondary win-btn"
                    type="button"
                    :disabled="Boolean(ctx.reportingWinners[match.id])"
                    @click="ctx.reportMatchWinner(match.id, match.team_b_id || '')"
                  >{{ t('tourneyMatches.win') }}</button>
                </div>
              </div>

              <!-- Admin controls (collapse when not needed) -->
              <div v-if="ctx.canManageEvent && !match.isPlaceholder" class="match-admin-row">
                <button
                  class="btn-secondary admin-btn"
                  type="button"
                  :disabled="Boolean(ctx.savingMatchups[match.id]) || Boolean(ctx.reportingWinners[match.id]) || Boolean(ctx.cancellingWinners[match.id])"
                  @click="toggleMatchupEditor(match.id)"
                >
                  {{ isEditingMatchup(match.id) ? t('tourneyMatches.close') : t('tourneyMatches.editMatchup') }}
                </button>
                <button
                  v-if="isEditingMatchup(match.id)"
                  class="btn-secondary admin-btn"
                  type="button"
                  :disabled="Boolean(ctx.savingMatchups[match.id]) || Boolean(ctx.reportingWinners[match.id]) || Boolean(ctx.cancellingWinners[match.id])"
                  @click="saveMatchupAndClose(match.id)"
                >
                  {{ ctx.savingMatchups[match.id] ? t('tourneyMatches.savingMatchup') : t('tourneyMatches.saveMatchup') }}
                </button>
                <button
                  v-if="canCancelWinner(match) && !isEditingMatchup(match.id)"
                  class="btn-danger admin-btn"
                  type="button"
                  :disabled="Boolean(ctx.cancellingWinners[match.id]) || Boolean(ctx.reportingWinners[match.id])"
                  @click="ctx.cancelMatchWinner(match.id)"
                >
                  {{ ctx.cancellingWinners[match.id] ? t('tourneyMatches.cancellingResult') : t('tourneyMatches.cancelResult') }}
                </button>
              </div>
            </article>
          </div>
        </section>

      <!-- SVG overlay: bracket connectors -->
      <svg
        v-if="bracketLinks.length"
        class="bracket-connectors-svg"
        aria-hidden="true"
      >
        <path
          v-for="link in bracketLinks"
          :key="link.id"
          :d="link.d"
          class="bracket-connector-path"
          :class="[`connector-${link.status}`, { 'connector-playin': link.isPlayIn }]"
        />
      </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Root ─────────────────────────────────────── */
.tourney-root {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  min-width: 0;
}

/* ── Toolbar ──────────────────────────────────── */
.tourney-toolbar {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  flex-wrap: wrap;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  padding: 0.85rem 1rem;
  background: var(--bg-1);
}

.tourney-toolbar-actions {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  flex-wrap: wrap;
  flex-shrink: 0;
  padding-right: 1rem;
  border-right: 1px solid var(--line);
}

.btn-icon {
  font-size: 1rem;
  line-height: 1;
  vertical-align: middle;
  margin-right: 0.2rem;
}

.tourney-toolbar-hint {
  flex: 1;
  min-width: 180px;
  font-size: 0.82rem;
  line-height: 1.5;
  margin: 0;
}

/* ── Empty state ──────────────────────────────── */
.bracket-empty-message {
  margin: 0;
}

/* ── Bracket scroll wrapper  ─────────────────── */
.tourney-bracket-wrap {
  overflow-x: auto;
  /* Explicitly constrain width so the scroll container is bounded.
     min-width:0 on the root makes this resolve to the available space. */
  width: 100%;
  padding-bottom: 0.25rem;
}

/* ── Bracket grid ─────────────────────────────── */
.tourney-bracket {
  --card-min-height: 104px;
  --base-round-gap: 0.75rem;
  --col-gap: 18px;
  --column-height: calc(
    (var(--max-round-cards) * var(--card-min-height)) +
    ((var(--max-round-cards) - 1) * var(--base-round-gap))
  );
  display: grid;
  grid-template-columns: repeat(var(--rounds), minmax(200px, 1fr));
  gap: var(--col-gap);
  min-width: max-content;
  align-items: stretch;
  position: relative;
}

/* ── Round column ─────────────────────────────── */
.bracket-round {
  display: grid;
  gap: 0.6rem;
}

.bracket-round-title {
  margin: 0;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-2);
}

.bracket-round-list {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: var(--round-gap, 0.75rem);
  padding-top: var(--round-pad, 0px);
  padding-bottom: var(--round-pad, 0px);
  min-height: var(--column-height);
  position: relative;
}

/* ── Match card ───────────────────────────────── */
.bracket-match {
  position: relative;
  z-index: 1;
  /* All cards in a column must share the same height so that top: 50% on
     connectors resolves to the same absolute Y. Use min-height (not height)
     so content is never clipped; JS measures the tallest card and injects
     --match-height so shorter cards pad up to match it. */
  min-height: var(--match-height, var(--card-min-height));
  background: var(--bg-1);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  padding: 0.55rem 0.65rem 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

/* Let editing cards expand naturally; measurement is paused while editing. */
.bracket-match.is-editing-card {
  min-height: var(--match-height, var(--card-min-height));
}

.bracket-match.is-ready {
  border-color: color-mix(in srgb, var(--brand-1) 35%, var(--line) 65%);
}

.bracket-match.is-completed {
  border-color: color-mix(in srgb, #17a36b 40%, var(--line) 60%);
}

.bracket-match.is-placeholder {
  opacity: 0.65;
  border-style: dashed;
}

/* ── Card header ──────────────────────────────── */
.match-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.4rem;
  min-width: 0;
}

.match-title-static {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--ink-3);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

/* ── Status badge ─────────────────────────────── */
.match-status-badge {
  flex-shrink: 0;
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  padding: 0.16rem 0.42rem;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.badge-open {
  background: color-mix(in srgb, var(--ink-3) 12%, transparent);
  color: var(--ink-2);
  border-color: color-mix(in srgb, var(--line) 90%, transparent);
}

.badge-ready {
  background: color-mix(in srgb, var(--brand-1) 15%, transparent);
  color: var(--brand-1);
  border-color: color-mix(in srgb, var(--brand-1) 28%, transparent);
}

.badge-completed, .badge-done {
  background: color-mix(in srgb, #1da56f 12%, transparent);
  color: #1da56f;
  border-color: color-mix(in srgb, #1da56f 28%, transparent);
}

/* ── Team rows ────────────────────────────────── */
.match-teams {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex: 1;
}

.match-teams-divider {
  height: 1px;
  background: var(--line);
}

.match-team-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.4rem;
  padding: 0.32rem 0.5rem;
  background: transparent;
  transition: background 0.15s;
}

.match-team-row.is-winner {
  background: color-mix(in srgb, #1da56f 10%, transparent);
}

.team-name {
  font-size: 0.84rem;
  font-weight: 600;
  color: var(--ink-2);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.match-team-row.is-winner .team-name {
  color: #1da56f;
  font-weight: 700;
}

.bracket-team-select {
  min-width: 0;
  flex: 1;
  max-width: 100%;
  font-size: 0.82rem;
}

.win-btn {
  flex-shrink: 0;
  padding: 0.15rem 0.38rem;
  font-size: 0.74rem;
}

/* ── Admin controls ───────────────────────────── */
.match-warning-icon {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  background: none;
  cursor: help;
  flex-shrink: 0;
  font-size: 0.85rem;
  line-height: 1;
}

.match-warning-icon::after {
  content: attr(data-tooltip);
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  width: max-content;
  max-width: min(200px, 80vw);
  padding: 0.3rem 0.5rem;
  background: #1c1917;
  color: #fef9c3;
  font-size: 0.7rem;
  font-weight: 400;
  letter-spacing: normal;
  text-transform: none;
  line-height: 1.4;
  border-radius: 4px;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 20;
  white-space: normal;
  text-align: left;
}

.match-warning-icon:hover::after,
.match-warning-icon:focus-visible::after {
  opacity: 1;
}

/* Lift the card above its siblings so the tooltip isn't clipped by adjacent cards. */
.bracket-match:has(.match-warning-icon:hover),
.bracket-match:has(.match-warning-icon:focus-visible) {
  z-index: 10;
}

.match-admin-row {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.admin-btn {
  padding: 0.14rem 0.4rem;
  font-size: 0.73rem;
  white-space: nowrap;
}

/* ── Play-in SVG connectors ──────────────────── */
.bracket-connectors-svg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
  z-index: 0;
}

.bracket-connector-path {
  fill: none;
  stroke-width: 1.5;
  /* Sharp corners for right-angle bracket paths */
  stroke-linecap: square;
  stroke-linejoin: miter;
}

/* Play-in links: smooth bezier curve + dashed stroke */
.connector-playin {
  stroke-dasharray: 5 3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* Status-based colours */
.connector-open {
  stroke: color-mix(in srgb, var(--line) 80%, transparent 20%);
  stroke-opacity: 0.7;
}

.connector-ready {
  stroke: color-mix(in srgb, var(--brand-1) 60%, var(--line) 40%);
  stroke-opacity: 0.85;
}

.connector-completed {
  stroke: color-mix(in srgb, #1da56f 60%, var(--line) 40%);
  stroke-opacity: 0.85;
}

/* ── Stats bar ────────────────────────────────── */
.bracket-stats {
  display: flex;
  gap: 0;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.stat-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.18rem;
  padding: 0.85rem 1.1rem;
  border-right: 1px solid var(--line);
}

.stat-item:last-child {
  border-right: none;
}

.stat-value {
  font-size: 1.35rem;
  font-weight: 800;
  color: var(--ink-1);
  line-height: 1;
  display: flex;
  align-items: baseline;
  gap: 0.25rem;
}

.stat-value-of {
  font-size: 0.85rem;
  font-weight: 500;
}

.stat-label {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--ink-2);
}

.stat-sub {
  font-size: 0.72rem;
  line-height: 1.3;
}

/* ── Responsive ───────────────────────────────── */
@media (max-width: 900px) {
  .tourney-toolbar {
    flex-direction: column;
    gap: 0.75rem;
  }

  .tourney-toolbar-actions {
    border-right: none;
    border-bottom: 1px solid var(--line);
    padding-right: 0;
    padding-bottom: 0.75rem;
    width: 100%;
  }

  .tourney-toolbar-hint {
    min-width: 0;
  }

  .tourney-bracket {
    grid-template-columns: repeat(var(--rounds), minmax(180px, 1fr));
    --col-gap: 40px;
  }

  .bracket-round-list {
    gap: 1.25rem;
    min-height: auto;
  }

  .bracket-match {
    min-height: var(--card-min-height);
  }

  .bracket-stats {
    flex-wrap: wrap;
  }

  .stat-item {
    flex: 1 1 40%;
    border-bottom: 1px solid var(--line);
  }

  .stat-item:last-child {
    border-bottom: none;
  }
}
</style>
