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
      <p class="muted page-subtitle">Built for organizers who run serious community game nights.</p>
    </header>

    <section class="card about-hero reveal-block reveal-1">
      <p class="about-eyebrow">Platform Mission</p>
      <h2>Give Overwatch communities a modern operations cockpit for match nights.</h2>
      <p class="muted">
        Tornare started from a simple friction point: community organizers spending more time on admin than on the game itself.
        The product focuses on practical execution, from first signup to final match handoff.
      </p>
      <div class="about-hero-actions">
        <RouterLink class="about-cta about-cta-link about-cta-link-primary" to="/events">Open Event Hub</RouterLink>
        <RouterLink class="about-cta about-cta-link" to="/news">Updates</RouterLink>
      </div>
    </section>

    <section class="about-kpi-strip reveal-block reveal-2">
      <article class="card about-kpi">
        <span class="about-kpi-label">Focus</span>
        <strong>Organizer Efficiency</strong>
        <p class="muted">Reduce setup friction from signup to lobby handoff.</p>
      </article>
      <article class="card about-kpi">
        <span class="about-kpi-label">Approach</span>
        <strong>Visible Workflow</strong>
        <p class="muted">Clear states for roster, teams, and match progression.</p>
      </article>
      <article class="card about-kpi">
        <span class="about-kpi-label">Outcome</span>
        <strong>Better Match Nights</strong>
        <p class="muted">Less admin overhead, more energy on gameplay quality.</p>
      </article>
    </section>

    <section class="about-pillars reveal-block reveal-3">
      <article v-for="pillar in pillars" :key="pillar.id" class="card about-pillar">
        <h3>{{ pillar.title }}</h3>
        <p class="muted">{{ pillar.text }}</p>
      </article>
    </section>

    <section class="card reveal-block reveal-4">
      <h2>Roadmap Snapshot</h2>
      <div class="about-roadmap">
        <article v-for="item in roadmap" :key="item.id" class="about-roadmap-item">
          <span class="about-roadmap-phase">{{ item.phase }}</span>
          <p class="muted">{{ item.items }}</p>
        </article>
      </div>
    </section>

    <section class="card about-principles reveal-block reveal-5">
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

.page-subtitle {
  margin: 0;
  text-align: right;
}

.about-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.about-hero {
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 0.55rem;
  border-color: color-mix(in srgb, var(--brand-2) 34%, var(--line) 66%);
  background:
    radial-gradient(760px 210px at 92% 0%, rgba(255, 255, 255, 0.07), transparent 70%),
    radial-gradient(1000px 150px at 0% 0%, rgba(74, 109, 164, 0.18), transparent 60%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 90%, #1f2733 10%) 0%, var(--card) 100%);
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
  display: grid;
  grid-template-columns: repeat(2, minmax(0, max-content));
  gap: 0.6rem;
  margin-top: 1rem;
}

.about-cta {
  text-decoration: none;
}

.about-cta-link {
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

.about-cta-link-primary {
  font-weight: 700;
}

.about-cta-link:hover {
  color: color-mix(in srgb, var(--brand-1) 82%, #fff 18%);
  text-decoration: underline;
}

.about-kpi-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.about-kpi {
  display: grid;
  gap: 0.24rem;
  border-color: color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  background: color-mix(in srgb, var(--card) 93%, #1b2330 7%);
}

.about-kpi strong {
  color: var(--ink-1);
  font-size: 1.05rem;
}

.about-kpi p {
  margin: 0;
}

.about-kpi-label {
  color: var(--brand-1);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-size: 0.73rem;
  font-weight: 700;
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
.reveal-5 { animation-delay: 300ms; }

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 980px) {
  .page-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .page-subtitle {
    text-align: left;
  }

  .about-kpi-strip {
    grid-template-columns: 1fr;
  }

  .about-pillars {
    grid-template-columns: 1fr;
  }
}
</style>
