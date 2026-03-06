<script setup>
import { storeToRefs } from 'pinia'
import { useAlertsStore } from '../stores/alerts'

const alertsStore = useAlertsStore()
const { items } = storeToRefs(alertsStore)

function iconFor(type) {
  if (type === 'success') return 'check_circle'
  if (type === 'error') return 'error'
  if (type === 'warning') return 'warning'
  return 'info'
}
</script>

<template>
  <section class="alert-host" aria-live="polite" aria-label="Notifications">
    <article
      v-for="item in items"
      :key="item.id"
      class="alert-toast"
      :class="`alert-${item.type}`"
      role="status"
    >
      <span class="material-symbols-rounded alert-icon" aria-hidden="true">{{ iconFor(item.type) }}</span>
      <p class="alert-message">{{ item.message }}</p>
      <button class="alert-dismiss" type="button" @click="alertsStore.remove(item.id)">
        <span class="material-symbols-rounded" aria-hidden="true">close</span>
        <span class="sr-only">Dismiss notification</span>
      </button>
    </article>
  </section>
</template>

<style scoped>
.alert-host {
  position: fixed;
  right: 1rem;
  bottom: 1rem;
  z-index: 120;
  display: grid;
  gap: 0.45rem;
  width: min(380px, calc(100vw - 2rem));
}

.alert-toast {
  border-radius: 12px;
  border: 1px solid var(--line);
  background: var(--card);
  box-shadow: 0 12px 24px rgba(14, 30, 61, 0.24);
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: start;
  gap: 0.45rem;
  padding: 0.52rem 0.54rem;
}

.alert-message {
  margin: 0.05rem 0 0;
  color: var(--ink-1);
  font-weight: 600;
}

.alert-icon {
  margin-top: 0.08rem;
}

.alert-dismiss {
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-2);
  min-width: 1.8rem;
  min-height: 1.8rem;
  border-radius: 8px;
  padding: 0.2rem;
}

.alert-success {
  border-color: color-mix(in srgb, #1f8f56 40%, var(--line) 60%);
}

.alert-success .alert-icon {
  color: #43c985;
}

.alert-error {
  border-color: color-mix(in srgb, #c14563 44%, var(--line) 56%);
}

.alert-error .alert-icon {
  color: #ef7d9b;
}

.alert-info {
  border-color: color-mix(in srgb, var(--brand-2) 46%, var(--line) 54%);
}

.alert-info .alert-icon {
  color: var(--brand-1);
}

.alert-warning {
  border-color: color-mix(in srgb, #f0b83a 42%, var(--line) 58%);
}

.alert-warning .alert-icon {
  color: #f0b83a;
}

@media (max-width: 640px) {
  .alert-host {
    right: 0.7rem;
    left: 0.7rem;
    width: auto;
    bottom: 0.7rem;
  }
}
</style>
