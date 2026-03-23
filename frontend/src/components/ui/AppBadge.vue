<script lang="ts">
export const VARIANTS = ['ok', 'warning', 'danger', 'info', 'neutral', 'accent', 'muted']
</script>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  /** Visual variant: ok | warning | danger | info | neutral | accent | muted */
  variant?: string
  label?: string
  /** Border radius override. Token shorthand or raw CSS value:
   * 'sm' | 'item' | 'md' | 'lg' | 'pill' | any CSS value (e.g. '4px') */
  radius?: string | null
  /** Custom background (overrides variant). Any CSS value or token reference. */
  bg?: string | null
  /** Custom text color (overrides variant). */
  color?: string | null
  /** Custom border color (overrides variant). */
  border?: string | null
}>(), {
  variant: 'neutral',
  label: '',
  radius: null,
  bg: null,
  color: null,
  border: null,
})

const RADIUS_TOKENS = {
  sm:   'var(--radius-sm)',
  md:   'var(--radius-md)',
  lg:   'var(--radius-lg)',
  pill: 'var(--radius-pill)',
}

const safeVariant = computed(() =>
  VARIANTS.includes(props.variant) ? props.variant : 'neutral'
)

const styleObject = computed(() => ({
  ...(props.radius ? { borderRadius: RADIUS_TOKENS[props.radius as keyof typeof RADIUS_TOKENS] ?? props.radius } : {}),
  ...(props.bg     ? { background: props.bg }         : {}),
  ...(props.color  ? { color: props.color }            : {}),
  ...(props.border ? { borderColor: props.border }     : {}),
}))
</script>

<template>
  <span class="app-badge" :class="`is-${safeVariant}`" :style="styleObject">
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

.app-badge.is-warning {
  background: var(--warn-bg);
  color: var(--warn-ink);
  border-color: var(--warn-soft);
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
