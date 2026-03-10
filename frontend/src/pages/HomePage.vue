<script setup>
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { apiCall } from '../lib/api'
import torbjornImage from '../assets/branding/torbjorn.webp'
import { formatEventStartDate } from '../lib/dates'
import EventListItem from '../components/events/EventListItem.vue'
import SpotlightEventCard from '../components/events/SpotlightEventCard.vue'
import ActionCtaButton from '../components/ui/ActionCtaButton.vue'
import StatusPill from '../components/ui/StatusPill.vue'
import EventActionButton from '../components/ui/EventActionButton.vue'

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

const activityDisplayRows = computed(() => {
  const targetCount = 8
  const filledRows = activityRows.value.map((row) => ({
    ...row,
    placeholder: false,
  }))

  const placeholders = Array.from({ length: Math.max(0, targetCount - filledRows.length) }, (_, index) => ({
    id: `placeholder-${index}`,
    date: '--',
    name: '',
    format: '--',
    players: 0,
    maxPlayers: 0,
    status: '',
    placeholder: true,
  }))

  return [...filledRows, ...placeholders]
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
    <section class="home-hero">
      <img class="home-hero-art" :src="torbjornImage" alt="Torbjorn hero art" />
      <p class="home-eyebrow">Community Match Ops</p>
      <h1 class="home-title">Run <em>Match Nights</em>, Not Admin.</h1>
      <p class="home-subtitle muted">
        Tornare is your operations cockpit for Overwatch communities.
        Launch events quickly, keep signups visible, and move from planning to lobby with less friction.
      </p>
      <div class="home-hero-kpis">
        <span class="home-kpi-pill"><span class="home-kpi-value">{{ totalEvents }}</span> live events</span>
        <span class="home-kpi-pill"><span class="home-kpi-value">{{ totalSignups }}</span> registered players</span>
        <span class="home-kpi-pill"><span class="home-kpi-value">{{ upcomingThisWeek }}</span> starting this week</span>
      </div>
      <div class="home-hero-actions">
        <ActionCtaButton to="/events">Open Event Hub</ActionCtaButton>
        <EventActionButton
          to="/news"
          variant="muted"
          size="cta"
          :full-width="false"
          :with-top-spacing="false"
          class="home-hero-secondary-btn"
        >
          Latest Updates
        </EventActionButton>
      </div>
    </section>

    <div class="home-section-head">
      <div class="home-section-title-wrap">
        <p class="home-section-kicker">STATUS CENTER</p>
        <h2 class="home-section-title">Dashboard Overview</h2>
      </div>
    </div>

    <section class="home-dashboard-grid">
      <section class="home-ticker reveal-block reveal-1">
        <div class="home-ticker-head">
          <span class="material-symbols-rounded home-ticker-icon" aria-hidden="true">bolt</span>
          <h2>LIVE ACTIVITY</h2>
        </div>
        <p v-if="activityRows.length === 0" class="muted">No activity yet. Create an event to kick things off.</p>
        <div class="home-activity-table-wrap">
          <div class="home-activity-table-head">
            <span>Date</span>
            <span>Event</span>
            <span>Format</span>
            <span>Players</span>
            <span>Status</span>
          </div>
          <div class="home-activity-table-body">
            <article
              v-for="row in activityDisplayRows"
              :key="`activity-${row.id}`"
              :class="['home-activity-row', { 'is-empty': row.placeholder }]"
            >
              <span class="home-activity-time">{{ row.date }}</span>
              <span class="home-activity-event" :title="row.name">{{ row.placeholder ? '\u00A0' : row.name }}</span>
              <span class="home-activity-format">{{ row.format }}</span>
              <span class="home-activity-players">
                <span v-if="!row.placeholder" class="home-activity-players-bar" aria-hidden="true">
                  <span class="home-activity-players-fill" :class="{ 'is-full': row.maxPlayers > 0 && row.players >= row.maxPlayers }" :style="activityPlayersFill(row.players, row.maxPlayers)"></span>
                </span>
                <span class="home-activity-players-value">{{ row.placeholder ? '\u00A0' : row.players }}</span>
              </span>
              <StatusPill v-if="!row.placeholder" :status="row.status" />
            </article>
          </div>
        </div>
      </section>

      <aside class="home-dashboard-side reveal-block reveal-2">
        <section class="home-signal-grid">
          <article class="home-signal">
            <div class="home-signal-head">
              <span class="home-signal-label">Board</span>
              <span class="material-symbols-rounded home-signal-icon home-signal-icon-light" aria-hidden="true">calendar_month</span>
            </div>
            <strong class="home-signal-value">{{ totalEvents }}</strong>
            <p class="muted">Current event listings available to your community.</p>
          </article>
          <article class="home-signal">
            <div class="home-signal-head">
              <span class="home-signal-label">Signups</span>
              <span class="material-symbols-rounded home-signal-icon home-signal-icon-light" aria-hidden="true">group</span>
            </div>
            <strong class="home-signal-value">{{ totalSignups }}</strong>
            <p class="muted">Total players currently committed across events.</p>
          </article>
        </section>

        <section class="home-countdown-grid">
          <article v-for="(event, index) in countdownEvents" :key="`countdown-${event.id}`" :class="['home-countdown', { 'home-countdown-upnext': index === 0 }]">
            <span class="home-countdown-label">{{ index === 0 ? 'Up Next' : 'After That' }}<span v-if="index === 0" class="material-symbols-rounded home-countdown-icon" aria-hidden="true">schedule</span></span>
              <strong class="home-countdown-value">{{ countdownLabel(event.start_date) }}</strong>
              <h3 class="home-countdown-title">{{ event.name }}</h3>
              <p class="muted">{{ formatEventStartDate(event.start_date) || 'No date set' }}</p>
              <EventActionButton
                :to="{ name: 'event', params: { id: event.id } }"
                :variant="index === 0 ? 'solid' : 'muted'"
              >
                Open event
              </EventActionButton>
          </article>
          <article v-if="countdownEvents.length === 0" class="home-countdown home-countdown-empty">
            <span class="home-countdown-label">Up Next</span>
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
      <RouterLink class="home-jump-card" to="/events" data-watermark="trophy">
        <h2><span class="material-symbols-rounded home-jump-icon" aria-hidden="true">trophy</span>Event Hub</h2>
        <p class="muted">Create events, configure formats, and manage signups from one operational view.</p>
        <span class="home-jump-link">
          <span>Go To Events</span>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
      </RouterLink>

      <RouterLink class="home-jump-card" to="/news" data-watermark="campaign">
        <h2><span class="material-symbols-rounded home-jump-icon" aria-hidden="true">campaign</span>Latest Updates</h2>
        <p class="muted">Broadcast patch notes, rule changes, and league news to everyone in one feed.</p>
        <span class="home-jump-link">
          <span>Read News</span>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
      </RouterLink>

      <RouterLink class="home-jump-card" to="/about" data-watermark="description">
        <h2><span class="material-symbols-rounded home-jump-icon" aria-hidden="true">description</span>Project Story</h2>
        <p class="muted">See the roadmap and mission behind Tornare and where the platform is headed next.</p>
        <span class="home-jump-link">
          <span>About Tornare</span>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
      </RouterLink>
    </section>

    <section class="home-latest reveal-block reveal-6">
      <div class="home-latest-head">
        <div class="home-latest-title-wrap">
          <p class="home-latest-kicker">UPCOMING ROSTER</p>
          <h2>LATEST EVENTS</h2>
        </div>
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
      <div class="home-banner-copy">
        <h2>Command Center For Captains And Organizers</h2>
        <p class="muted">From signup links to team coordination, Tornare keeps your event lifecycle visible and actionable.</p>
      </div>
      <RouterLink class="home-banner-action" to="/about">
        <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
        <span>LEARN MORE</span>
      </RouterLink>
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

.home-shell :deep(.material-symbols-rounded) {
  color: color-mix(in srgb, var(--brand-1) 84%, white 16%);
}

.home-hero {
  position: relative;
  overflow: hidden;
  margin-inline: 0;
  display: grid;
  align-content: start;
  row-gap: var(--space-3);
  padding-top: var(--space-3);
  padding-bottom: var(--space-3);
  padding-left: 0;
  padding-right: clamp(1.25rem, 20vw, 12rem);
  background: transparent;
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
  gap: 0;
  height: 100%;
}

.home-dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.55fr) minmax(0, 1fr);
  gap: var(--space-2);
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

.home-section-title-wrap {
  display: grid;
  gap: 0.2rem;
}

.home-section-kicker {
  margin: 0;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--brand-1) 82%, white 18%);
}

