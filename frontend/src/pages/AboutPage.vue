<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

const { t } = useI18n()

const pillars = computed(() => [
  {
    id: 'clarity',
    icon: 'grid_view',
    title: t('about.pillar1Title'),
    text: t('about.pillar1Text')
  },
  {
    id: 'speed',
    icon: 'bolt',
    title: t('about.pillar2Title'),
    text: t('about.pillar2Text')
  },
  {
    id: 'community',
    icon: 'groups',
    title: t('about.pillar3Title'),
    text: t('about.pillar3Text')
  }
])

const roadmap = computed(() => [
  {
    id: 'now',
    phase: t('about.roadmapNow'),
    tag: 'now',
    items: t('about.roadmapNowItems')
  },
  {
    id: 'next',
    phase: t('about.roadmapNext'),
    tag: 'next',
    items: t('about.roadmapNextItems')
  },
  {
    id: 'later',
    phase: t('about.roadmapLater'),
    tag: 'later',
    items: t('about.roadmapLaterItems')
  }
])
</script>

<template>
  <main class="about-page">

    <!-- ── Hero ───────────────────────────────────────────────── -->
    <section class="about-hero">
      <div class="about-hero-inner">
        <div class="about-hero-content reveal-1">
          <span class="about-badge">{{ t('about.badge') }}</span>
          <h1 class="about-headline">
            {{ t('about.headline1') }}<br>
            <span class="about-headline-accent">{{ t('about.headlineAccent') }}</span> {{ t('about.headline3') }}<br>
            {{ t('about.headline4') }}
          </h1>
          <p class="about-hero-sub">
            {{ t('about.heroSub') }}
          </p>
          <div class="about-hero-actions">
            <RouterLink class="btn btn-primary about-hero-btn" to="/events">
              {{ t('about.openHub') }}
              <span class="material-symbols-rounded" aria-hidden="true">arrow_outward</span>
            </RouterLink>
            <RouterLink class="btn btn-secondary about-hero-btn" to="/news">{{ t('about.updates') }}</RouterLink>
          </div>
        </div>

        <div class="about-hero-widget reveal-2">
          <div class="widget-header">
            <span class="widget-label">{{ t('about.liveMonitoring') }}</span>
            <span class="material-symbols-rounded widget-gear" aria-hidden="true">settings</span>
          </div>
          <div class="widget-bar-wrap">
            <div class="widget-bar"></div>
          </div>
          <div class="widget-stats">
            <div class="widget-stat">
              <span class="widget-stat-label">{{ t('about.signupRate') }}</span>
              <strong class="widget-stat-value">142/hr</strong>
            </div>
            <div class="widget-stat">
              <span class="widget-stat-label">{{ t('about.activeLobbies') }}</span>
              <strong class="widget-stat-value">24</strong>
            </div>
          </div>
          <p class="widget-quote">{{ t('about.quote') }}</p>
        </div>
      </div>
    </section>

    <!-- ── KPI strip ──────────────────────────────────────────── -->
    <section class="about-kpi-strip reveal-3">
      <article class="card about-kpi">
        <span class="about-kpi-label">{{ t('about.focusLabel') }}</span>
        <strong>{{ t('about.focusTitle') }}</strong>
        <p class="muted">{{ t('about.focusDesc') }}</p>
      </article>
      <article class="card about-kpi">
        <span class="about-kpi-label">{{ t('about.approachLabel') }}</span>
        <strong>{{ t('about.approachTitle') }}</strong>
        <p class="muted">{{ t('about.approachDesc') }}</p>
      </article>
      <article class="card about-kpi">
        <span class="about-kpi-label">{{ t('about.outcomeLabel') }}</span>
        <strong>{{ t('about.outcomeTitle') }}</strong>
        <p class="muted">{{ t('about.outcomeDesc') }}</p>
      </article>
    </section>

    <!-- ── Pillars ────────────────────────────────────────────── -->
    <section class="about-section reveal-4">
      <div class="about-section-head">
        <h2 class="about-section-title">{{ t('about.integrityTitle') }}</h2>
        <p class="muted">{{ t('about.integrityDesc') }}</p>
      </div>
      <div class="about-pillars">
        <article v-for="pillar in pillars" :key="pillar.id" class="card about-pillar">
          <span class="about-pillar-icon">
            <span class="material-symbols-rounded" aria-hidden="true">{{ pillar.icon }}</span>
          </span>
          <div>
            <strong class="about-pillar-title">{{ pillar.title }}</strong>
            <p class="muted">{{ pillar.text }}</p>
          </div>
        </article>
      </div>
    </section>

    <!-- ── Roadmap ────────────────────────────────────────────── -->
    <section class="about-roadmap-section reveal-5">
      <div class="card about-roadmap-left">
        <h2>{{ t('about.roadmapTitle') }}</h2>
        <p class="muted">{{ t('about.roadmapDesc') }}</p>
        <div class="about-roadmap-doc">
          <span class="material-symbols-rounded about-roadmap-doc-icon" aria-hidden="true">description</span>
          <div>
            <strong>{{ t('about.ecosystemStrategy') }}</strong>
            <span class="muted about-roadmap-doc-sub">{{ t('about.ecosystemSub') }}</span>
          </div>
        </div>
      </div>
      <div class="about-roadmap-right">
        <article v-for="item in roadmap" :key="item.id" class="about-roadmap-item">
          <span :class="['about-roadmap-tag', `about-roadmap-tag--${item.tag}`]">{{ item.phase }}</span>
          <p class="muted">{{ item.items }}</p>
        </article>
      </div>
    </section>

    <!-- ── Design Principles ──────────────────────────────────── -->
    <section class="about-section reveal-6">
      <div class="about-section-head">
        <h2 class="about-section-title">{{ t('about.principlesTitle') }}</h2>
      </div>
      <div class="about-principles-body">
        <ul class="about-principles-list">
          <li>{{ t('about.principle1') }}</li>
          <li>{{ t('about.principle2') }}</li>
          <li>{{ t('about.principle3') }}</li>
        </ul>
        <div class="card about-principles-quote">
          <span class="material-symbols-rounded about-principles-shield" aria-hidden="true">verified_user</span>
          <p>{{ t('about.principlesQuote') }}</p>
        </div>
      </div>
    </section>

  </main>
