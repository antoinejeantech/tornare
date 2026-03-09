<script setup>
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { apiCall } from '../lib/api'
import torbjornImage from '../assets/branding/torbjorn.webp'
import { formatEventStartDate } from '../lib/dates'
import EventListItem from '../components/events/EventListItem.vue'
import SpotlightEventCard from '../components/events/SpotlightEventCard.vue'

const events = ref([])
const loadingEvents = ref(false)

const sortedEvents = computed(() => {
  const next = [...events.value]
  next.sort((a, b) => {
    const aStart = normalizeDate(a?.start_date)
    const bStart = normalizeDate(b?.start_date)

    if (aStart === null && bStart === null) {
      return String(a?.name || '').localeCompare(String(b?.name || ''))
    }
    if (aStart === null) {
      return 1
    }
    if (bStart === null) {
      return -1
    }

    return aStart - bStart
  })
  return next
})

const featuredEvent = computed(() => {
  const manuallyFeatured = sortedEvents.value.find((event) => Boolean(event?.is_featured))
  if (manuallyFeatured) {
    return manuallyFeatured
  }

  const now = Date.now()
  const nextUpcoming = sortedEvents.value.find((event) => {
    const start = normalizeDate(event?.start_date)
    return start !== null && start >= now
  })

  return nextUpcoming || sortedEvents.value[0] || null
})

const latestEvents = computed(() => {
  if (!featuredEvent.value) {
    return sortedEvents.value.slice(0, 4)
  }

  return sortedEvents.value.filter((event) => event.id !== featuredEvent.value.id).slice(0, 4)
})

const totalEvents = computed(() => events.value.length)

const totalSignups = computed(() => {
  return events.value.reduce((sum, event) => {
    return sum + (Array.isArray(event?.players) ? event.players.length : 0)
  }, 0)
})

const upcomingThisWeek = computed(() => {
  const now = Date.now()
  const weekEnd = now + 7 * 24 * 60 * 60 * 1000

  return events.value.filter((event) => {
    const start = normalizeDate(event?.start_date)
    return start !== null && start >= now && start <= weekEnd
  }).length
})

const countdownEvents = computed(() => {
  const now = Date.now()
  return sortedEvents.value
    .filter((event) => {
      const start = normalizeDate(event?.start_date)
      return start !== null && start > now
    })
    .slice(0, 2)
})

const activityRows = computed(() => {
  return sortedEvents.value.slice(0, 6).map((event) => {
    const players = Array.isArray(event?.players) ? event.players.length : 0
    const maxPlayers = Number(event?.max_players) || 0

    return {
      id: event.id,
      date: formatShortDate(event?.start_date),
      name: event?.name || 'Untitled event',
      format: String(event?.format || '5v5'),
      players,
      maxPlayers,
      status: eventStatusForDashboard(event, players, maxPlayers),
    }
  })
})

function normalizeDate(value) {
  if (!value) {
    return null
  }

  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? null : parsed
}

function countdownLabel(startDate) {
  const start = normalizeDate(startDate)
  if (start === null) {
    return 'TBA'
  }

  const diff = Math.max(0, start - Date.now())
  const totalMinutes = Math.floor(diff / (1000 * 60))
  const days = Math.floor(totalMinutes / (60 * 24))
  const hours = Math.floor((totalMinutes % (60 * 24)) / 60)

  if (days > 0) {
    return `${days}d ${hours}h`
  }

  return `${hours}h`
}

function formatShortDate(value) {
  const normalized = normalizeDate(value)
  if (normalized === null) {
    return '--'
  }

  return new Date(normalized).toLocaleDateString([], {
    month: 'short',
    day: '2-digit',
  })
}

function eventStatusForDashboard(event, players, maxPlayers) {
  if (maxPlayers > 0 && players >= maxPlayers) {
    return 'Full'
  }

  const startAt = normalizeDate(event?.start_date)
  if (startAt !== null && startAt <= Date.now()) {
    return 'Progress'
  }

  return 'Open'
}

function activityStatusClass(status) {
  if (status === 'Full') {
    return 'is-full'
  }
  if (status === 'Progress') {
    return 'is-progress'
  }

  return 'is-open'
}

function activityPlayersFill(players, maxPlayers) {
  const max = Math.max(1, Number(maxPlayers) || 1)
  const ratio = Math.max(0, Math.min(1, players / max))
  return {
    width: `${Math.round(ratio * 100)}%`,
  }
}

