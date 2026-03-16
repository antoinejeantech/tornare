<script setup>
defineProps({
  /**
   * Visual variant: ok | danger | info | neutral | accent | muted
   */
  variant: {
    type: String,
    default: 'neutral',
  },
  label: {
    type: String,
    default: '',
  },
})
</script>

<template>
  <span class="app-badge" :class="`is-${variant}`">
    <slot>{{ label }}</slot>
  </span>
</template>

<style scoped>
.app-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  border-radius: var(--radius-sm);
  padding: 0.14rem 0.42rem;
  text-align: center;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  border: 1px solid transparent;
}

.app-badge.is-ok {
  background: var(--ok-bg);
  color: var(--ok-ink);
  border-color: var(--ok-soft);
}

.app-badge.is-danger {
  background: var(--danger-bg);
  color: var(--danger-ink);
  border-color: var(--danger-soft);
}

.app-badge.is-info {
  background: var(--info-bg);
  color: var(--info-ink);
  border-color: var(--info-soft);
}

.app-badge.is-neutral {
  background: color-mix(in srgb, var(--primary-700) 72%, var(--bg-1) 28%);
  color: white;
  border-color: transparent;
}

.app-badge.is-accent {
  background: color-mix(in srgb, var(--brand-1) 15%, var(--bg-1) 85%);
  color: var(--brand-1);
  border-color: color-mix(in srgb, var(--brand-1) 40%, transparent 60%);
}

.app-badge.is-muted {
  background: var(--bg-1);
  color: var(--ink-muted);
  border-color: var(--line);
}
</style>