</template>

<style scoped>
/* ── Shell ─────────────────────────────────────────────────────── */
.about-page {
  display: flex;
  flex-direction: column;
  gap: 3.5rem;
  padding-bottom: 4rem;
  animation: rise-in 300ms ease-out;
}

@keyframes rise-in {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}

.reveal-1 { animation: rise-in 380ms ease-out 60ms both; }
.reveal-2 { animation: rise-in 380ms ease-out 130ms both; }
.reveal-3 { animation: rise-in 380ms ease-out 180ms both; }
.reveal-4 { animation: rise-in 380ms ease-out 230ms both; }
.reveal-5 { animation: rise-in 380ms ease-out 280ms both; }
.reveal-6 { animation: rise-in 380ms ease-out 330ms both; }

/* ── Hero ──────────────────────────────────────────────────────── */
.about-hero {
  background:
    radial-gradient(ellipse 80% 60% at 10% 0%, rgba(74, 109, 164, 0.14), transparent 65%),
    radial-gradient(ellipse 50% 40% at 90% 100%, rgba(240, 191, 76, 0.06), transparent 60%),
    var(--bg-0);
  border-bottom: 1px solid var(--line);
  padding: 4.5rem 1rem 3.5rem;
}

.about-hero-inner {
  max-width: 1020px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3rem;
  align-items: center;
}

.about-badge {
  display: inline-block;
  border: 1px solid color-mix(in srgb, var(--brand-1) 40%, var(--line) 60%);
  border-radius: var(--radius-pill);
  padding: 0.2rem 0.7rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--brand-1);
  background: color-mix(in srgb, var(--brand-1) 8%, transparent 92%);
  margin-bottom: 1.2rem;
}