.home-section-head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.7rem;
  width: 100%;
  padding-bottom: var(--space-2);
  margin-bottom: var(--space-2);
  border-bottom: 1px solid color-mix(in srgb, var(--line) 86%, transparent 14%);
}

.home-ticker-head {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.56rem 0.72rem 1.5rem 0.72rem;
  border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  border-bottom: 1px solid var(--line-strong);
  background: color-mix(in srgb, var(--grey-900) 58%, black 42%);
}

.home-ticker-head h2 {
  margin: 0;
}

.home-ticker-icon {
  color: var(--ink-muted);
  font-size: 1rem;
}

.home-activity-table-wrap {
  flex: 1;
  overflow: hidden;
  border-radius: 0 0 12px 12px;
  border: none;
  background: transparent;
  box-shadow: none;
}

.home-ticker-head + .home-activity-table-wrap {
  margin-top: 0;
}

.home-activity-table-head,
.home-activity-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 2.6fr) minmax(0, 1fr) minmax(0, 1.2fr) minmax(0, 0.4fr);
  gap: 0.52rem;
  align-items: center;
}

.home-activity-table-head {
  padding: 0.56rem 0.72rem;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--grey-300);
  border-bottom: 1px solid var(--line-strong);
  background: color-mix(in srgb, var(--grey-900) 66%, black 34%);
}

