<script setup>
import { computed, inject } from 'vue'
import overwatchLogo from '../../assets/branding/overwatch-logo-gold.png'
import { formatEventStartDate } from '../../lib/dates'

const ctx = inject('eventCtx')

const rosterCount = computed(() => ctx.event?.players.length || 0)
const teamCount = computed(() => ctx.event?.teams.length || 0)
const matchCount = computed(() => ctx.event?.matches.length || 0)
const formattedStartDate = computed(() => formatEventStartDate(ctx.event?.start_date))

const assignedCount = computed(() => {
  if (!ctx.event) {
    return 0
  }

  return ctx.event.players.filter((player) => Boolean(player.team_id)).length
})

const unassignedCount = computed(() => Math.max(0, rosterCount.value - assignedCount.value))

const latestMatches = computed(() => {
  if (!ctx.event) {
    return []
  }

  return [...ctx.event.matches].slice(0, 3)
})

const largestTeams = computed(() => {
  if (!ctx.event) {
    return []
  }

  return [...ctx.event.teams]
    .sort((a, b) => b.player_ids.length - a.player_ids.length)
    .slice(0, 3)
})

function matchupLabel(match) {
  if (!match.team_a_name || !match.team_b_name) {
    return 'Matchup not set'
  }

  return `${match.team_a_name} vs ${match.team_b_name}`
}
</script>

<template>
  <section>
    <h3 class="overview-title">
      <img class="overwatch-logo" :src="overwatchLogo" alt="Overwatch logo" />
      <span>Overview</span>
    </h3>
    <p v-if="formattedStartDate" class="muted">{{ formattedStartDate }}</p>
    <p v-if="ctx.event.description" class="overview-description muted">{{ ctx.event.description }}</p>
    <div class="overview-kpis">
      <article class="overview-kpi">
        <p class="overview-kpi-label">Roster</p>
        <p class="overview-kpi-value">{{ rosterCount }}/{{ ctx.event.max_players }}</p>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Teams</p>
        <p class="overview-kpi-value">{{ teamCount }}</p>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Matches</p>
        <p class="overview-kpi-value">{{ matchCount }}</p>
      </article>
    </div>

    <div class="overview-grid">
      <article class="overview-card">
        <h4>Players</h4>
        <p class="muted">{{ assignedCount }} assigned to teams · {{ unassignedCount }} unassigned</p>
        <button class="btn-secondary" @click="ctx.openSection('roster')">Open players</button>
      </article>

      <article class="overview-card">
        <h4>Teams</h4>
        <p v-if="largestTeams.length === 0" class="muted">No teams yet.</p>
        <ul v-else class="overview-list">
          <li v-for="team in largestTeams" :key="team.id">
            <span>{{ team.name }}</span>
            <span class="muted">{{ team.player_ids.length }} players</span>
          </li>
        </ul>
        <button class="btn-secondary" @click="ctx.openSection('teams')">Open teams</button>
      </article>

      <article class="overview-card">
        <h4>Latest Matches</h4>
        <p v-if="latestMatches.length === 0" class="muted">No matches created yet.</p>
        <ul v-else class="overview-list">
          <li v-for="match in latestMatches" :key="match.id">
            <span>{{ match.title }}</span>
            <span class="muted">{{ matchupLabel(match) }}</span>
          </li>
        </ul>
        <button class="btn-secondary" @click="ctx.openSection('matches')">Open matches</button>
      </article>
    </div>
  </section>
</template>

<style scoped>
.overview-kpis {
  display: grid;
  gap: 0.5rem;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 0.7rem;
}

.overview-title {
  margin: 0 0 0.3rem;
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.overwatch-logo {
  width: 20px;
  height: 20px;
  object-fit: contain;
  flex: 0 0 auto;
}

.overview-description {
  margin: 0 0 0.45rem;
  white-space: pre-wrap;
}

.overview-kpi {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  background: color-mix(in srgb, var(--card) 90%, #162134 10%);
  border-radius: 10px;
  padding: 0.5rem 0.58rem;
}

.overview-kpi-label {
  margin: 0;
  color: var(--ink-2);
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
}

.overview-kpi-value {
  margin: 0.15rem 0 0;
  color: var(--ink-1);
  font-size: 1.2rem;
  font-weight: 800;
}

.overview-grid {
  display: grid;
  gap: 0.55rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.overview-card {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  background: color-mix(in srgb, var(--card) 92%, #f0f6ff 8%);
  border-radius: 10px;
  padding: 0.55rem 0.62rem;
  display: flex;
  flex-direction: column;
  gap: 0.42rem;
}

.overview-card .btn-secondary {
  margin-top: auto;
}

.overview-card h4 {
  margin: 0 0 0.28rem;
}

.overview-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.35rem;
}

.overview-list li {
  display: flex;
  justify-content: space-between;
  gap: 0.6rem;
}

@media (max-width: 960px) {
  .overview-kpis {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .overview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
