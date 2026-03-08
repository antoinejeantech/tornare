<script setup>
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { apiCall } from '../lib/api'
import torbjornImage from '../assets/branding/torbjorn.webp'
import { formatEventStartDate } from '../lib/dates'
import EventListItem from '../components/events/EventListItem.vue'

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

const featuredEventMeta = computed(() => {
  if (!featuredEvent.value) {
    return ''
  }

  const playerCount = Array.isArray(featuredEvent.value.players) ? featuredEvent.value.players.length : 0
  const startText = formatEventStartDate(featuredEvent.value.start_date)
  const parts = [
    String(featuredEvent.value.event_type || 'PUG'),
    String(featuredEvent.value.format || '5v5'),
    `${playerCount}/${Number(featuredEvent.value.max_players) || 0} players`,
  ]

  if (startText) {
    parts.push(startText)
  }

  return parts.join(' · ')
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

const activityItems = computed(() => {
  if (events.value.length === 0) {
    return []
  }

  return events.value.slice(0, 8).map((event, index) => {
    const playerCount = Array.isArray(event?.players) ? event.players.length : 0
    const startText = formatEventStartDate(event?.start_date) || 'No start date'
    const format = String(event?.format || '5v5')
    return `${index + 1}. ${event?.name || 'Untitled event'} - ${format} - ${playerCount} players - ${startText}`
  })
})

function countdownLabel(startDate) {
  const start = normalizeDate(startDate)
  if (start === null) {
    return 'TBA'
  }

  const diff = Math.max(0, start - Date.now())
  const totalMinutes = Math.floor(diff / (1000 * 60))
  const days = Math.floor(totalMinutes / (60 * 24))
  const hours = Math.floor((totalMinutes % (60 * 24)) / 60)
  const minutes = totalMinutes % 60

  if (days > 0) {
    return `${days}d ${hours}h`
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }

  return `${minutes}m`
}

function normalizeDate(value) {
  if (!value) {
    return null
  }

  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? null : parsed
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
        Tornare is your operations cockpit for Overwatch communities. Launch events fast, keep signups visible,
        and move from planning to lobby with less friction.
      </p>
      <div class="home-hero-kpis">
        <span class="home-kpi-pill">{{ totalEvents }} live events</span>
        <span class="home-kpi-pill">{{ totalSignups }} registered players</span>
        <span class="home-kpi-pill">{{ upcomingThisWeek }} starting this week</span>
      </div>
      <div class="home-hero-actions">
        <RouterLink class="btn-primary home-cta" to="/events">Open Event Hub</RouterLink>
        <RouterLink class="btn-secondary home-cta" to="/news">Latest News</RouterLink>
      </div>
    </section>

    <section class="home-ticker card reveal-block reveal-1">
      <div class="home-ticker-head">
        <h2>Live Activity</h2>
        <span class="home-ticker-dot" aria-hidden="true"></span>
      </div>
      <p v-if="activityItems.length === 0" class="muted">No activity yet. Create an event to kick things off.</p>
      <div v-else class="home-ticker-track-wrap">
        <div class="home-ticker-track">
          <span v-for="(item, index) in [...activityItems, ...activityItems]" :key="`ticker-${index}`" class="home-ticker-item">
            {{ item }}
          </span>
        </div>
      </div>
    </section>

    <section class="home-signal-grid reveal-block reveal-2">
      <article class="home-signal card">
        <span class="home-signal-label">Board</span>
        <strong class="home-signal-value">{{ totalEvents }}</strong>
        <p class="muted">Current event listings available to your community.</p>
      </article>
      <article class="home-signal card">
        <span class="home-signal-label">Signups</span>
        <strong class="home-signal-value">{{ totalSignups }}</strong>
        <p class="muted">Total players currently committed across events.</p>
      </article>
      <article class="home-signal card">
        <span class="home-signal-label">7-Day Pulse</span>
        <strong class="home-signal-value">{{ upcomingThisWeek }}</strong>
        <p class="muted">Events kicking off within the next week.</p>
      </article>
    </section>

    <section class="home-countdown-grid reveal-block reveal-3">
      <article v-for="event in countdownEvents" :key="`countdown-${event.id}`" class="home-countdown card">
        <span class="home-countdown-label">Next start in</span>
        <strong class="home-countdown-value">{{ countdownLabel(event.start_date) }}</strong>
        <h3 class="home-countdown-title">{{ event.name }}</h3>
        <p class="muted">{{ formatEventStartDate(event.start_date) || 'No date set' }}</p>
        <RouterLink class="home-inline-link" :to="{ name: 'event', params: { id: event.id } }">Open event</RouterLink>
      </article>
      <article v-if="countdownEvents.length === 0" class="home-countdown card home-countdown-empty">
        <span class="home-countdown-label">No upcoming starts</span>
        <strong class="home-countdown-value">-</strong>
        <p class="muted">Create an event with a start date to populate countdown cards.</p>
      </article>
    </section>

    <section v-if="featuredEvent" class="home-spotlight card reveal-block reveal-4">
      <div class="home-spotlight-head">
        <span class="home-spotlight-badge">Spotlight Event</span>
        <RouterLink class="home-inline-link" :to="{ name: 'event', params: { id: featuredEvent.id } }">Open</RouterLink>
      </div>
      <h2 class="home-spotlight-title">{{ featuredEvent.name }}</h2>
      <p class="muted">{{ featuredEventMeta }}</p>
    </section>

    <section class="home-grid reveal-block reveal-5">
      <article class="home-feature card">
        <span class="material-symbols-rounded home-feature-icon" aria-hidden="true">stadium</span>
        <h2>Event Hub</h2>
        <p class="muted">Create events, configure formats, and manage signups from one operational view.</p>
        <RouterLink class="home-inline-link" to="/events">Go to events</RouterLink>
      </article>
      <article class="home-feature card">
        <span class="material-symbols-rounded home-feature-icon" aria-hidden="true">campaign</span>
        <h2>Latest Updates</h2>
        <p class="muted">Broadcast patch notes, rule changes, and league news to everyone in one feed.</p>
        <RouterLink class="home-inline-link" to="/news">Read news</RouterLink>
      </article>
      <article class="home-feature card">
        <span class="material-symbols-rounded home-feature-icon" aria-hidden="true">groups</span>
        <h2>Project Story</h2>
        <p class="muted">See the roadmap and mission behind Tornare and where the platform is headed next.</p>
        <RouterLink class="home-inline-link" to="/about">About Tornare</RouterLink>
      </article>
    </section>

    <section class="home-latest card reveal-block reveal-6">
      <div class="home-latest-head">
        <h2>Latest Events</h2>
        <RouterLink class="home-inline-link" to="/events">View all events</RouterLink>
      </div>
      <p v-if="loadingEvents" class="muted">Loading events...</p>
      <p v-else-if="latestEvents.length === 0" class="muted">No additional events yet. Open Event Hub to create one.</p>
      <ul v-else class="home-latest-list">
        <EventListItem
          v-for="event in latestEvents"
          :key="event.id"
          :event="event"
          as="link"
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
  gap: 0.88rem;
}

.home-hero {
  position: relative;
  overflow: hidden;
  padding: 1.25rem;
  padding-right: clamp(1.25rem, 25vw, 15.5rem);
  border-color: color-mix(in srgb, var(--brand-2) 38%, var(--line) 62%);
  background:
    radial-gradient(600px 220px at 85% 0%, color-mix(in srgb, var(--brand-1) 22%, transparent) 0%, transparent 70%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 86%, #e7f1ff 14%) 0%, var(--card) 100%);
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
  display: grid;
  gap: 0.55rem;
  border-color: color-mix(in srgb, var(--brand-2) 30%, var(--line) 70%);
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

.home-ticker-track-wrap {
  overflow: hidden;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  background: color-mix(in srgb, var(--card) 92%, #eff5ff 8%);
}

.home-ticker-track {
  display: inline-flex;
  gap: 1.1rem;
  white-space: nowrap;
  padding: 0.52rem 0.75rem;
  animation: ticker-scroll 26s linear infinite;
}

.home-ticker-item {
  font-family: "Space Mono", ui-monospace, monospace;
  font-size: 0.78rem;
  color: var(--ink-2);
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
  margin: 0.35rem 0 0;
  font-size: clamp(1.8rem, 2vw + 1.1rem, 2.8rem);
  line-height: 1.05;
  letter-spacing: -0.01em;
  max-width: 20ch;
}

.home-subtitle {
  margin: 0.65rem 0 0;
  max-width: 68ch;
}

.home-hero-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, max-content));
  gap: 0.55rem;
  margin-top: 0.8rem;
}

.home-hero-kpis {
  margin-top: 0.75rem;
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
  font-family: "Space Mono", ui-monospace, monospace;
  font-weight: 700;
}

.home-signal-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.home-countdown-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.7rem;
}

.home-countdown {
  display: grid;
  gap: 0.25rem;
  border-color: color-mix(in srgb, var(--brand-1) 24%, var(--line) 76%);
  background: color-mix(in srgb, var(--card) 92%, #ecf4ff 8%);
}

.home-countdown-label {
  font-size: 0.75rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.home-countdown-value {
  font-size: 1.5rem;
  line-height: 1;
}

.home-countdown-title {
  margin: 0.1rem 0 0;
}

.home-countdown p {
  margin: 0;
}

.home-countdown-empty {
  grid-column: 1 / -1;
}

.home-signal {
  display: grid;
  gap: 0.25rem;
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

.home-spotlight {
  display: grid;
  gap: 0.35rem;
  border-color: color-mix(in srgb, var(--brand-2) 32%, var(--line) 68%);
  background:
    radial-gradient(1000px 90px at 0% 0%, rgba(66, 133, 244, 0.16), transparent 60%),
    color-mix(in srgb, var(--card) 92%, #eef5ff 8%);
}

.home-spotlight-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.home-spotlight-badge {
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--brand-1);
}

.home-spotlight-title {
  margin: 0;
}

.home-cta {
  text-decoration: none;
}

.home-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.home-feature {
  display: grid;
  gap: 0.35rem;
}

.home-feature-icon {
  font-size: 1.25rem;
  color: var(--brand-1);
}

.home-feature h2,
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

.home-banner {
  border-color: color-mix(in srgb, var(--brand-1) 28%, var(--line) 72%);
  background: linear-gradient(145deg, color-mix(in srgb, var(--card) 90%, #edf4ff 10%), var(--card));
}

.home-latest {
  display: grid;
  gap: 0.55rem;
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
  gap: 0.5rem;
}

.home-banner p {
  margin: 0.45rem 0 0;
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

@keyframes ticker-scroll {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(-50%);
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

  .home-grid {
    grid-template-columns: 1fr;
  }

  .home-signal-grid {
    grid-template-columns: 1fr;
  }

  .home-countdown-grid {
    grid-template-columns: 1fr;
  }

  .home-hero-actions {
    grid-template-columns: 1fr;
  }
}
</style>