.about-headline {
  font-family: var(--font-heading);
  font-size: clamp(2.6rem, 5vw, 4rem);
  font-weight: 900;
  line-height: 1.04;
  letter-spacing: -0.02em;
  text-transform: uppercase;
  color: var(--ink-1);
  margin: 0 0 1.1rem;
}

.about-headline-accent {
  color: var(--brand-1);
  font-style: italic;
}

.about-hero-sub {
  color: var(--ink-2);
  font-size: 1rem;
  line-height: 1.65;
  max-width: 38ch;
  margin: 0 0 1.8rem;
}

.about-hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.7rem;
}

.about-hero-btn {
  font-size: 0.88rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 0.6rem 1.2rem;
  border-radius: var(--radius-md);
  text-decoration: none;
}

/* Widget */
.about-hero-widget {
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--card) 80%, var(--bg-0) 20%);
  padding: 1.25rem 1.35rem;
  display: grid;
  gap: 0.9rem;
  box-shadow: 0 8px 24px rgba(2, 6, 14, 0.32);
}

.widget-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.widget-label {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.widget-gear {
  font-size: 1rem;
  color: var(--ink-3, #6b7280);
}

.widget-bar-wrap {
  height: 4px;
  border-radius: 2px;
  background: var(--line);
  overflow: hidden;
}

.widget-bar {
  height: 100%;
  width: 68%;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--brand-1), color-mix(in srgb, var(--brand-1) 60%, #fff 40%));
}

.widget-stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.6rem;
}

.widget-stat {
  display: flex;
  flex-direction: column;
  gap: 0.18rem;
}

.widget-stat-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-3, #6b7280);
}

.widget-stat-value {
  font-size: 1.55rem;
  font-weight: 800;
  font-family: var(--font-heading);
  color: var(--ink-1);
  line-height: 1;
}

.widget-quote {
  font-size: 0.78rem;
  color: var(--ink-2);
  line-height: 1.5;
  border-top: 1px solid var(--line);
  padding-top: 0.75rem;
  margin: 0;
  font-style: italic;
}

/* ── Shared section wrapper ────────────────────────────────────── */
.about-kpi-strip,
.about-section,
.about-roadmap-section,
.about-principles-body {
  max-width: 1020px;
  margin: 0 auto;
  width: 100%;
  padding: 0 1rem;
}

/* ── KPI strip ─────────────────────────────────────────────────── */
.about-kpi-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.85rem;
}

.about-kpi {
  display: grid;
  gap: 0.3rem;
}

.about-kpi strong {
  font-size: 1.08rem;
  color: var(--ink-1);
}

.about-kpi p {
  margin: 0;
  font-size: 0.88rem;
  line-height: 1.55;
}

.about-kpi-label {
  color: var(--brand-1);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.7rem;
  font-weight: 700;
}

/* ── Section head ──────────────────────────────────────────────── */
.about-section {
  display: grid;
  gap: 1.5rem;
}

.about-section-head {
  display: grid;
  gap: 0.5rem;
}

.about-section-title {
  font-family: var(--font-heading);
  font-size: 1.55rem;
  font-weight: 900;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--ink-1);
  margin: 0;
  padding-bottom: 0.55rem;
  border-bottom: 3px solid var(--brand-1);
  display: inline-block;
}

.about-section-head .muted {
  margin: 0;
  font-size: 0.95rem;
  line-height: 1.6;
  max-width: 58ch;
}

/* ── Pillars ───────────────────────────────────────────────────── */
.about-pillars {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.85rem;
}

.about-pillar {
  display: flex;
  gap: 0.9rem;
  align-items: flex-start;
}

.about-pillar-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--brand-1) 28%, var(--line) 72%);
  background: color-mix(in srgb, var(--brand-1) 10%, var(--card) 90%);
  color: var(--brand-1);
  flex-shrink: 0;
}