async function loadLatestEvents() {
  loadingEvents.value = true
  try {
    const loadedEvents = await apiCall('/api/events')
    events.value = Array.isArray(loadedEvents) ? loadedEvents : []
  } catch {
    events.value = []
  } finally {
    loadingEvents.value = false
  }
}

onMounted(loadLatestEvents)
</script>

<template>
  <main class="app-shell home-shell">
    <section class="home-hero card">
      <img class="home-hero-art" :src="torbjornImage" alt="Torbjorn hero art" />
      <p class="home-eyebrow">Community Match Ops</p>
      <h1 class="home-title">Build Match Night Momentum, Not Admin Debt.</h1>
      <p class="home-subtitle muted">
        Tornare is your operations cockpit for Overwatch communities.
        Launch events quickly, keep signups visible, and move from planning to lobby with less friction.
      </p>
      <div class="home-hero-kpis">
        <span class="home-kpi-pill">{{ totalEvents }} live events</span>
        <span class="home-kpi-pill">{{ totalSignups }} registered players</span>
        <span class="home-kpi-pill">{{ upcomingThisWeek }} starting this week</span>
      </div>
      <div class="home-hero-actions">
        <RouterLink class="home-cta home-cta-link home-cta-link-primary" to="/events">Open Event Hub</RouterLink>
        <RouterLink class="home-cta home-cta-link" to="/news">Updates</RouterLink>
      </div>
    </section>

    <div class="home-section-head">
      <h2 class="home-section-title">Dashboard Overview</h2>
    </div>

    <section class="home-dashboard-grid">
      <section class="home-ticker card reveal-block reveal-1">
        <div class="home-ticker-head">
          <h2>Live Activity</h2>
          <span class="home-ticker-dot" aria-hidden="true"></span>
        </div>
        <p v-if="activityRows.length === 0" class="muted">No activity yet. Create an event to kick things off.</p>
        <div v-else class="home-activity-table-wrap">
          <div class="home-activity-table-head">
            <span>Date</span>
            <span>Event</span>
            <span>Format</span>
            <span>Players</span>
            <span>Status</span>
          </div>
          <div class="home-activity-table-body">
            <article v-for="row in activityRows" :key="`activity-${row.id}`" class="home-activity-row">
              <span class="home-activity-time">{{ row.date }}</span>
              <span class="home-activity-event" :title="row.name">{{ row.name }}</span>
              <span class="home-activity-format">{{ row.format }}</span>
              <span class="home-activity-players">
                <span class="home-activity-players-bar" aria-hidden="true">
                  <span class="home-activity-players-fill" :style="activityPlayersFill(row.players, row.maxPlayers)"></span>
                </span>
                <span class="home-activity-players-value">{{ row.players }}</span>
              </span>
              <span class="home-activity-status" :class="activityStatusClass(row.status)">{{ row.status }}</span>
            </article>
          </div>
        </div>
      </section>

      <aside class="home-dashboard-side reveal-block reveal-2">
        <section class="home-signal-grid">
          <article class="home-signal card">
            <div class="home-signal-head">
              <span class="home-signal-label">Board</span>
            </div>
            <strong class="home-signal-value">{{ totalEvents }}</strong>
            <p class="muted">Current event listings available to your community.</p>
          </article>
          <article class="home-signal card">
            <div class="home-signal-head">
              <span class="home-signal-label">Signups</span>
            </div>
            <strong class="home-signal-value">{{ totalSignups }}</strong>
            <p class="muted">Total players currently committed across events.</p>
          </article>
          <article class="home-signal card">
            <div class="home-signal-head">
              <span class="home-signal-label">7-Day Pulse</span>
            </div>
            <strong class="home-signal-value">{{ upcomingThisWeek }}</strong>
            <p class="muted">Events kicking off within the next week.</p>
          </article>
        </section>

        <section class="home-countdown-grid">
          <article v-for="event in countdownEvents" :key="`countdown-${event.id}`" class="home-countdown card">
            <span class="home-countdown-label">Next Event</span>
              <strong class="home-countdown-value">{{ countdownLabel(event.start_date) }}</strong>
              <h3 class="home-countdown-title">{{ event.name }}</h3>
              <p class="muted">{{ formatEventStartDate(event.start_date) || 'No date set' }}</p>
              <RouterLink class="home-inline-link" :to="{ name: 'event', params: { id: event.id } }">Open event</RouterLink>
          </article>
          <article v-if="countdownEvents.length === 0" class="home-countdown card home-countdown-empty">
            <span class="home-countdown-label">Next Event</span>
              <strong class="home-countdown-value">-</strong>
              <p class="muted">No upcoming events.</p>
          </article>
        </section>
      </aside>
    </section>

    <SpotlightEventCard
      v-if="featuredEvent"
      class="reveal-block reveal-4"
      :event="featuredEvent"
      badge-label="Spotlight Event"
    />

    <section class="home-jump-grid reveal-block reveal-5" aria-label="Quick links">
      <RouterLink class="card home-jump-card" to="/events">
        <span class="material-symbols-rounded home-jump-icon" aria-hidden="true">event</span>
        <h2>Event Hub</h2>
        <p class="muted">Create events, configure formats, and manage signups from one operational view.</p>
        <span class="home-jump-link">Go To Events</span>
      </RouterLink>

      <RouterLink class="card home-jump-card" to="/news">
        <span class="material-symbols-rounded home-jump-icon" aria-hidden="true">campaign</span>
        <h2>Latest Updates</h2>
        <p class="muted">Broadcast patch notes, rule changes, and league news to everyone in one feed.</p>
        <span class="home-jump-link">Read News</span>
      </RouterLink>

      <RouterLink class="card home-jump-card" to="/about">
        <span class="material-symbols-rounded home-jump-icon" aria-hidden="true">groups</span>
        <h2>Project Story</h2>
        <p class="muted">See the roadmap and mission behind Tornare and where the platform is headed next.</p>
        <span class="home-jump-link">About Tornare</span>
      </RouterLink>
    </section>

    <section class="home-latest card reveal-block reveal-6">
      <div class="home-latest-head">
        <h2>Latest Events</h2>
        <RouterLink class="home-inline-link home-inline-link-action" to="/events">
          <span>View all events</span>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </RouterLink>
      </div>
      <p v-if="loadingEvents" class="muted">Loading events...</p>
      <p v-else-if="latestEvents.length === 0" class="muted">No additional events yet. Open Event Hub to create one.</p>
      <ul v-else class="home-latest-list">
        <EventListItem
          v-for="event in latestEvents"
          :key="event.id"
          :event="event"
          :to="{ name: 'event', params: { id: event.id } }"
        />
      </ul>
    </section>

    <section class="home-banner card reveal-block reveal-7">
      <h2>Command Center For Captains And Organizers</h2>
      <p class="muted">From signup links to team coordination, Tornare keeps your event lifecycle visible and actionable.</p>
    </section>
  </main>
