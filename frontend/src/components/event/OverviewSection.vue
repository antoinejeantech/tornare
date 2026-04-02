<script setup lang="ts">
import { computed, inject } from 'vue'
import { RouterLink } from 'vue-router'
import { getDateTimestamp } from '../../lib/dates'
import AppBadge from '../ui/AppBadge.vue'
import PlayerCard from '../player/PlayerCard.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import type { EventCtxType } from '../../composables/event/event-inject'

const ctx = inject<EventCtxType>('eventCtx')!

const rosterCount = computed(() => ctx.event?.players.length || 0)
const teamCount = computed(() => ctx.event?.teams.length || 0)
const matchCount = computed(() => ctx.event?.matches.length || 0)
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

const nextMatches = computed(() => {
  if (!ctx.event) {
    return []
  }

  const now = Date.now()
  const withDate = ctx.event.matches
    .filter((m) => m.start_date)
    .map((m) => ({ match: m, ts: getDateTimestamp(m.start_date) }))
    .filter((entry): entry is { match: typeof entry.match; ts: number } => entry.ts !== null)
    .sort((a, b) => {
      const aFuture = a.ts >= now
      const bFuture = b.ts >= now
      if (aFuture !== bFuture) return aFuture ? -1 : 1
      return aFuture ? a.ts - b.ts : b.ts - a.ts
    })
    .map((x) => x.match)
  const withoutDate = ctx.event.matches.filter((m) => getDateTimestamp(m.start_date) === null)
  return [...withDate, ...withoutDate]
})

