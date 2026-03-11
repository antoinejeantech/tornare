<script setup>
import { RouterLink } from 'vue-router'
import { computed } from 'vue'

const props = defineProps({
  to: {
    type: [String, Object],
    default: null,
  },
  label: {
    type: String,
    default: 'View all events',
  },
  as: {
    type: String,
    default: 'auto',
    validator: (value) => ['auto', 'link', 'text'].includes(value),
  },
  arrowSide: {
    type: String,
    default: 'right',
    validator: (value) => ['left', 'right'].includes(value),
  },
})

const renderAsLink = computed(() => {
  if (props.as === 'link') {
    return Boolean(props.to)
  }
  if (props.as === 'text') {
    return false
  }
  return Boolean(props.to)
})

const rootTag = computed(() => (renderAsLink.value ? RouterLink : 'span'))
const rootAttrs = computed(() => (renderAsLink.value ? { to: props.to } : {}))
const isArrowLeft = computed(() => props.arrowSide === 'left')
const arrowPath = computed(() => {
  return isArrowLeft.value
    ? 'M10 3 5 8l5 5'
    : 'M6 3l5 5-5 5'
})
</script>

<template>
  <component
    :is="rootTag"
    class="inline-arrow-link"
    :class="{ 'arrow-left': isArrowLeft, 'is-text': !renderAsLink }"
    v-bind="rootAttrs"
  >
    <svg v-if="isArrowLeft" viewBox="0 0 16 16" aria-hidden="true">
      <path :d="arrowPath" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span>{{ label }}</span>
    <svg v-if="!isArrowLeft" viewBox="0 0 16 16" aria-hidden="true">
      <path :d="arrowPath" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </component>
</template>

<style scoped>
.inline-arrow-link {
  display: inline-flex;
  align-items: center;
  gap: 0.26rem;
  color: var(--ink-muted);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  text-decoration: none;
}

.inline-arrow-link:hover {
  color: var(--ink-2);
  text-decoration: none;
}

.inline-arrow-link svg {
  width: 0.78rem;
  height: 0.78rem;
  transition: transform 180ms ease;
}

.inline-arrow-link:hover svg {
  transform: translateX(2px);
}

.inline-arrow-link.arrow-left:hover svg {
  transform: translateX(-2px);
}

.inline-arrow-link.is-text {
  cursor: inherit;
}
</style>