</template>

<style scoped>
.home-shell {
  max-width: 1820px;
  width: min(96vw, 1820px);
  display: grid;
  gap: 0.82rem;
}

.home-hero {
  position: relative;
  overflow: hidden;
  padding: 1.35rem;
  padding-right: clamp(1.25rem, 20vw, 12rem);
  border-color: color-mix(in srgb, var(--brand-2) 38%, var(--line) 62%);
  background:
    radial-gradient(620px 220px at 85% 0%, rgba(255, 255, 255, 0.08) 0%, transparent 72%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 90%, #2a2a2a 10%) 0%, var(--card) 100%);
}

.home-hero-art {
  position: absolute;
  right: 0.2rem;
  bottom: 0;
  width: clamp(175px, 30vw, 330px);
  max-height: 95%;
  object-fit: contain;
  pointer-events: none;
  filter: drop-shadow(0 8px 18px rgba(6, 16, 36, 0.35));
  opacity: 0.95;
}

.home-ticker {
  display: flex;
  flex-direction: column;
  gap: 0.52rem;
  height: 100%;
}

.home-dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.55fr) minmax(0, 1fr);
  gap: 0.62rem;
  align-items: stretch;
}

.home-dashboard-side {
  display: grid;
  gap: 0.62rem;
}

.home-section-title {
  margin: 0;
  font-size: clamp(1.1rem, 0.9vw + 0.9rem, 1.4rem);
  letter-spacing: 0.01em;
  text-transform: uppercase;
}

.home-section-head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.7rem;
}

.home-ticker-head {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.home-ticker-head h2 {
  margin: 0;
}

.home-ticker-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--brand-1);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--brand-1) 20%, transparent);
}

