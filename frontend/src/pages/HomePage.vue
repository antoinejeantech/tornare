<script setup>
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { apiCall } from '../lib/api'
import overwatchLogo from '../assets/ranks/overwatch-logo.png'

const events = ref([])
const loadingEvents = ref(false)

const latestEvents = computed(() => events.value.slice(0, 3))

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
  <main class="app-shell">
    <section class="home-hero card">
      <p class="home-eyebrow">Community Match Ops</p>
      <h1 class="home-title">Run cleaner Overwatch events from signup to final matchup.</h1>
      <p class="home-subtitle muted">
        Tornare gives you one command center for rosters, teams, and match setup. Build events fast, keep lineups organized,
        and move from planning to play without spreadsheet chaos.
      </p>
      <div class="home-hero-actions">
        <RouterLink class="btn-primary home-cta" to="/events">Open Event Hub</RouterLink>
        <RouterLink class="btn-secondary home-cta" to="/news">Latest News</RouterLink>
      </div>
    </section>

    <section class="home-grid">
      <article class="home-feature card">
        <h2>Event Hub</h2>
        <p class="muted">Create events, edit settings, and keep everything in one place.</p>
        <RouterLink class="home-inline-link" to="/events">Go to events</RouterLink>
      </article>
      <article class="home-feature card">
        <h2>Latest Updates</h2>
        <p class="muted">Track placeholder announcements and upcoming release notes.</p>
        <RouterLink class="home-inline-link" to="/news">Read news</RouterLink>
      </article>
      <article class="home-feature card">
        <h2>Project Story</h2>
        <p class="muted">Learn what Tornare is, where it's going, and what comes next.</p>
        <RouterLink class="home-inline-link" to="/about">About Tornare</RouterLink>
      </article>
    </section>

    <section class="home-latest card">
      <div class="home-latest-head">
        <h2>Latest Events</h2>
        <RouterLink class="home-inline-link" to="/events">View all events</RouterLink>
      </div>
      <p v-if="loadingEvents" class="muted">Loading events...</p>
      <p v-else-if="latestEvents.length === 0" class="muted">No events yet. Start by creating your first one in the Event Hub.</p>
      <ul v-else class="home-latest-list">
        <li v-for="event in latestEvents" :key="event.id" class="home-latest-item">
          <RouterLink class="home-latest-link" :to="{ name: 'event', params: { id: event.id } }">
            <span class="home-latest-title-wrap">
              <img class="overwatch-logo" :src="overwatchLogo" alt="Overwatch logo" />
              <span class="home-latest-title">{{ event.name }}</span>
            </span>
            <span class="muted">{{ event.event_type }} · {{ event.matches.length }} matches · {{ event.players.length }}/{{ event.max_players }} players</span>
          </RouterLink>
        </li>
      </ul>
    </section>

    <section class="home-banner card">
      <h2>Designed For Captains And Organizers</h2>
      <p class="muted">Set up matches quickly, balance teams, and keep your event flow visible to everyone.</p>
    </section>
  </main>
</template>

<style scoped>
.home-hero {
  position: relative;
  overflow: hidden;
  padding: 1.25rem;
  border-color: color-mix(in srgb, var(--brand-2) 38%, var(--line) 62%);
  background:
    radial-gradient(600px 220px at 85% 0%, color-mix(in srgb, var(--brand-1) 22%, transparent) 0%, transparent 70%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 86%, #e7f1ff 14%) 0%, var(--card) 100%);
}

.home-eyebrow {
  margin: 0;
  color: var(--accent);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.8rem;
  font-weight: 700;
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

.home-latest-item {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  border-radius: 10px;
  background: color-mix(in srgb, var(--card) 92%, #eef5ff 8%);
}

.home-latest-link {
  text-decoration: none;
  color: inherit;
  display: grid;
  gap: 0.2rem;
  padding: 0.58rem 0.68rem;
}

.home-latest-link:hover .home-latest-title {
  color: var(--brand-1);
}

.home-latest-title {
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.home-latest-title-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.overwatch-logo {
  width: 18px;
  height: 18px;
  object-fit: contain;
  flex: 0 0 auto;
}

.home-banner p {
  margin: 0.45rem 0 0;
}

@media (max-width: 980px) {
  .home-grid {
    grid-template-columns: 1fr;
  }

  .home-hero-actions {
    grid-template-columns: 1fr;
  }
}
</style>
