<script setup lang="ts">
defineProps<{
  title: string
  meta: string
}>()
</script>

<template>
  <main class="legal-shell">
    <header class="legal-header">
      <h1 class="legal-title">{{ title }}</h1>
      <p class="legal-meta">{{ meta }}</p>
    </header>
    <div class="legal-body card">
      <slot />
    </div>
  </main>
</template>

<style scoped>
.legal-shell {
  max-width: 780px;
  margin: 0 auto;
  padding: 2.5rem 1rem 4rem;
  display: grid;
  gap: 1.8rem;
  animation: rise-in 300ms ease-out;
}

@keyframes rise-in {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}

.legal-header {
  display: grid;
  gap: 0.35rem;
}

.legal-title {
  font-family: var(--font-heading);
  font-size: 2rem;
  font-weight: 900;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--ink-1);
  margin: 0;
}

.legal-meta {
  font-size: 0.82rem;
  color: var(--ink-muted);
  margin: 0;
}

.legal-body {
  display: grid;
  gap: 0;
  padding: 2rem 2rem;
}

:slotted(.legal-intro) {
  font-size: 0.97rem;
  line-height: 1.7;
  color: var(--ink-2);
  margin: 0 0 1.8rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid var(--line);
}

:slotted(.legal-section) {
  padding: 1.5rem 0;
  border-bottom: 1px solid var(--line);
}

:slotted(.legal-section:last-child) {
  border-bottom: none;
  padding-bottom: 0;
}

:slotted(.legal-section h2) {
  font-family: var(--font-heading);
  font-size: 1.05rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--ink-1);
  margin: 0 0 0.9rem;
}

:slotted(.legal-section h3) {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink-1);
  margin: 1.1rem 0 0.35rem;
}

:slotted(.legal-section p) {
  font-size: 0.92rem;
  line-height: 1.7;
  color: var(--ink-2);
  margin: 0 0 0.65rem;
}

:slotted(.legal-section p:last-child) {
  margin-bottom: 0;
}

:slotted(.legal-section ul) {
  margin: 0.4rem 0 0.65rem 1.1rem;
  padding: 0;
  display: grid;
  gap: 0.45rem;
}

:slotted(.legal-section li) {
  font-size: 0.92rem;
  line-height: 1.65;
  color: var(--ink-2);
}

:slotted(.legal-link) {
  color: var(--brand-1);
  text-decoration: none;
}

:slotted(.legal-link:hover) {
  text-decoration: underline;
}
</style>
