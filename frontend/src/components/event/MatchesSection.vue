<script setup>
import { computed, inject } from 'vue'
import { useRouter } from 'vue-router'

const ctx = inject('eventCtx')
const router = useRouter()
const teamPalette = ['#f04f23', '#0f2f8c', '#00a3a3', '#7828c8', '#f7b801', '#2e7d4f']

function colorForTeamId(teamId) {
  if (!teamId) {
    return '#c8ccda'
  }

  const id = String(teamId)
  let hash = 0
  for (let index = 0; index < id.length; index += 1) {
    hash = (hash * 31 + id.charCodeAt(index)) >>> 0
  }

  return teamPalette[hash % teamPalette.length]
}

function matchStripeStyle(match) {
  return {
    '--team-a': colorForTeamId(match.team_a_id),
    '--team-b': colorForTeamId(match.team_b_id)
  }
}

function openMatch(matchId) {
  router.push({ name: 'match', params: { id: matchId } })
}

const isTourney = computed(() => Boolean(ctx.isTourneyEvent))

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

function roundListStyle(roundIndex) {
  const cardHeight = 132
  const baseGap = 8
  const gap = ((2 ** roundIndex) * (cardHeight + baseGap)) - cardHeight
  const childCenterStep = roundIndex > 0 ? ((2 ** (roundIndex - 1)) * (cardHeight + baseGap)) : 0
  return {
    '--round-gap': `${gap}px`,
    '--child-center-step': `${childCenterStep}px`,
  }
}

const bracketRounds = computed(() => {
  const matches = Array.isArray(ctx.event?.matches) ? ctx.event.matches : []
  const roundMatches = matches.filter((match) => Number.isInteger(match.round) && Number.isInteger(match.position))

  const seededTeamCount = Math.max(2, nextPowerOfTwo(Math.max(2, ctx.event?.teams?.length || 0)))
  const fallbackRounds = bracketRoundsCount(seededTeamCount)

  const maxRoundFromMatches = roundMatches.reduce((max, match) => Math.max(max, Number(match.round || 0)), 0)
  const totalRounds = Math.max(fallbackRounds, maxRoundFromMatches)
  const rounds = []

  for (let round = 1; round <= totalRounds; round += 1) {
    const fallbackSlots = Math.max(1, seededTeamCount >> round)
    const roundExisting = roundMatches.filter((match) => Number(match.round) === round)
    const maxExistingPos = roundExisting.reduce((max, match) => Math.max(max, Number(match.position || 0)), 0)
    const slots = Math.max(fallbackSlots, maxExistingPos)

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

    rounds.push({
      id: `round-${round}`,
      label: round === totalRounds ? 'Final' : `Round ${round}`,
      cards,
    })
  }

  return rounds
})
</script>