.about-pillar-icon .material-symbols-rounded {
  font-size: 1.2rem;
  font-variation-settings: 'FILL' 1;
}

.about-pillar-title {
  display: block;
  font-size: 1rem;
  color: var(--ink-1);
  margin-bottom: 0.28rem;
}

.about-pillar p {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.55;
}

/* ── Roadmap ───────────────────────────────────────────────────── */
.about-roadmap-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
  align-items: start;
}

.about-roadmap-left {
  display: grid;
  gap: 0.7rem;
}

.about-roadmap-left h2 {
  font-family: var(--font-heading);
  font-size: 1.45rem;
  font-weight: 800;
  color: var(--ink-1);
  margin: 0;
}

.about-roadmap-left .muted {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.6;
}

.about-roadmap-doc {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  padding: 0.65rem 0.8rem;
  background: color-mix(in srgb, var(--bg-1) 60%, var(--card) 40%);
  margin-top: 0.3rem;
}

.about-roadmap-doc-icon {
  font-size: 1.35rem;
  color: var(--brand-1);
  font-variation-settings: 'FILL' 1;
}

.about-roadmap-doc strong {
  display: block;
  font-size: 0.9rem;
  color: var(--ink-1);
}

.about-roadmap-doc-sub {
  font-size: 0.75rem;
}

.about-roadmap-right {
  display: grid;
  gap: 0.65rem;
}

.about-roadmap-item {
  display: flex;
  align-items: flex-start;
  gap: 0.85rem;
}

.about-roadmap-item .muted {
  font-size: 0.9rem;
  line-height: 1.55;
  margin: 0;
  padding-top: 0.12rem;
}

.about-roadmap-tag {
  flex-shrink: 0;
  font-size: 0.65rem;
  font-weight: 800;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 0.22rem 0.55rem;
  border-radius: var(--radius-pill);
  border: 1px solid;
  margin-top: 0.08rem;
}

.about-roadmap-tag--now {
  background: var(--brand-1);
  border-color: var(--brand-1);
  color: #101216;
}

.about-roadmap-tag--next {
  background: transparent;
  border-color: color-mix(in srgb, var(--brand-1) 40%, var(--line) 60%);
  color: var(--brand-1);
}

.about-roadmap-tag--later {
  background: transparent;
  border-color: var(--line);
  color: var(--ink-2);
}

/* ── Design Principles ─────────────────────────────────────────── */
.about-principles-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
  align-items: center;
}

.about-principles-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 0.85rem;
}

.about-principles-list li {
  display: flex;
  align-items: flex-start;
  gap: 0.55rem;
  font-size: 0.95rem;
  line-height: 1.55;
  color: var(--ink-2);
}

.about-principles-list li::before {
  content: '';
  display: inline-block;
  width: 1.1rem;
  height: 1.1rem;
  border-radius: 50%;
  border: 1.5px solid color-mix(in srgb, var(--brand-1) 55%, var(--line) 45%);
  background: color-mix(in srgb, var(--brand-1) 8%, transparent 92%);
  flex-shrink: 0;
  margin-top: 0.2rem;
}

.about-principles-quote {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  text-align: center;
  padding: 2rem 1.5rem;
}

.about-principles-shield {
  font-size: 2.4rem;
  color: var(--brand-1);
  font-variation-settings: 'FILL' 1;
}

.about-principles-quote p {
  margin: 0;
  font-size: 0.95rem;
  line-height: 1.65;
  color: var(--ink-2);
  font-style: italic;
  max-width: 32ch;
}

/* ── Responsive ─────────────────────────────────────────────────── */
@media (max-width: 860px) {
  .about-hero-inner {
    grid-template-columns: 1fr;
  }

  .about-hero-widget {
    display: none;
  }

  .about-kpi-strip,
  .about-pillars {
    grid-template-columns: 1fr;
  }

  .about-roadmap-section,
  .about-principles-body {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .about-hero {
    padding: 2.8rem 1rem 2.2rem;
  }

  .about-headline {
    font-size: 2.2rem;
  }
}
</style>
