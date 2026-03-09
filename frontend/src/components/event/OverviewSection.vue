<script setup>
import { computed, inject } from 'vue'
import { RouterLink } from 'vue-router'
import { formatEventStartDate } from '../../lib/dates'

const ctx = inject('eventCtx')

const rosterCount = computed(() => ctx.event?.players.length || 0)
const teamCount = computed(() => ctx.event?.teams.length || 0)
const matchCount = computed(() => ctx.event?.matches.length || 0)
const formattedStartDate = computed(() => formatEventStartDate(ctx.event?.start_date))
const creatorProfileRoute = computed(() => {
  const creatorId = String(ctx.event?.creator_id || '').trim()
  if (!creatorId) {
    return null
  }

  return { name: 'profile', params: { id: creatorId } }
})

const assignedCount = computed(() => {
  if (!ctx.event) {
    return 0
  }

  return ctx.event.players.filter((player) => Boolean(player.team_id)).length
})

const unassignedCount = computed(() => Math.max(0, rosterCount.value - assignedCount.value))

const rosterFillPercent = computed(() => {
  const maxPlayers = Number(ctx.event?.max_players || 0)
  if (maxPlayers <= 0) {
    return 0
  }

  return Math.max(0, Math.min(100, Math.round((rosterCount.value / maxPlayers) * 100)))
})

const assignmentPercent = computed(() => {
  if (rosterCount.value <= 0) {
    return 0
  }

  return Math.max(0, Math.min(100, Math.round((assignedCount.value / rosterCount.value) * 100)))
})

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

function readinessLabel() {
  if (rosterCount.value === 0) {
    return 'Starting Setup'
  }
  if (teamCount.value === 0) {
    return 'Creating Teams'
  }
  if (matchCount.value === 0) {
    return 'Creating Matches'
  }

  return 'Operations Ready'
}
</script>

<template>
  <section>
    <header class="overview-hero">
      <div class="overview-hero-head">
        <h3 class="section-title">
          <span class="material-symbols-rounded section-title-icon" aria-hidden="true">insights</span>
          <span>Event Snapshot</span>
        </h3>
        <span class="overview-readiness">{{ readinessLabel() }}</span>
      </div>

      <div class="overview-meta-row">
        <span class="overview-chip">{{ ctx.event.event_type }}</span>
        <span class="overview-chip">{{ ctx.event.format }}</span>
        <span v-if="formattedStartDate" class="overview-chip">{{ formattedStartDate }}</span>
        <RouterLink v-if="creatorProfileRoute" class="overview-chip overview-chip-link" :to="creatorProfileRoute">
          by {{ ctx.event.creator_name || 'Unknown' }}
        </RouterLink>
        <span v-else class="overview-chip">by {{ ctx.event.creator_name || 'Unknown' }}</span>
      </div>

      <p v-if="ctx.event.description" class="overview-description muted">{{ ctx.event.description }}</p>
    </header>

    <div class="overview-kpis">
      <article class="overview-kpi">
        <p class="overview-kpi-label">Roster</p>
        <p class="overview-kpi-value">{{ rosterCount }}/{{ ctx.event.max_players }}</p>
        <p class="muted overview-kpi-meta">{{ rosterFillPercent }}% full</p>
        <span class="overview-kpi-track" aria-hidden="true"><span class="overview-kpi-fill" :style="{ width: `${rosterFillPercent}%` }"></span></span>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Assignment</p>
        <p class="overview-kpi-value">{{ assignedCount }}/{{ rosterCount }}</p>
        <p class="muted overview-kpi-meta">{{ assignmentPercent }}% assigned</p>
        <span class="overview-kpi-track" aria-hidden="true"><span class="overview-kpi-fill" :style="{ width: `${assignmentPercent}%` }"></span></span>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Teams</p>
        <p class="overview-kpi-value">{{ teamCount }}</p>
        <p class="muted overview-kpi-meta">{{ unassignedCount }} unassigned</p>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Matches</p>
        <p class="overview-kpi-value">{{ matchCount }}</p>
        <p class="muted overview-kpi-meta">{{ ctx.isTourneyEvent ? 'Bracket operations' : 'PUG operations' }}</p>
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
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-bottom: 0.7rem;
}

.section-title {
  margin: 0;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}

.overview-hero {
  border: 1px solid color-mix(in srgb, var(--line) 76%, var(--brand-2) 24%);
  border-radius: 12px;
  padding: 0.82rem;
  margin-bottom: 0.68rem;
  background:
    radial-gradient(560px 180px at 0% 0%, color-mix(in srgb, var(--brand-2) 24%, transparent 76%), transparent 72%),
    radial-gradient(420px 160px at 95% 0%, color-mix(in srgb, #8fb3ee 18%, transparent 82%), transparent 74%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 90%, #121b2a 10%), var(--card));
  display: grid;
  gap: 0.5rem;
}

.overview-hero-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.55rem;
}

.overview-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.38rem;
}

.overview-chip {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 80%, var(--brand-1) 20%);
  background: color-mix(in srgb, var(--accent) 18%, var(--meta-bg) 82%);
  color: var(--meta-ink);
  padding: 0.14rem 0.52rem;
  font-size: 0.73rem;
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-weight: 700;
}

.overview-chip-link {
  color: color-mix(in srgb, var(--brand-1) 72%, var(--meta-ink) 28%);
  text-decoration: none;
}

.overview-chip-link:hover {
  text-decoration: underline;
}

.overview-readiness {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-2) 44%, var(--line) 56%);
  background: color-mix(in srgb, var(--brand-2) 16%, var(--card) 84%);
  color: color-mix(in srgb, var(--ink-1) 92%, #fff 8%);
  padding: 0.16rem 0.56rem;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  font-weight: 800;
}

.overview-description {
  margin: 0;
  white-space: pre-wrap;
}

.overview-kpi {
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 89%, #132038 11%);
  border-radius: 10px;
  padding: 0.5rem 0.58rem;
  display: grid;
  gap: 0.14rem;
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
  font-size: 1.28rem;
  font-weight: 800;
}

.overview-kpi-meta {
  margin: 0;
  font-size: 0.78rem;
}

.overview-kpi-track {
  height: 5px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--line) 86%, #10203a 14%);
  overflow: hidden;
}

.overview-kpi-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, color-mix(in srgb, var(--brand-2) 76%, #fff 24%), color-mix(in srgb, var(--brand-1) 70%, #fff 30%));
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

  .overview-hero-head {
    flex-direction: column;
    align-items: flex-start;
  }

  .overview-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 620px) {
  .overview-kpis {
    grid-template-columns: 1fr;
  }
}
</style>