<template>
  <section>
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">sports_esports</span>
      <span>Matches and Matchups</span>
    </h3>
    <form
      v-if="ctx.canManageEvent && !isTourney"
      class="grid-form compact-form"
      @submit.prevent="ctx.createMatch"
    >
      <label>
        Match title
        <input v-model="ctx.newMatchTitle" placeholder="Match 1" />
      </label>
      <label>
        Map
        <input v-model="ctx.newMatchMap" placeholder="King's Row" />
      </label>
      <button type="submit" class="btn-primary" :disabled="!ctx.canCreateMatch || ctx.creatingMatch">
        {{ ctx.creatingMatch ? 'Creating...' : 'Create match' }}
      </button>
    </form>

    <div v-if="isTourney" class="tourney-toolbar">
      <button
        v-if="ctx.canManageEvent"
        class="btn-primary"
        :disabled="ctx.creatingMatch || ctx.event.matches.length > 0"
        @click="ctx.generateTourneyBracket"
      >
        {{ ctx.creatingMatch ? 'Generating...' : 'Generate Bracket' }}
      </button>
      <p class="muted">
        {{ ctx.event.matches.length > 0 ? 'Bracket generated. Report winners to advance teams.' : 'Bracket preview is shown below. Generate when teams are ready.' }}
      </p>
    </div>

    <div v-if="isTourney" class="tourney-bracket-wrap">
      <div
        class="tourney-bracket"
        :style="{ '--rounds': bracketRounds.length, '--first-round-cards': bracketRounds[0]?.cards?.length || 1 }"
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
                ready: match.status === 'READY',
                completed: match.status === 'COMPLETED',
                placeholder: match.isPlaceholder,
              }"
            >
              <span
                v-if="roundIndex < bracketRounds.length - 1"
                class="child-outgoing-link"
                aria-hidden="true"
              ></span>
              <span v-if="roundIndex > 0" class="parent-incoming-link" aria-hidden="true">
                <span class="fork-segment fork-spine"></span>
                <span class="fork-segment fork-arm-top"></span>
                <span class="fork-segment fork-arm-bottom"></span>
                <span class="fork-segment fork-arm-right"></span>
              </span>
              <button
                v-if="!match.isPlaceholder"
                class="bracket-match-title"
                type="button"
                @click="openMatch(match.id)"
              >
                {{ match.title }}
              </button>
              <p v-else class="bracket-match-title muted">{{ match.title }}</p>
              <div class="bracket-team-row" :class="{ winner: match.winner_team_id === match.team_a_id && match.team_a_id }">
                <span class="bracket-team-name">{{ displayTeamName(match, 'A') }}</span>
                <button
                  v-if="canReportWinner(match, match.team_a_id)"
                  class="btn-secondary bracket-win-btn"
                  :disabled="Boolean(ctx.reportingWinners[match.id])"
                  @click="ctx.reportMatchWinner(match.id, match.team_a_id)"
                >Win</button>
              </div>
              <div class="bracket-team-row" :class="{ winner: match.winner_team_id === match.team_b_id && match.team_b_id }">
                <span class="bracket-team-name">{{ displayTeamName(match, 'B') }}</span>
                <button
                  v-if="canReportWinner(match, match.team_b_id)"
                  class="btn-secondary bracket-win-btn"
                  :disabled="Boolean(ctx.reportingWinners[match.id])"
                  @click="ctx.reportMatchWinner(match.id, match.team_b_id)"
                >Win</button>
              </div>
              <p class="muted bracket-status">
                {{ match.winner_team_name ? `Winner: ${match.winner_team_name}` : `Status: ${match.status}` }}
              </p>
            </article>
          </div>
        </section>
      </div>
    </div>

    <p v-else-if="ctx.event.matches.length === 0" class="muted">No matches yet. Create your first match.</p>
    <ul v-else class="entry-list match-list-compact">
      <li
        v-for="match in ctx.event.matches"
        :key="match.id"
        class="match-item match-item-openable"
        :class="{ 'matchup-set': Boolean(match.team_a_id && match.team_b_id) }"
        :style="matchStripeStyle(match)"
        role="button"
        tabindex="0"
        @click="openMatch(match.id)"
        @keydown.enter.prevent="openMatch(match.id)"
        @keydown.space.prevent="openMatch(match.id)"
      >
        <div class="list-main">
          <span class="entry-title">{{ match.title }}</span>
          <span class="muted">{{ match.players.length }}/{{ match.max_players }} · {{ match.map }}</span>
          <div class="matchup-row" @click.stop>
            <select
              v-model="ctx.matchupSelections[match.id].teamAId"
              :disabled="!ctx.canManageEvent || Boolean(ctx.savingMatchups[match.id])"
            >
              <option value="">Choose team</option>
              <option v-for="team in ctx.event.teams" :key="`a-${team.id}`" :value="String(team.id)">
                {{ team.name }}
              </option>
            </select>
            <span class="muted">vs</span>
            <select
              v-model="ctx.matchupSelections[match.id].teamBId"
              :disabled="!ctx.canManageEvent || Boolean(ctx.savingMatchups[match.id])"
            >
              <option value="">Choose team</option>
              <option v-for="team in ctx.event.teams" :key="`b-${team.id}`" :value="String(team.id)">
                {{ team.name }}
              </option>
            </select>
            <button
              v-if="ctx.canManageEvent"
              class="btn-secondary icon-btn"
              :disabled="Boolean(ctx.savingMatchups[match.id])"
              :title="ctx.savingMatchups[match.id] ? 'Saving matchup' : 'Save matchup'"
              @click="ctx.saveMatchup(match.id)"
            >
              <span class="material-symbols-rounded" aria-hidden="true">
                {{ ctx.savingMatchups[match.id] ? 'hourglass_top' : 'save' }}
              </span>
              <span class="sr-only">{{ ctx.savingMatchups[match.id] ? 'Saving matchup' : 'Save matchup' }}</span>
            </button>
          </div>
        </div>
        <div class="match-side-actions" @click.stop>
          <button
            v-if="ctx.canManageEvent"
            class="btn-danger icon-btn"
            :disabled="ctx.deletingMatchId === match.id"
            :title="ctx.deletingMatchId === match.id ? 'Deleting match' : 'Delete match'"
            @click="ctx.deleteMatch(match.id)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ ctx.deletingMatchId === match.id ? 'hourglass_top' : 'delete' }}
            </span>
            <span class="sr-only">{{ ctx.deletingMatchId === match.id ? 'Deleting match' : 'Delete match' }}</span>
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.section-title {
  margin: 0 0 0.3rem;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}

