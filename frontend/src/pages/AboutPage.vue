<script setup>
import { RouterLink } from 'vue-router'

const pillars = [
  {
    id: 'clarity',
    title: 'Operational Clarity',
    text: 'Every event phase should be visible: setup, signup, balancing, and final match execution.'
  },
  {
    id: 'speed',
    title: 'Fast Organizer Loops',
    text: 'Common actions should take seconds, not context switches across spreadsheets and chat threads.'
  },
  {
    id: 'community',
    title: 'Community-First Workflow',
    text: 'Tools should support captains, hosts, and players equally with transparent event data.'
  }
]

const roadmap = [
  {
    id: 'now',
    phase: 'Now',
    items: 'Event setup, profile system, role-aware team management, and polished list experiences.'
  },
  {
    id: 'next',
    phase: 'Next',
    items: 'OAuth identity linking, richer player analytics, and deeper organizer automation flows.'
  },
  {
    id: 'later',
    phase: 'Later',
    items: 'League-level tooling, historical performance dashboards, and expanded ecosystem integrations.'
  }
]
</script>

<template>
  <main class="app-shell about-shell">
    <header class="page-header">
      <h1 class="page-title">About Tornare</h1>
    </header>

    <section class="card about-hero reveal-block reveal-1">
      <p class="about-eyebrow">Platform Mission</p>
      <h2>Give Overwatch communities a modern operations cockpit for match nights.</h2>
      <p class="muted">
        Tornare started from a simple friction point: community organizers spending more time on admin than on the game itself.
        The product focuses on practical execution, from first signup to final match handoff.
      </p>
      <div class="about-hero-actions">
        <RouterLink class="btn-primary" to="/events">Open Event Hub</RouterLink>
        <RouterLink class="btn-secondary" to="/news">Read Product Updates</RouterLink>
      </div>
    </section>

    <section class="about-pillars reveal-block reveal-2">
      <article v-for="pillar in pillars" :key="pillar.id" class="card about-pillar">
        <h3>{{ pillar.title }}</h3>
        <p class="muted">{{ pillar.text }}</p>
      </article>
    </section>

    <section class="card reveal-block reveal-3">
      <h2>Roadmap Snapshot</h2>
      <div class="about-roadmap">
        <article v-for="item in roadmap" :key="item.id" class="about-roadmap-item">
          <span class="about-roadmap-phase">{{ item.phase }}</span>
          <p class="muted">{{ item.items }}</p>
        </article>
      </div>
    </section>

    <section class="card about-principles reveal-block reveal-4">
      <h2>Design Principles</h2>
      <ul class="about-list muted">
        <li>Prefer obvious workflows over hidden controls.</li>
        <li>Make ownership boundaries explicit for safe collaboration.</li>
        <li>Expose useful status signals early so organizers can act quickly.</li>
      </ul>
    </section>
  </main>
</template>

<style scoped>
.about-shell {
  max-width: 1820px;
  width: min(96vw, 1820px);
  display: grid;
  gap: 0.88rem;
}

.about-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.about-hero {
  display: grid;
  gap: 0.55rem;
  border-color: color-mix(in srgb, var(--brand-2) 34%, var(--line) 66%);
  background:
    radial-gradient(1000px 140px at 0% 0%, rgba(66, 133, 244, 0.16), transparent 60%),
    color-mix(in srgb, var(--card) 92%, #edf5ff 8%);
}

.about-hero h2,
.about-hero p {
  margin: 0;
}

.about-eyebrow {
  margin: 0;
  color: var(--accent);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-size: 0.78rem;
  font-weight: 700;
}

.about-hero-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.about-pillars {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.about-pillar {
  display: grid;
  gap: 0.35rem;
}

.about-pillar h3,
.about-pillar p {
  margin: 0;
}

.about-roadmap {
  display: grid;
  gap: 0.55rem;
  margin-top: 0.5rem;
}

.about-roadmap-item {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  border-radius: 10px;
  background: color-mix(in srgb, var(--card) 92%, #edf4ff 8%);
  padding: 0.62rem 0.7rem;
  display: grid;
  gap: 0.3rem;
}

.about-roadmap-item p {
  margin: 0;
}

.about-roadmap-phase {
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--brand-1);
}

.about-principles h2 {
  margin: 0 0 0.45rem;
}

.about-list {
  margin: 0;
  padding-left: 1rem;
  display: grid;
  gap: 0.32rem;
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

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 980px) {
  .about-pillars {
    grid-template-columns: 1fr;
  }
}
</style>
