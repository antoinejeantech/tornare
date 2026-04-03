<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

const { t } = useI18n()
const openIds = ref(new Set<string>())

function toggle(id: string) {
  if (openIds.value.has(id)) {
    openIds.value.delete(id)
  } else {
    openIds.value.add(id)
  }
  // trigger reactivity
  openIds.value = new Set(openIds.value)
}

const sections = computed(() => [
  {
    id: 'getting-started',
    title: t('faq.s1Title'),
    icon: 'rocket_launch',
    questions: [
      { id: 'gs-1', q: t('faq.s1q1'), a: t('faq.s1a1') },
      { id: 'gs-2', q: t('faq.s1q2'), a: t('faq.s1a2') },
      { id: 'gs-3', q: t('faq.s1q3'), a: t('faq.s1a3') },
      { id: 'gs-4', q: t('faq.s1q4'), a: t('faq.s1a4') },
    ]
  },
  {
    id: 'events',
    title: t('faq.s2Title'),
    icon: 'event',
    questions: [
      { id: 'ev-1', q: t('faq.s2q1'), a: t('faq.s2a1') },
      { id: 'ev-2', q: t('faq.s2q2'), a: t('faq.s2a2') },
      { id: 'ev-3', q: t('faq.s2q3'), a: t('faq.s2a3') },
      { id: 'ev-4', q: t('faq.s2q4'), a: t('faq.s2a4') },
      { id: 'ev-5', q: t('faq.s2q5'), a: t('faq.s2a5') },
    ]
  },
  {
    id: 'players',
    title: t('faq.s3Title'),
    icon: 'group',
    questions: [
      { id: 'pl-1', q: t('faq.s3q1'), a: t('faq.s3a1') },
      { id: 'pl-2', q: t('faq.s3q2'), a: t('faq.s3a2') },
      { id: 'pl-3', q: t('faq.s3q3'), a: t('faq.s3a3') },
      { id: 'pl-4', q: t('faq.s3q4'), a: t('faq.s3a4') },
    ]
  },
  {
    id: 'accounts',
    title: t('faq.s4Title'),
    icon: 'manage_accounts',
    questions: [
      { id: 'ac-1', q: t('faq.s4q1'), a: t('faq.s4a1') },
      { id: 'ac-2', q: t('faq.s4q2'), a: t('faq.s4a2') },
      { id: 'ac-3', q: t('faq.s4q3'), a: t('faq.s4a3') },
      { id: 'ac-4', q: t('faq.s4q4'), a: t('faq.s4a4') },
    ]
  }
])
</script>

<template>
  <main class="faq-shell">

    <header class="faq-header">
      <span class="material-symbols-rounded faq-header-icon" aria-hidden="true">help</span>
      <div>
        <h1 class="faq-title">{{ t('faq.title') }}</h1>
        <p class="faq-subtitle">{{ t('faq.subtitle') }}</p>
      </div>
    </header>

    <section
      v-for="section in sections"
      :key="section.id"
      class="card faq-section"
    >
      <div class="faq-section-head">
        <span class="material-symbols-rounded faq-section-icon" aria-hidden="true">{{ section.icon }}</span>
        <h2 class="faq-section-title">{{ section.title }}</h2>
      </div>

      <ul class="faq-list">
        <li
          v-for="item in section.questions"
          :key="item.id"
          class="faq-item"
          :class="{ 'faq-item--open': openIds.has(item.id) }"
        >
          <button class="faq-question" @click="toggle(item.id)" :aria-expanded="openIds.has(item.id)">
            <span>{{ item.q }}</span>
            <span class="material-symbols-rounded faq-chevron" aria-hidden="true">expand_more</span>
          </button>
          <div v-if="openIds.has(item.id)" class="faq-answer">
            <p>{{ item.a }}</p>
          </div>
        </li>
      </ul>
    </section>

    <div class="faq-footer card">
      <span class="material-symbols-rounded faq-footer-icon" aria-hidden="true">live_help</span>
      <p>{{ t('faq.footerText') }} <RouterLink to="/support" class="faq-footer-link">{{ t('faq.footerLink') }}</RouterLink></p>
    </div>

  </main>
</template>

<style scoped>
.faq-shell {
  width: 100%;
  max-width: 760px;
  margin-inline: auto;
  padding: 2rem 1.25rem 4rem;
  display: grid;
  gap: 1.1rem;
}

.faq-section,
.faq-footer {
  min-width: 0;
}

.faq-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 0 0.25rem;
}

.faq-header-icon {
  font-size: 2.6rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
  flex-shrink: 0;
}

.faq-title {
  margin: 0;
  font-size: clamp(1.5rem, 3vw, 2.1rem);
  letter-spacing: -0.02em;
}

.faq-subtitle {
  margin: 0.15rem 0 0;
  color: var(--ink-2);
  font-size: 1rem;
}

/* ── Section ───────────────────────────────────────── */
.faq-section {
  display: grid;
  gap: 0.6rem;
}

.faq-section-head {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.faq-section-icon {
  font-size: 1.25rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
}

.faq-section-title {
  margin: 0;
  font-size: 1.05rem;
  letter-spacing: -0.01em;
}

/* ── List ──────────────────────────────────────────── */
.faq-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.35rem;
}

.faq-item {
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: border-color 160ms;
}

.faq-item--open {
  border-color: color-mix(in srgb, var(--brand-1) 28%, var(--line) 72%);
}

.faq-question {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.65rem 0.8rem;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  color: var(--ink-1);
  font-size: 0.93rem;
  font-weight: 600;
  line-height: 1.45;
  transition: background 120ms;
}

.faq-question:hover {
  background: color-mix(in srgb, var(--brand-1) 5%, transparent 95%);
}

.faq-chevron {
  font-size: 1.15rem;
  flex-shrink: 0;
  color: var(--ink-3);
  transition: transform 200ms;
}

.faq-item--open .faq-chevron {
  transform: rotate(180deg);
}

.faq-answer {
  padding: 0 0.8rem 0.7rem;
  border-top: 1px solid var(--line);
}

.faq-answer p {
  margin: 0.6rem 0 0;
  font-size: 0.88rem;
  color: var(--ink-2);
  line-height: 1.65;
}

/* ── Footer prompt ─────────────────────────────────── */
.faq-footer {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.faq-footer-icon {
  font-size: 1.5rem;
  font-variation-settings: 'FILL' 1;
  color: var(--ink-3);
  flex-shrink: 0;
}

.faq-footer p {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-2);
}

.faq-footer-link {
  color: var(--brand-1);
  font-weight: 700;
  text-decoration: none;
}

.faq-footer-link:hover {
  text-decoration: underline;
}
</style>