.grid-form {
  display: grid;
  gap: 0.56rem;
  margin-bottom: 0.72rem;
}

.grid-form label {
  display: grid;
  gap: 0.28rem;
}

.entry-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.entry-list li {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.7rem;
}

.match-list-compact {
  max-height: 420px;
  overflow: auto;
  padding-right: 0.15rem;
}

.match-item {
  position: relative;
  align-items: flex-start;
  border-left: 4px solid var(--line);
}

.match-item-openable {
  cursor: pointer;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.match-item-openable:hover,
.match-item-openable:focus-visible {
  box-shadow:
    0 12px 24px rgba(16, 34, 72, 0.16),
    0 3px 9px rgba(16, 34, 72, 0.14);
  border-color: color-mix(in srgb, var(--brand-2) 40%, var(--line) 60%);
  outline: none;
}

.match-item.matchup-set {
  border-left-color: transparent;
}

.match-item.matchup-set::before {
  content: "";
  position: absolute;
  left: -1px;
  top: 0;
  bottom: 0;
  width: 6px;
  border-radius: 10px 0 0 10px;
  background: linear-gradient(180deg, var(--team-a), var(--team-b));
}

.list-main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.28rem;
}

.entry-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.matchup-row {
  display: flex;
  align-items: center;
  gap: 0.42rem;
  flex-wrap: wrap;
}

.match-side-actions {
  display: grid;
  gap: 0.32rem;
}

.tourney-toolbar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.65rem;
  flex-wrap: wrap;
}

.tourney-bracket-wrap {
  overflow-x: auto;
  padding-bottom: 0.2rem;
}

.tourney-bracket {
  --card-min-height: 132px;
  --base-round-gap: 0.52rem;
  --col-gap: 16px;
  --column-height: calc(
    (var(--first-round-cards) * var(--card-min-height)) +
    ((var(--first-round-cards) - 1) * var(--base-round-gap))
  );
  display: grid;
  grid-template-columns: repeat(var(--rounds), minmax(220px, 1fr));
  gap: var(--col-gap);
  min-width: max-content;
  align-items: stretch;
}

.bracket-round {
  display: grid;
  gap: 0.52rem;
}

.bracket-round-title {
  margin: 0;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-2);
}

