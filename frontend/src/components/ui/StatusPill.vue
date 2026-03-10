<script setup>
import { computed } from 'vue'

const props = defineProps({
  status: {
    type: String,
    default: '',
  },
  label: {
    type: String,
    default: '',
  },
  tone: {
    type: String,
    default: '',
  },
})

const text = computed(() => {
  return String(props.label || props.status || '').trim()
})

const toneClass = computed(() => {
  if (props.tone === 'neutral') {
    return 'is-neutral'
  }

  if (props.status === 'Full') {
    return 'is-full'
  }
  if (props.status === 'Progress') {
    return 'is-progress'
  }

  return 'is-open'
})
</script>

<template>
  <span class="status-pill" :class="toneClass">{{ text }}</span>
</template>

<style scoped>
.status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
  border-radius: var(--radius-sm);
  padding: 0.14rem 0.22rem;
  text-align: center;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: white;
  border: 1px solid transparent;
}

.status-pill.is-open {
  background: var(--ok-soft);
  border-color: var(--ok-soft);
}

.status-pill.is-full {
  background: var(--danger-soft);
  border-color: var(--danger-soft);
}

.status-pill.is-progress {
  background: var(--info-soft);
  border-color: var(--info-soft);
}

.status-pill.is-neutral {
  background: color-mix(in srgb, var(--primary-700) 72%, var(--bg-1) 28%);
  border-color: color-mix(in srgb, var(--primary-700) 72%, var(--bg-1) 28%);
}
</style>
