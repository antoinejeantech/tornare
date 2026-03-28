<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import type { RouteLocationRaw } from 'vue-router'

const props = withDefaults(defineProps<{
  to?: RouteLocationRaw
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
  variant?: string
  fullWidth?: boolean
  withTopSpacing?: boolean
  size?: string
}>(), {
  to: undefined,
  disabled: false,
  type: 'button',
  variant: 'solid',
  fullWidth: true,
  withTopSpacing: true,
  size: 'compact',
})

const variantClass = computed(() => {
  return props.variant === 'muted' ? 'event-action-btn-muted' : 'event-action-btn-solid'
})

const widthClass = computed(() => {
  return props.fullWidth ? 'event-action-btn-full' : 'event-action-btn-auto'
})

const spacingClass = computed(() => {
  return props.withTopSpacing ? 'event-action-btn-with-top-spacing' : 'event-action-btn-no-top-spacing'
})

const sizeClass = computed(() => {
  return props.size === 'cta' ? 'event-action-btn-size-cta' : 'event-action-btn-size-compact'
})
</script>

<template>
  <RouterLink
    v-if="to !== undefined"
    :to="to"
    :class="['event-action-btn', variantClass, widthClass, spacingClass, sizeClass]"
  >
    <slot>Open event</slot>
  </RouterLink>
  <button
    v-else
    :type="type"
    :disabled="disabled"
    :class="['event-action-btn', variantClass, widthClass, spacingClass, sizeClass]"
  >
    <slot>Open event</slot>
  </button>
</template>

<style scoped>
.event-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  text-decoration: none;
}

.event-action-btn-size-compact {
  min-height: 0;
  padding: 0.4rem 0.58rem;
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.event-action-btn-size-cta {
  min-height: 2.2rem;
  padding: 0.52rem 0.92rem;
  font-size: 0.95rem;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.event-action-btn-full {
  width: 100%;
}

.event-action-btn-auto {
  width: auto;
}

.event-action-btn-with-top-spacing {
  margin-top: var(--space-1);
}

.event-action-btn-no-top-spacing {
  margin-top: 0;
}

.event-action-btn-solid {
  border-color: var(--line-strong);
  background: color-mix(in srgb, var(--grey-900) 74%, black 26%);
  color: white;
}

.event-action-btn-solid:hover {
  color: white;
  border-color: color-mix(in srgb, var(--line-strong) 82%, white 18%);
}

.event-action-btn-muted {
  border-color: transparent;
  background: transparent;
  color: var(--ink-muted);
}

.event-action-btn-muted:hover {
  color: var(--ink-2);
}
</style>