.bracket-round-list {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: var(--round-gap, 0.52rem);
  min-height: var(--column-height);
  position: relative;
}

.bracket-match {
  --connector-stroke: 2px;
  --connector-overlap: 2px;
  --connector-ink: color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  position: relative;
  min-height: var(--card-min-height);
  height: var(--card-min-height);
  border: 1px solid color-mix(in srgb, var(--line) 85%, var(--brand-2) 15%);
  background: color-mix(in srgb, var(--card) 90%, #f2f6ff 10%);
  border-radius: 10px;
  padding: 0.46rem 0.5rem;
  display: grid;
  gap: 0.32rem;
  box-shadow: 0 1px 5px rgba(15, 39, 84, 0.08);
}

.child-outgoing-link {
  position: absolute;
  right: calc((var(--col-gap) / -2) - var(--connector-overlap));
  top: calc(50% - (var(--connector-stroke) / 2));
  width: calc((var(--col-gap) / 2) + var(--connector-overlap));
  height: var(--connector-stroke);
  border-radius: 999px;
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
  border-radius: 999px;
}

.fork-spine {
  left: var(--fork-center-x);
  top: calc(0px - var(--connector-overlap));
  width: var(--connector-stroke);
  height: calc(100% + (var(--connector-overlap) * 2));
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

.bracket-match.ready {
  --connector-ink: color-mix(in srgb, var(--brand-1) 64%, #9ec0ff 36%);
  border-color: color-mix(in srgb, var(--brand-1) 45%, var(--line) 55%);
}

.bracket-match.completed {
  --connector-ink: color-mix(in srgb, #1da56f 68%, #b6f0da 32%);
  border-color: color-mix(in srgb, #17a36b 52%, var(--line) 48%);
}

.bracket-match.placeholder {
  --connector-ink: color-mix(in srgb, var(--line) 94%, #d8deed 6%);
  opacity: 0.7;
  border-style: dashed;
}

.bracket-match-title {
  border: 0;
  background: none;
  text-align: left;
  padding: 0;
  font-weight: 800;
  color: var(--ink-1);
  cursor: pointer;
}

.bracket-team-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.42rem;
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--brand-2) 12%);
  border-radius: 8px;
  padding: 0.28rem 0.36rem;
  background: color-mix(in srgb, var(--card) 92%, #ebf2ff 8%);
}

.bracket-team-row.winner {
  border-color: color-mix(in srgb, #1d9d6f 58%, var(--line) 42%);
  background: color-mix(in srgb, #d7f5e8 42%, var(--card) 58%);
}

.bracket-team-name {
  font-size: 0.86rem;
  font-weight: 650;
}

.bracket-win-btn {
  min-width: 48px;
  padding: 0.2rem 0.42rem;
  font-size: 0.78rem;
}

.bracket-status {
  margin: 0.12rem 0 0;
  font-size: 0.78rem;
}

.bracket-round.round-first .bracket-match {
  gap: 0.22rem;
  padding-top: 0.4rem;
  padding-bottom: 0.4rem;
}

.bracket-round.round-first .bracket-team-row {
  padding: 0.2rem 0.34rem;
}

.bracket-round.round-first .bracket-status {
  margin-top: 0;
  font-size: 0.74rem;
  line-height: 1.1;
}

@media (max-width: 900px) {
  .match-item {
    flex-direction: column;
    align-items: stretch;
  }

  .match-side-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .tourney-bracket {
    grid-template-columns: repeat(var(--rounds), minmax(200px, 1fr));
    --col-gap: 12px;
  }

  .bracket-round-list {
    gap: 0.6rem;
    min-height: auto;
  }

  .bracket-round.has-connectors .bracket-round-list {
    padding-left: 0;
  }

  .parent-incoming-link,
  .child-outgoing-link {
    display: none;
  }

  .bracket-match {
    min-height: auto;
    height: auto;
  }
}
</style>