function formatMatchDate(isoStr: string): string {
  if (!isoStr) return ''
  const d = new Date(isoStr)
  if (isNaN(d.getTime())) return ''
  return d.toLocaleString(undefined, {
    month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

const largestTeams = computed(() => {
  if (!ctx.event) {
    return []
  }

  return [...ctx.event.teams]
    .sort((a, b) => (b.player_ids?.length ?? 0) - (a.player_ids?.length ?? 0))
    .slice(0, 3)
})

const featuredPlayers = computed(() => {
  if (!ctx.event) {
    return []
  }

  return [...ctx.event.players].slice(0, 3)
})

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

function sectionRoute(section: string) {
  return {
    name: 'event',
    params: { id: String(ctx.event?.id || '') },
    query: { section },
  }
}
</script>

<template>
  <section class="overview-section">
    <EventSectionHeader icon="dashboard" title="Event Snapshot" />

    <header class="overview-hero">
      <div class="overview-meta-row">
        <AppBadge variant="chip" radius="pill">{{ ctx.event?.event_type }}</AppBadge>
        <AppBadge variant="chip" radius="pill">{{ ctx.event?.format }}</AppBadge>
        <RouterLink v-if="creatorProfileRoute" class="overview-creator-chip" :to="creatorProfileRoute">
          by {{ ctx.event?.creator_name || 'Unknown' }}
        </RouterLink>
        <span v-else class="overview-creator-chip is-static">by {{ ctx.event?.creator_name || 'Unknown' }}</span>
        <span class="overview-readiness" :aria-label="`Status ${readinessLabel()}`">
          <span class="overview-readiness-copy">
            <span class="overview-readiness-kicker">STATUS</span>
            <span class="overview-readiness-value">{{ readinessLabel() }}</span>
          </span>
          <span class="overview-readiness-dot" aria-hidden="true"></span>
        </span>
      </div>

      <p v-if="ctx.event?.description" class="overview-description muted">{{ ctx.event?.description }}</p>
    </header>

    <div class="overview-kpis">
      <article class="overview-kpi">
        <p class="overview-kpi-label">Roster</p>
        <p class="overview-kpi-value">{{ rosterCount }}/{{ ctx.event?.max_players }}</p>
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
        <span class="overview-kpi-track" aria-hidden="true"><span class="overview-kpi-fill" :style="{ width: `${assignmentPercent}%` }"></span></span>
      </article>
      <article class="overview-kpi">
        <p class="overview-kpi-label">Matches</p>
        <p class="overview-kpi-value">{{ matchCount }}</p>
        <p class="muted overview-kpi-meta">{{ ctx.isTourneyEvent ? 'Bracket operations' : 'PUG operations' }}</p>
      </article>
    </div>

    <div class="overview-grid">
      <article class="overview-card">
        <div class="overview-card-head">
          <h4>Players</h4>
          <span class="material-symbols-rounded overview-card-icon" aria-hidden="true">group</span>
        </div>
        <ul v-if="featuredPlayers.length > 0" class="overview-player-list">
          <li v-for="player in featuredPlayers" :key="player.id">
            <PlayerCard :player="player" :clickable="false" :show-socials="false" />
          </li>
        </ul>
        <p v-else class="muted">No players yet.</p>
        <p class="muted overview-card-meta">{{ assignedCount }} assigned to teams • {{ unassignedCount }} unassigned</p>
        <RouterLink class="overview-open-btn" :to="sectionRoute('roster')">
          <span>Open players</span>
          <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
        </RouterLink>
      </article>

      <article class="overview-card">
        <div class="overview-card-head">
          <h4>Teams</h4>
          <span class="material-symbols-rounded overview-card-icon" aria-hidden="true">verified_user</span>
        </div>
        <p v-if="largestTeams.length === 0" class="muted">No teams yet.</p>
        <ul v-else class="overview-team-list">
          <li v-for="(team, index) in largestTeams" :key="team.id" class="overview-team-row">
            <span class="overview-team-tag">T{{ index + 1 }}</span>
            <span class="overview-team-name">{{ team.name }}</span>
            <span class="overview-team-size">{{ team.player_ids?.length ?? 0 }} players</span>
          </li>
        </ul>
        <RouterLink class="overview-open-btn" :to="sectionRoute('teams')">
          <span>Open teams</span>
          <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
        </RouterLink>
      </article>

      <article class="overview-card">
        <div class="overview-card-head">
          <h4>Next Matches</h4>
          <span class="material-symbols-rounded overview-card-icon" aria-hidden="true">swords</span>
        </div>
        <div v-if="nextMatches.length === 0" class="overview-empty-state">
          <span class="overview-empty-icon-wrap" aria-hidden="true">
            <span class="material-symbols-rounded overview-empty-icon">schedule</span>
          </span>
          <p class="overview-empty-title">No matches created yet</p>
          <p class="muted overview-empty-copy">The bracket is currently being generated by the administrator. Check back soon.</p>
        </div>
        <ul v-else class="overview-list">
          <li v-for="(match, index) in nextMatches" :key="match.id" class="overview-match-row">
            <span class="overview-match-tag">M{{ index + 1 }}</span>
            <span class="overview-match-name">
              <template v-if="match.team_a_name && match.team_b_name">
                <span class="overview-match-team">{{ match.team_a_name }}</span>
                <span class="overview-match-vs" aria-hidden="true">vs</span>
                <span class="overview-match-team">{{ match.team_b_name }}</span>
              </template>
              <span v-else class="muted">Matchup not set</span>
            </span>
            <span v-if="match.start_date" class="overview-match-date">{{ formatMatchDate(match.start_date) }}</span>
          </li>
        </ul>
        <RouterLink class="overview-open-btn" :to="sectionRoute('matches')">
          <span>Open matches</span>
          <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
        </RouterLink>
      </article>
    </div>
  </section>
</template>

<style scoped>
.overview-kpis {
  display: grid;
  gap: 0.72rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin: 0 0 0.9rem;
}

.overview-hero {
  border: 1px solid color-mix(in srgb, var(--line-strong) 58%, var(--bg-0) 42%);
  border-radius: var(--radius-md);
  padding: 1.1rem 1.15rem;
  margin-bottom: 0.9rem;
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
  display: grid;
  gap: 0.3rem;
  overflow-x: clip;
}

.overview-hero-head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.55rem;
}

.overview-meta-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.32rem;
}

.overview-creator-chip {
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--bg-1) 14%);
  background: color-mix(in srgb, var(--card) 78%, var(--bg-1) 22%);
  color: color-mix(in srgb, white 94%, var(--ink-1) 6%);
  padding: 0.14rem 0.42rem;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  text-decoration: none;
}

a.overview-creator-chip:hover {
  color: color-mix(in srgb, var(--brand-1) 90%, #ffe08f 10%);
  text-decoration: none;
}

.overview-readiness {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line) 88%, var(--bg-0) 12%);
  background: color-mix(in srgb, var(--card) 76%, var(--bg-1) 24%);
  padding: 0.66rem 1rem 0.64rem;
  margin-left: auto;
  flex-shrink: 0;
}

.overview-readiness-copy {
  display: grid;
  justify-items: end;
  line-height: 1.05;
}

.overview-readiness-kicker {
  font-size: 0.58rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
}

.overview-readiness-value {
  margin-top: 0.04rem;
  font-size: 0.76rem;
  font-weight: 700;
  color: color-mix(in srgb, white 94%, var(--ink-1) 6%);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.overview-readiness-dot {
  width: 0.58rem;
  height: 0.58rem;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--brand-1) 74%, #ffd869 26%);
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--brand-1) 14%, transparent 86%),
    0 0 var(--radius-md) color-mix(in srgb, var(--brand-1) 30%, transparent 70%);
}