.home-activity-table-wrap {
  flex: 1;
  overflow: hidden;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--line) 82%, var(--brand-1) 18%);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--card) 94%, #1a2438 6%) 0%, color-mix(in srgb, var(--card) 98%, #1a2438 2%) 100%);
  box-shadow: inset 0 1px 0 color-mix(in srgb, #ffffff 14%, transparent 86%);
}

.home-activity-table-head,
.home-activity-row {
  display: grid;
  grid-template-columns: 84px minmax(180px, 1fr) 68px 110px 78px;
  gap: 0.52rem;
  align-items: center;
}

.home-activity-table-head {
  padding: 0.56rem 0.72rem;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--ink-2);
  border-bottom: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 90%, #172236 10%);
}

.home-activity-table-body {
  display: grid;
}

.home-activity-row {
  padding: 0.52rem 0.7rem;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 92%, #101928 8%);
  transition: background 0.16s ease;
}

.home-activity-row:last-child {
  border-bottom: 0;
}

.home-activity-row:hover {
  background: color-mix(in srgb, var(--brand-2) 8%, var(--card) 92%);
}

.home-activity-time,
.home-activity-format,
.home-activity-players-value,
.home-activity-status {
  font-family: "Space Mono", ui-monospace, monospace;
  font-size: 0.78rem;
}

.home-activity-time,
.home-activity-format {
  color: var(--ink-2);
}

.home-activity-event {
  font-size: 0.86rem;
  font-weight: 700;
  color: var(--ink-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.home-activity-players {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.home-activity-players-bar {
  position: relative;
  width: 66px;
  height: 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--line) 85%, #0f1930 15%);
  overflow: hidden;
}

.home-activity-players-fill {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 999px;
  background: linear-gradient(90deg, color-mix(in srgb, var(--brand-2) 74%, #ffffff 26%), color-mix(in srgb, var(--accent) 66%, #ffffff 34%));
}

.home-activity-status {
  border-radius: 999px;
  padding: 0.1rem 0.34rem;
  text-align: center;
  font-size: 0.68rem;
  font-weight: 800;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  border: 1px solid transparent;
}

.home-activity-status.is-open {
  color: #9ce9b8;
  background: #123224;
  border-color: #2e7a4f;
}

.home-activity-status.is-full {
  color: #ffb9a2;
  background: #3c1b16;
  border-color: #8b4433;
}

.home-activity-status.is-progress {
  color: #c4dcff;
  background: #1c2f4b;
  border-color: #3f5f8d;
}

.home-eyebrow {
  margin: 0;
  color: var(--accent);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.78rem;
  font-weight: 700;
}

.home-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.home-title {
  margin: 0.45rem 0 0;
  font-size: clamp(1.8rem, 2vw + 1.1rem, 2.8rem);
  line-height: 1.05;
  letter-spacing: -0.01em;
  max-width: 24ch;
  color: color-mix(in srgb, var(--heading-ink) 88%, #fff 12%);
}

.home-subtitle {
  margin: 0.8rem 0 0;
  max-width: 76ch;
  line-height: 1.5;
}

.home-hero-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, max-content));
  gap: 0.6rem;
  margin-top: 1rem;
}

.home-hero-kpis {
  margin-top: 0.95rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.home-kpi-pill {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-1) 35%, var(--line) 65%);
  background: color-mix(in srgb, var(--accent) 20%, var(--meta-bg) 80%);
  color: var(--meta-ink);
  padding: 0.18rem 0.6rem;
  font-size: 0.74rem;
  font-family: "Avenir Next", "Segoe UI", "Helvetica Neue", sans-serif;
  font-weight: 700;
}

.home-signal-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
}

.home-signal {
  display: grid;
  gap: 0.25rem;
  min-height: 106px;
}

.home-signal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.home-signal p {
  margin: 0;
}

.home-signal-label {
  color: var(--ink-2);
  font-size: 0.74rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.home-signal-value {
  font-size: 1.55rem;
  line-height: 1;
}

.home-countdown-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem;
}

