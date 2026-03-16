<script setup>
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

const ctx = inject('eventCtx')
const router = useRouter()
const editingMatchups = ref({})
const bracketWrapEl = ref(null)
const measuredCardHeight = ref(104)
let resizeObserver = null

function openMatch(matchId) {
  router.push({ name: 'match', params: { id: matchId } })
}

function nextPowerOfTwo(value) {
  let size = 1
  while (size < value) {
    size *= 2
  }
  return size
}

function bracketRoundsCount(size) {
  let rounds = 0
  let current = size
  while (current > 1) {
    current /= 2
    rounds += 1
  }
  return Math.max(1, rounds)
}

function mainBracketSize(teamCount) {
  const nextPow2 = nextPowerOfTwo(teamCount)
  if (teamCount === nextPow2) {
    return nextPow2
  }
  return nextPow2 / 2
}

function knockoutLabel(matchesInRound) {
  if (matchesInRound <= 1) {
    return 'Final'
  }
  if (matchesInRound === 2) {
    return 'Semifinals'
  }
  if (matchesInRound === 4) {
    return 'Quarterfinals'
  }
  return `Round of ${matchesInRound * 2}`
}

function buildPreviewRounds(teamCount) {
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

function roundLabelFromMatches(round, cards) {
  const hasPlayInTitles = cards.some((card) => String(card.title || '').toLowerCase().startsWith('play-in'))
  if (hasPlayInTitles || round === 1 && cards.length > 0 && cards.every((card) => String(card.title || '').toLowerCase().startsWith('play-in'))) {
    return 'Play-In'
  }

  return knockoutLabel(cards.length)
}

function displayTeamName(match, slot) {
  if (slot === 'A') {
    return match.team_a_name || 'TBD'
  }
  return match.team_b_name || 'TBD'
}

function canReportWinner(match, teamId) {
  if (!ctx.canManageEvent || !teamId) {
    return false
  }
  if (match.winner_team_id) {
    return false
  }
  return Boolean(match.team_a_id && match.team_b_id)
}

function canCancelWinner(match) {
  if (!ctx.canManageEvent) {
    return false
  }
  return Boolean(match.winner_team_id)
}

function isEditingMatchup(matchId) {
  return Boolean(editingMatchups.value[matchId])
}

function toggleMatchupEditor(matchId) {
  editingMatchups.value = {
    ...editingMatchups.value,
    [matchId]: !editingMatchups.value[matchId],
  }
}

async function saveMatchupAndClose(matchId) {
  const saved = await ctx.saveMatchup(matchId)
  if (saved) {
    editingMatchups.value = {
      ...editingMatchups.value,
      [matchId]: false,
    }
  }
}

function roundListStyle(roundIndex) {
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

  const previousCardsCount = rounds[roundIndex - 1]?.cards?.length || cardsCount
  const childCenterStep = roundIndex > 0
    ? roundCenterStep(previousCardsCount, cardHeight, baseGap)
    : 0
  return {
    '--round-gap': `${gap}px`,
    '--round-pad': `${edgePadding}px`,
    '--child-center-step': `${childCenterStep}px`,
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

const linkageInfo = computed(() => {
  const matches = Array.isArray(ctx.event?.matches) ? ctx.event.matches : []
  const hasParent = new Set()
  const hasNext = new Set()

  for (const match of matches) {
    const id = String(match.id)
    if (match.next_match_id) {
      hasNext.add(id)
      hasParent.add(String(match.next_match_id))
    }
  }

  return { hasParent, hasNext }
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

const previewLinkageInfo = computed(() => {
  const hasParent = new Set()
  const hasNext = new Set()

  const teamCount = Math.max(2, ctx.event?.teams?.length || 0)
  const mainSize = mainBracketSize(teamCount)
  const playInCount = teamCount - mainSize
  const mainRoundStart = playInCount > 0 ? 2 : 1
  const mainRounds = bracketRoundsCount(mainSize)

  const placeholderId = (round, position) => `placeholder-${round}-${position}`

  // Main bracket internal links (quarterfinals -> semifinals -> final, etc.)
  for (let idx = 0; idx < mainRounds - 1; idx += 1) {
    const round = mainRoundStart + idx
    const matchesInRound = Math.max(1, mainSize >> (idx + 1))

    for (let position = 1; position <= matchesInRound; position += 1) {
      const currentId = placeholderId(round, position)
      const parentRound = round + 1
      const parentPosition = Math.floor((position + 1) / 2)
      const parentId = placeholderId(parentRound, parentPosition)

      hasNext.add(currentId)
      hasParent.add(parentId)
    }
  }

  // Play-in links are wired exactly like backend: direct slots first, then play-in slots.
  if (playInCount > 0) {
    const directCount = teamCount - (playInCount * 2)
    const firstRoundMatches = Math.max(1, mainSize / 2)
    const slots = []

    for (let idx = 0; idx < directCount; idx += 1) {
      slots.push({ type: 'direct' })
    }
    for (let idx = 0; idx < playInCount; idx += 1) {
      slots.push({ type: 'playin', playInIdx: idx })
    }

    for (let position = 1; position <= firstRoundMatches; position += 1) {
      const slotA = slots[(position - 1) * 2]
      const slotB = slots[(position - 1) * 2 + 1]
      const parentId = placeholderId(mainRoundStart, position)

      if (slotA?.type === 'playin') {
        const playInId = placeholderId(1, slotA.playInIdx + 1)
        hasNext.add(playInId)
        hasParent.add(parentId)
      }
      if (slotB?.type === 'playin') {
        const playInId = placeholderId(1, slotB.playInIdx + 1)
        hasNext.add(playInId)
        hasParent.add(parentId)
      }
    }
  }

  return { hasParent, hasNext }
})

function hasParentLink(match) {
  const id = String(match.id)
  if (hasGeneratedMatches.value) {
    return linkageInfo.value.hasParent.has(id)
  }
  return previewLinkageInfo.value.hasParent.has(id)
}

function hasNextLink(match) {
  const id = String(match.id)
  if (hasGeneratedMatches.value) {
    return linkageInfo.value.hasNext.has(id)
  }
  return previewLinkageInfo.value.hasNext.has(id)
}

function showOutgoingLink(match, round, roundIndex) {
  if (!hasNextLink(match)) {
    return false
  }

  const nextRound = bracketRounds.value[roundIndex + 1]
  if (!nextRound) {
    return false
  }

  return hasRegularTransition(nextRound.cards.length, round.cards.length)
}

function showParentFork(match, round, roundIndex) {
  if (!hasParentLink(match) || roundIndex <= 0) {
    return false
  }

  const previousRound = bracketRounds.value[roundIndex - 1]
  if (!previousRound) {
    return false
  }

  return hasRegularTransition(round.cards.length, previousRound.cards.length)
}

function hasRegularTransition(parentCount, childCount) {
  return childCount === parentCount * 2
}

function roundCenterStep(cardsCount, cardHeight = 212, baseGap = 16) {
  const effectiveCardHeight = measuredCardHeight.value || cardHeight
  const maxCards = maxRoundCards.value
  const columnHeight = (maxCards * effectiveCardHeight) + ((maxCards - 1) * baseGap)
  const safeCards = Math.max(1, cardsCount)

  // Keep connector geometry in sync with roundListStyle center distribution.
  return columnHeight / safeCards
}

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
    maxHeight = Math.max(maxHeight, card.offsetHeight)
  })

  if (maxHeight > 0) {
    measuredCardHeight.value = maxHeight
  }
}

onMounted(async () => {
  await refreshMeasuredCardHeight()

  if (typeof ResizeObserver !== 'undefined' && bracketWrapEl.value) {
    resizeObserver = new ResizeObserver(() => {
      refreshMeasuredCardHeight()
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

watch(editingMatchups, () => {
  refreshMeasuredCardHeight()
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
          {{ ctx.creatingMatch ? 'Generating…' : 'Generate Random Bracket' }}
        </button>
        <button
          class="btn-secondary"
          type="button"
          :disabled="ctx.creatingMatch || ctx.clearingBracket || hasPlayedMatches || !hasEnoughTeamsForBracket"
          @click="ctx.generateTourneyBracket('empty')"
        >
          {{ ctx.creatingMatch ? 'Generating…' : 'Generate Empty Bracket' }}
        </button>
        <button
          class="btn-danger toolbar-btn-delete"
          type="button"
          :disabled="ctx.creatingMatch || ctx.clearingBracket || hasPlayedMatches || !hasGeneratedMatches"
          @click="ctx.clearTourneyBracket"
        >
          <span class="material-symbols-rounded btn-icon" aria-hidden="true">delete</span>
          {{ ctx.clearingBracket ? 'Clearing…' : 'Delete Bracket' }}
        </button>
      </div>
      <p class="tourney-toolbar-hint muted">
        {{ hasPlayedMatches
          ? 'At least one match result is set, so bracket regeneration and deletion are disabled.'
          : (!hasEnoughTeamsForBracket
            ? 'Create at least 2 teams to generate a tournament bracket.'
            : (ctx.event.matches.length > 0
              ? 'No match has been played yet. You can regenerate in random/empty mode or delete the generated bracket.'
              : 'Choose random generation for auto-seeded matchups, or empty generation to assign matchups manually. All changes are saved automatically.')) }}
      </p>
    </div>

    <!-- Stats bar -->
    <div v-if="hasGeneratedMatches || teamCount > 0" class="bracket-stats">
      <div class="stat-item">
        <span class="stat-value">{{ ctx.event?.players?.length ?? 0 }}</span>
        <span class="stat-label">Total Players</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ teamCount }}</span>
        <span class="stat-label">Teams Registered</span>
        <span v-if="hasGeneratedMatches" class="stat-sub muted">Full Bracket</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ ctx.event?.matches?.length ?? 0 }}</span>
        <span class="stat-label">Total Matches</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">
          {{ ctx.event?.matches?.filter(m => m.winner_team_id).length ?? 0 }}
          <span class="stat-value-of muted">/ {{ ctx.event?.matches?.length ?? 0 }}</span>
        </span>
        <span class="stat-label">Matches Played</span>
      </div>
    </div>

    <!-- Empty state -->
    <p v-if="!hasGeneratedMatches && teamCount === 0" class="muted bracket-empty-message">
      No teams created yet. Create teams first to preview or generate the tournament bracket.
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
          :class="{ 'has-connectors': roundIndex > 0, 'round-first': roundIndex === 0 }"
        >
          <h4 class="bracket-round-title">{{ round.label }}</h4>
          <div class="bracket-round-list" :style="roundListStyle(roundIndex)">
            <article
              v-for="match in round.cards"
              :key="match.id"
              class="bracket-match"
              :class="{
                'is-ready': match.status === 'READY',
                'is-completed': match.status === 'COMPLETED',
                'is-placeholder': match.isPlaceholder,
                'is-editing-card': ctx.canManageEvent && !match.isPlaceholder && isEditingMatchup(match.id),
              }"
            >
              <!-- Bracket connectors -->
              <span
                v-if="showOutgoingLink(match, round, roundIndex)"
                class="child-outgoing-link"
                aria-hidden="true"
              ></span>
              <span v-if="showParentFork(match, round, roundIndex)" class="parent-incoming-link" aria-hidden="true">
                <span class="fork-segment fork-spine"></span>
                <span class="fork-segment fork-arm-top"></span>
                <span class="fork-segment fork-arm-bottom"></span>
                <span class="fork-segment fork-arm-right"></span>
              </span>

              <!-- Card header: title + status badge -->
              <div class="match-header">
                <button
                  v-if="!match.isPlaceholder"
                  class="match-title-btn"
                  type="button"
                  @click="openMatch(match.id)"
                >
                  {{ match.title }}
                </button>
                <span v-else class="match-title-static">{{ match.title }}</span>
                <span
                  class="match-status-badge"
                  :class="`badge-${(match.winner_team_name ? 'completed' : match.status || 'open').toLowerCase()}`"
                >
                  {{ match.winner_team_name ? 'Done' : (match.status || 'Open') }}
                </span>
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
                    <option value="">Choose team</option>
                    <option v-for="team in ctx.event.teams" :key="`t-a-${match.id}-${team.id}`" :value="String(team.id)">
                      {{ team.name }}
                    </option>
                  </select>
                  <span v-else class="team-name">{{ displayTeamName(match, 'A') }}</span>
                  <button
                    v-if="canReportWinner(match, match.team_a_id) && !isEditingMatchup(match.id)"
                    class="btn-secondary win-btn"
                    type="button"
                    :disabled="Boolean(ctx.reportingWinners[match.id])"
                    @click="ctx.reportMatchWinner(match.id, match.team_a_id)"
                  >Win</button>
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
                    <option value="">Choose team</option>
                    <option v-for="team in ctx.event.teams" :key="`t-b-${match.id}-${team.id}`" :value="String(team.id)">
                      {{ team.name }}
                    </option>
                  </select>
                  <span v-else class="team-name">{{ displayTeamName(match, 'B') }}</span>
                  <button
                    v-if="canReportWinner(match, match.team_b_id) && !isEditingMatchup(match.id)"
                    class="btn-secondary win-btn"
                    type="button"
                    :disabled="Boolean(ctx.reportingWinners[match.id])"
                    @click="ctx.reportMatchWinner(match.id, match.team_b_id)"
                  >Win</button>
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
                  {{ isEditingMatchup(match.id) ? 'Close' : 'Edit matchup' }}
                </button>
                <button
                  v-if="isEditingMatchup(match.id)"
                  class="btn-secondary admin-btn"
                  type="button"
                  :disabled="Boolean(ctx.savingMatchups[match.id]) || Boolean(ctx.reportingWinners[match.id]) || Boolean(ctx.cancellingWinners[match.id])"
                  @click="saveMatchupAndClose(match.id)"
                >
                  {{ ctx.savingMatchups[match.id] ? 'Saving…' : 'Save' }}
                </button>
                <button
                  v-if="canCancelWinner(match) && !isEditingMatchup(match.id)"
                  class="btn-danger admin-btn"
                  type="button"
                  :disabled="Boolean(ctx.cancellingWinners[match.id]) || Boolean(ctx.reportingWinners[match.id])"
                  @click="ctx.cancelMatchWinner(match.id)"
                >
                  {{ ctx.cancellingWinners[match.id] ? 'Cancelling…' : 'Cancel Result' }}
                </button>
              </div>
            </article>
          </div>
        </section>
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
  --connector-stroke: 2px;
  --connector-overlap: 2px;
  --connector-ink: color-mix(in srgb, var(--line) 80%, transparent 20%);
  position: relative;
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
  --connector-ink: color-mix(in srgb, var(--brand-1) 60%, var(--line) 40%);
  border-color: color-mix(in srgb, var(--brand-1) 35%, var(--line) 65%);
}

.bracket-match.is-completed {
  --connector-ink: color-mix(in srgb, #1da56f 60%, var(--line) 40%);
  border-color: color-mix(in srgb, #17a36b 40%, var(--line) 60%);
}

.bracket-match.is-placeholder {
  --connector-ink: color-mix(in srgb, var(--line) 60%, transparent 40%);
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

.match-title-btn {
  border: 0;
  background: none;
  padding: 0;
  text-align: left;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--ink-2);
  cursor: pointer;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  transition: color 0.15s;
}

.match-title-btn:hover {
  color: var(--ink-1);
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

/* ── Bracket connectors ───────────────────────── */
.child-outgoing-link {
  position: absolute;
  right: calc((var(--col-gap) / -2) - var(--connector-overlap));
  top: calc(50% - (var(--connector-stroke) / 2));
  width: calc((var(--col-gap) / 2) + var(--connector-overlap));
  height: var(--connector-stroke);
  border-radius: var(--radius-pill);
  background: var(--connector-ink);
  pointer-events: none;
}

.parent-incoming-link {
  position: absolute;
  --fork-center-x: calc((100% - var(--connector-stroke)) / 2);
  left: calc((var(--col-gap) / -2) - var(--connector-overlap));
  top: 50%;
  width: calc((var(--col-gap) / 2) + var(--connector-overlap));
  height: var(--child-center-step, calc(var(--card-min-height) + 8px));
  transform: translateY(-50%);
  pointer-events: none;
}

.fork-segment {
  position: absolute;
  background: var(--connector-ink);
  border-radius: var(--radius-pill);
}

.fork-spine {
  left: var(--fork-center-x);
  top: 0;
  width: var(--connector-stroke);
  height: 100%;
}

.fork-arm-top,
.fork-arm-bottom {
  left: 0;
  width: calc(var(--fork-center-x) + var(--connector-stroke));
  height: var(--connector-stroke);
}

.fork-arm-top {
  top: calc(0px - (var(--connector-stroke) / 2));
}

.fork-arm-bottom {
  bottom: calc(0px - (var(--connector-stroke) / 2));
}

.fork-arm-right {
  left: var(--fork-center-x);
  width: calc(100% - var(--fork-center-x) + var(--connector-overlap));
  top: calc(50% - (var(--connector-stroke) / 2));
  height: var(--connector-stroke);
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
    --col-gap: 12px;
  }

  .bracket-round-list {
    gap: 0.5rem;
    min-height: auto;
  }

  .parent-incoming-link,
  .child-outgoing-link {
    display: none;
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