.overview-description {
  margin: 0;
  white-space: pre-wrap;
  color: color-mix(in srgb, var(--ink-2) 90%, white 10%);
}

.overview-kpi {
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: var(--radius-md);
  padding: 0.88rem 0.92rem;
  display: grid;
  gap: 0.24rem;
  box-shadow: none;
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
  font-size: 1.52rem;
  font-weight: 800;
}

.overview-kpi-meta {
  margin: 0;
  font-size: 0.78rem;
}

.overview-kpi-track {
  height: 5px;
  margin-top: 0.32rem;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--line) 85%, var(--bg-1) 15%);
  overflow: hidden;
}

.overview-kpi-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: color-mix(in srgb, var(--brand-1) 86%, #ffd869 14%);
}

.overview-grid {
  display: grid;
  gap: 0.72rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.overview-card {
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: var(--radius-md);
  padding: 0.96rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  box-shadow: none;
  min-width: 0;
  overflow-x: clip;
}

.overview-card-meta {
  margin: 0;
  text-align: center;
}

.overview-open-btn {
  margin-top: auto;
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.42rem;
  padding: 0.46rem 0.72rem;
  border: 1px solid color-mix(in srgb, var(--line-strong) 82%, white 18%);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--grey-900) 74%, black 26%);
  color: white;
  text-decoration: none;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-size: 0.74rem;
  font-weight: 700;
  white-space: nowrap;
}

.overview-open-btn .material-symbols-rounded {
  font-size: 0.95rem;
  color: white;
}

.overview-open-btn:hover {
  color: white;
  border-color: color-mix(in srgb, var(--line-strong) 72%, white 28%);
  background: color-mix(in srgb, var(--grey-900) 68%, black 32%);
  text-decoration: none;
}

.overview-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.overview-card h4 {
  margin: 0 0 0.28rem;
}

.overview-player-list,
.overview-team-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.45rem;
  min-width: 0;
}

.overview-team-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  padding: 0.16rem 0;
}

.overview-team-tag {
  min-width: 2.1rem;
  height: 2.1rem;
  border-radius: var(--radius-md);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.95rem;
  font-weight: 800;
  color: color-mix(in srgb, white 90%, var(--ink-1) 10%);
  background: color-mix(in srgb, var(--bg-1) 84%, var(--card) 16%);
}

.overview-team-name {
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--ink-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.overview-team-size {
  font-size: 0.88rem;
  color: var(--ink-2);
  white-space: nowrap;
}

.overview-empty-state {
  flex: 1;
  display: grid;
  justify-items: center;
  align-content: center;
  text-align: center;
  gap: 0.56rem;
  padding: 0.95rem 0.35rem;
}

.overview-empty-icon-wrap {
  width: 5.2rem;
  height: 5.2rem;
  border-radius: var(--radius-pill);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--bg-1) 84%, transparent 16%);
}

.overview-empty-icon {
  font-size: 2.25rem;
  color: var(--ink-muted);
}

.overview-empty-title {
  margin: 0;
  font-size: 1.04rem;
  font-weight: 700;
  color: var(--ink-1);
}

.overview-empty-copy {
  margin: 0;
  max-width: 27ch;
}

.overview-card-icon {
  font-size: 1.08rem;
  line-height: 1;
  color: color-mix(in srgb, var(--ink-2) 80%, var(--brand-1) 20%);
}

.overview-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.35rem;
  max-height: 13.5rem;
  overflow-y: auto;
}

.overview-match-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  padding: 0.16rem 0;
}

.overview-match-tag {
  min-width: 2.1rem;
  height: 2.1rem;
  border-radius: var(--radius-md);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.78rem;
  font-weight: 800;
  color: color-mix(in srgb, white 90%, var(--ink-1) 10%);
  background: color-mix(in srgb, var(--bg-1) 84%, var(--card) 16%);
}

.overview-match-name {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--ink-1);
  overflow: hidden;
  min-width: 0;
}

.overview-match-team {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.overview-match-vs {
  flex-shrink: 0;
  font-size: 0.64rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--brand-1) 80%, #ffd869 20%);
  padding: 0.1rem 0.3rem;
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--brand-1) 12%, transparent 88%);
}

.overview-match-date {
  font-size: 0.8rem;
  color: var(--ink-2);
  white-space: nowrap;
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
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
