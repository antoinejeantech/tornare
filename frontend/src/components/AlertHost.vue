<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAlertsStore } from '../stores/alerts'
import type { AlertType } from '../stores/alerts'

const { t } = useI18n()
const alertsStore = useAlertsStore()
const { items } = storeToRefs(alertsStore)

function iconFor(type: AlertType): string {
  if (type === 'success') return 'check_circle'
  if (type === 'error') return 'error'
  if (type === 'warning') return 'warning'
  return 'info'
}
</script>

<template>
  <section class="alert-host" aria-live="polite" :aria-label="t('nav.notifications')">
    <TransitionGroup name="alert-slide" tag="div" class="alert-list">
      <article
        v-for="item in items"
        :key="item.id"
        class="alert-toast"
        :class="`alert-${item.type}`"
        role="status"
      >
        <span class="material-symbols-rounded alert-icon" aria-hidden="true">{{ iconFor(item.type) }}</span>

        <div class="alert-body">
          <p v-if="item.title" class="alert-title">{{ item.title }}</p>
          <p class="alert-message" :class="{ 'alert-message--alone': !item.title }">{{ item.message }}</p>
        </div>

        <button class="alert-dismiss" type="button" :aria-label="t('alerts.dismiss')" @click="alertsStore.remove(item.id)">
          <span class="material-symbols-rounded" aria-hidden="true">close</span>
        </button>
      </article>
    </TransitionGroup>
  </section>
</template>

<style scoped>
.alert-host {
  position: fixed;
  right: 1rem;
  bottom: 1rem;
  z-index: 120;
  width: min(400px, calc(100vw - 2rem));
  pointer-events: none;
}

.alert-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  align-items: stretch;
}

/* ── Toast ── */
.alert-toast {
  pointer-events: all;
  border-radius: var(--radius-md);
  border: 1px solid var(--line);
  border-left: 4px solid currentColor;
  background: color-mix(in srgb, var(--card) 96%, var(--bg-0) 4%);
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.18),
    0 8px 24px rgba(0, 0, 0, 0.22);
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 0 0.65rem;
  padding: 0.7rem 0.6rem 0.7rem 0.7rem;
  overflow: hidden;
}

/* ── Icon ── */
.alert-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
  /* filled style */
  font-variation-settings: 'FILL' 1, 'wght' 500, 'GRAD' 0, 'opsz' 20;
}

/* ── Body ── */
.alert-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.alert-title {
  margin: 0;
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--ink-1);
  letter-spacing: 0.01em;
}

.alert-message {
  margin: 0;
  font-size: 0.82rem;
  color: var(--ink-2);
  line-height: 1.45;
}

.alert-message--alone {
  font-weight: 600;
  color: var(--ink-1);
  font-size: 0.84rem;
}

/* ── Dismiss button ── */
.alert-dismiss {
  border: none;
  background: transparent;
  color: var(--ink-muted);
  width: 1.8rem;
  height: 1.8rem;
  border-radius: var(--radius-sm);
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.12s, background 0.12s;
}

.alert-dismiss:hover {
  color: var(--ink-1);
  background: color-mix(in srgb, var(--line) 60%, transparent 40%);
}

.alert-dismiss .material-symbols-rounded {
  font-size: 1rem;
}

/* ── Type colours (accent + icon inherit currentColor) ── */
.alert-success {
  color: #43c985;
  border-color: color-mix(in srgb, #1f8f56 50%, var(--line) 50%);
  background: color-mix(in srgb, #1f8f56 12%, var(--card) 88%);
}
.alert-error {
  color: #ef7d9b;
  border-color: color-mix(in srgb, #c14563 52%, var(--line) 48%);
  background: color-mix(in srgb, #c14563 12%, var(--card) 88%);
}
.alert-warning {
  color: #f0b83a;
  border-color: color-mix(in srgb, #c48a10 50%, var(--line) 50%);
  background: color-mix(in srgb, #c48a10 12%, var(--card) 88%);
}
.alert-info {
  color: var(--brand-1);
  border-color: color-mix(in srgb, var(--brand-2) 54%, var(--line) 46%);
  background: color-mix(in srgb, var(--brand-2) 12%, var(--card) 88%);
}

/* ── Enter / leave transitions ── */
.alert-slide-enter-active {
  transition: transform 0.22s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.18s ease;
}
.alert-slide-leave-active {
  transition: transform 0.18s ease-in, opacity 0.18s ease-in;
  position: absolute;
  width: 100%;
}
.alert-slide-move {
  transition: transform 0.22s ease;
}
.alert-slide-enter-from {
  transform: translateX(calc(100% + 1.5rem));
  opacity: 0;
}
.alert-slide-leave-to {
  transform: translateX(calc(100% + 1.5rem));
  opacity: 0;
}

@media (max-width: 640px) {
  .alert-host {
    right: 0.6rem;
    left: 0.6rem;
    bottom: 0.75rem;
    width: auto;
  }
}
</style>