.home-activity-table-body {
  display: grid;
}

.home-activity-row {
  padding: 0.52rem 0.7rem;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 92%, var(--bg-1) 8%);
  color: var(--ink-muted);
  transition: background 0.16s ease;
}

.home-activity-row:last-child {
  border-bottom: 0;
}

.home-activity-row:hover {
  background: color-mix(in srgb, var(--brand-2) 8%, var(--card) 92%);
}

.home-activity-row.is-empty {
  opacity: 0.42;
}

.home-activity-row.is-empty:hover {
  background: transparent;
}

.home-activity-time,
.home-activity-format,
.home-activity-players-value {
  font-family: var(--font-body);
  font-size: 0.78rem;
}

.home-activity-time,
.home-activity-format {
  color: var(--ink-muted);
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
  background: color-mix(in srgb, var(--line) 85%, var(--bg-1) 15%);
  overflow: hidden;
}

.home-activity-players-fill {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: 999px;
  background: linear-gradient(90deg, color-mix(in srgb, var(--brand-2) 92%, var(--bg-1) 8%), color-mix(in srgb, var(--accent) 90%, var(--bg-1) 10%));
}

.home-activity-players-fill.is-full {
  background: linear-gradient(90deg, color-mix(in srgb, var(--danger-soft) 90%, var(--bg-1) 10%), color-mix(in srgb, var(--danger-bg) 88%, var(--bg-1) 12%));
}

.home-eyebrow {
  margin: 0;
  color: var(--accent);
  font-family: var(--font-body);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.78rem;
  font-weight: 700;
}

.home-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.home-title {
  margin: 0;
  font-size: clamp(1.8rem, 2vw + 1.1rem, 2.8rem);
  line-height: 1.05;
  letter-spacing: -0.01em;
  max-width: 24ch;
  color: color-mix(in srgb, var(--heading-ink) 88%, white 12%);
}

.home-subtitle {
  margin: 0;
  max-width: 76ch;
  line-height: 1.5;
}

.home-hero-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, max-content));
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.home-hero-kpis {
  margin-top: 0;
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.home-kpi-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  min-height: 2rem;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--line) 78%, var(--brand-1) 22%);
  background: color-mix(in srgb, var(--card-soft) 78%, var(--bg-1) 22%);
  color: var(--ink-muted);
  padding: 0.34rem 0.78rem;
  font-size: 0.8rem;
  font-family: var(--font-body);
  font-weight: 600;
}

.home-kpi-value {
  color: white;
  font-size: 1rem;
  font-weight: 700;
  line-height: 1;
}

.home-signal-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem;
}

.home-signal {
  display: grid;
  gap: 0.25rem;
  min-height: 106px;
  padding: 0.5rem var(--space-1);
  border-bottom: 0;
}

.home-signal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.35rem;
}

.home-signal-icon {
  font-size: 0.95rem;
}

.home-signal-icon-light {
  color: var(--ink-muted);
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
  grid-template-columns: 1fr;
  gap: 0.5rem;
}

.home-countdown {
  display: grid;
  gap: 0.22rem;
  min-height: 128px;
  padding: 0.5rem var(--space-1);
  border-bottom: 0;
}

