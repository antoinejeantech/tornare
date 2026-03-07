<script setup>
import { inject } from 'vue'
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
</script>

<template>
  <section>
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">sports_esports</span>
      <span>Matches and Matchups</span>
    </h3>
    <form v-if="ctx.canManageEvent" class="grid-form compact-form" @submit.prevent="ctx.createMatch">
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

    <p v-if="ctx.event.matches.length === 0" class="muted">No matches yet. Create your first match.</p>
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

@media (max-width: 900px) {
  .match-item {
    flex-direction: column;
    align-items: stretch;
  }

  .match-side-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