.home-countdown {
  display: grid;
  gap: 0.22rem;
  background: color-mix(in srgb, var(--card) 92%, #162134 8%);
  min-height: 128px;
}

.home-countdown-label {
  font-size: 0.72rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.home-countdown-value {
  font-size: 1.22rem;
  line-height: 1;
}

.home-countdown-title {
  margin: 0.08rem 0 0;
  font-size: 0.95rem;
}

.home-countdown p {
  margin: 0;
}

.home-countdown-empty {
  grid-column: 1 / -1;
}


.home-cta {
  text-decoration: none;
}

.home-cta-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--brand-1);
  font-size: 0.95rem;
  font-weight: 400;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.home-cta-link-primary {
  font-weight: 700;
}

.home-cta-link:hover {
  color: color-mix(in srgb, var(--brand-1) 82%, #fff 18%);
  text-decoration: underline;
}

.home-banner h2 {
  margin: 0;
}

.home-inline-link {
  color: var(--brand-1);
  font-weight: 800;
  text-decoration: none;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-size: 0.82rem;
}

.home-inline-link:hover {
  text-decoration: underline;
}

.home-inline-link-action {
  display: inline-flex;
  align-items: center;
  gap: 0.26rem;
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  text-transform: none;
}

.home-inline-link-action svg {
  width: 0.78rem;
  height: 0.78rem;
  transition: transform 180ms ease;
}

.home-inline-link-action:hover svg {
  transform: translateX(2px);
}

.home-banner {
  padding-top: 0.88rem;
  padding-bottom: 0.88rem;
  background: linear-gradient(145deg, color-mix(in srgb, var(--card) 90%, #edf4ff 10%), var(--card));
}

.home-latest {
  display: grid;
  gap: 0.5rem;
}

.home-latest-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 0.7rem;
}

.home-latest-head h2 {
  margin: 0;
}

.home-latest-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.62rem;
}

.home-latest-list :deep(.event-list-item) {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
}

.home-latest-list :deep(.event-list-main .muted) {
  line-height: 1.35;
}

.home-banner p {
  margin: 0.45rem 0 0;
}

.home-jump-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.62rem;
}

.home-jump-card {
  text-decoration: none;
  color: inherit;
  display: grid;
  align-content: start;
  gap: 0.32rem;
  min-height: 176px;
  border-color: color-mix(in srgb, var(--line) 86%, var(--brand-2) 14%);
  background:
    radial-gradient(380px 130px at 0% 0%, color-mix(in srgb, var(--brand-2) 18%, transparent 82%), transparent 68%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 93%, #19253a 7%), var(--card));
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
}

.home-jump-card h2,
.home-jump-card p {
  margin: 0;
}

.home-jump-icon {
  color: color-mix(in srgb, var(--brand-1) 84%, #fff 16%);
  font-size: 1rem;
}

.home-jump-link {
  margin-top: auto;
  color: color-mix(in srgb, var(--ink-2) 86%, var(--ink-1) 14%);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 0.7rem;
  font-weight: 700;
}

.home-jump-card:hover {
  border-color: color-mix(in srgb, var(--brand-2) 52%, var(--line) 48%);
  box-shadow:
    0 12px 26px rgba(8, 20, 41, 0.28),
    0 3px 10px rgba(8, 20, 41, 0.18);
  transform: translateY(-2px);
}

.home-jump-card:hover .home-jump-link {
  color: color-mix(in srgb, var(--brand-1) 80%, #fff 20%);
}

.reveal-block {
  opacity: 0;
  transform: translateY(10px);
  animation: reveal-rise 380ms ease-out forwards;
}

.reveal-1 { animation-delay: 60ms; }
.reveal-2 { animation-delay: 120ms; }
.reveal-3 { animation-delay: 180ms; }
.reveal-4 { animation-delay: 240ms; }
.reveal-5 { animation-delay: 300ms; }
.reveal-6 { animation-delay: 360ms; }
.reveal-7 { animation-delay: 420ms; }

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 980px) {
  .home-hero {
    padding-right: 1.25rem;
    padding-bottom: 7.4rem;
  }

  .home-hero-art {
    right: 50%;
    transform: translateX(50%);
    width: clamp(170px, 56vw, 300px);
    opacity: 0.9;
  }

  .home-signal-grid {
    grid-template-columns: 1fr;
  }

  .home-dashboard-grid {
    grid-template-columns: 1fr;
  }

  .home-countdown-grid {
    grid-template-columns: 1fr;
  }

  .home-jump-grid {
    grid-template-columns: 1fr;
  }

  .home-hero-actions {
    grid-template-columns: 1fr;
  }

  .home-section-head {
    flex-direction: column;
    align-items: flex-start;
  }

  .home-activity-table-head,
  .home-activity-row {
    grid-template-columns: 56px minmax(120px, 1fr) 58px 86px 68px;
    gap: 0.4rem;
  }
}
</style>