.home-countdown-label {
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.28rem;
  font-size: 0.72rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.home-countdown-icon {
  font-size: 0.9rem;
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

.home-countdown-upnext {
  border-left: 4px solid color-mix(in srgb, var(--brand-1) 86%, white 14%);
  padding-left: calc(var(--space-1) + 0.2rem);
}

.home-countdown-upnext .home-countdown-label {
  width: 100%;
  justify-content: space-between;
  color: color-mix(in srgb, var(--brand-1) 86%, white 14%);
  margin-bottom: var(--space-1);
}

.home-countdown-upnext .home-countdown-icon {
  color: color-mix(in srgb, var(--brand-1) 86%, white 14%);
}

.home-countdown-upnext .home-countdown-value {
  font-size: 1.55rem;
}

.home-countdown:not(.home-countdown-upnext) {
  color: var(--ink-muted);
}

.home-countdown:not(.home-countdown-upnext) .home-countdown-label,
.home-countdown:not(.home-countdown-upnext) .home-countdown-value,
.home-countdown:not(.home-countdown-upnext) .home-countdown-title,
.home-countdown:not(.home-countdown-upnext) .home-countdown-icon,
.home-countdown:not(.home-countdown-upnext) .home-inline-link {
  color: var(--ink-muted);
}

.home-countdown:not(.home-countdown-upnext) .home-countdown-label {
  margin-bottom: var(--space-1);
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

.home-cta-link-secondary {
  color: var(--ink-muted);
}

.home-hero-secondary-btn {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--ink-muted);
  font-size: 0.95rem;
}

.home-cta-link:hover {
  color: var(--ink-2);
  text-decoration: underline;
}

.home-hero-secondary-btn:hover {
  color: var(--ink-2);
}

.home-banner h2 {
  margin: 0;
  color: color-mix(in srgb, var(--brand-1) 84%, white 16%);
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

.home-latest-head .home-inline-link-action {
  color: var(--ink-muted);
}

.home-inline-link-action svg {
  width: 0.78rem;
  height: 0.78rem;
  transition: transform 180ms ease;
}

.home-inline-link-action:hover svg {
  transform: translateX(2px);
}

.home-latest-head .home-inline-link-action:hover {
  color: var(--ink-2);
  text-decoration: none;
}

.home-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 1.4rem 1.6rem;
  background: linear-gradient(145deg, color-mix(in srgb, var(--brand-1) 28%, var(--card) 72%), color-mix(in srgb, #de8b1f 20%, var(--card-soft) 80%));
}

.home-banner-copy {
  display: grid;
  gap: 0.36rem;
}

.home-banner-action {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
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

.home-banner-action .material-symbols-rounded {
  font-size: 0.95rem;
  color: white;
}

.home-banner-action:hover {
  color: color-mix(in srgb, white 90%, var(--brand-1) 10%);
  border-color: color-mix(in srgb, var(--line-strong) 72%, white 28%);
  background: color-mix(in srgb, var(--grey-900) 68%, black 32%);
  text-decoration: none;
}

.home-latest {
  display: grid;
  gap: 0.5rem;
  padding: 0.8rem 0;
}

.home-shell :deep(.spotlight-event-card) {
  margin-block: var(--space-2);
}

.home-latest-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 0.7rem;
  padding-bottom: var(--space-2);
  margin-bottom: var(--space-2);
  border-bottom: 1px solid color-mix(in srgb, var(--line) 86%, transparent 14%);
}

.home-latest-title-wrap {
  display: grid;
  gap: 0.2rem;
}

.home-latest-kicker {
  margin: 0;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--brand-1) 82%, white 18%);
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
  margin: 0;
  font-size: 0.88rem;
  color: var(--ink-muted);
}

.home-jump-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.62rem;
}

.home-jump-card {
  text-decoration: none;
  color: inherit;
  position: relative;
  overflow: hidden;
  display: grid;
  align-content: start;
  gap: 0.68rem;
  min-height: 120px;
  padding: 1.25rem 1rem;
  border-bottom: 0;
  transition: border-color 0.18s ease, color 0.18s ease;
}

.home-jump-card::after {
  content: attr(data-watermark);
  position: absolute;
  top: 0.22rem;
  right: 0.38rem;
  font-family: 'Material Symbols Rounded';
  font-size: 4.4rem;
  font-weight: 400;
  line-height: 1;
  color: color-mix(in srgb, white 12%, transparent 88%);
  pointer-events: none;
}

.home-jump-card h2,
.home-jump-card p {
  margin: 0;
}

.home-jump-card h2 {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  margin-bottom: var(--space-1);
  color: color-mix(in srgb, var(--brand-1) 84%, white 16%);
}

.home-jump-card p {
  margin-bottom: var(--space-1);
}

.home-jump-icon {
  color: color-mix(in srgb, var(--brand-1) 84%, white 16%);
  font-size: 1.25rem;
}

.home-jump-link {
  display: inline-flex;
  align-items: center;
  gap: 0.26rem;
  margin-top: auto;
  color: white;
  font-family: var(--font-body);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 0.7rem;
  font-weight: 700;
}

.home-jump-link svg {
  width: 0.78rem;
  height: 0.78rem;
  transition: transform 180ms ease;
}

.home-jump-card:hover {
  border-color: color-mix(in srgb, var(--brand-2) 52%, var(--line) 48%);
}

.home-jump-card:hover .home-jump-link {
  color: color-mix(in srgb, white 90%, var(--brand-1) 10%);
}

.home-jump-card:hover .home-jump-link svg {
  transform: translateX(2px);
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
    padding-right: var(--space-2);
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

  .home-banner {
    flex-direction: column;
    align-items: flex-start;
  }

  .home-activity-table-head,
  .home-activity-row {
    grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.9fr) minmax(0, 0.9fr) minmax(0, 1.1fr) minmax(0, 0.42fr);
    gap: 0.4rem;
  }
}
</style>
