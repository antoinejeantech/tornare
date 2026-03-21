<script setup>
import { RouterLink } from 'vue-router'
import { computed } from 'vue'

const props = defineProps({
  to: {
    type: [String, Object],
    default: null,
  },
  type: {
    type: String,
    default: 'button',
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  title: {
    type: String,
    default: '',
  },
})

const emit = defineEmits(['click'])

const isLink = computed(() => props.to !== null && props.to !== undefined)

function handleClick(event) {
  emit('click', event)
}
</script>

<template>
  <RouterLink v-if="isLink" :to="props.to" class="action-cta-button">
    <slot />
  </RouterLink>
  <button v-else :type="type" class="action-cta-button" :disabled="disabled" :title="title" @click="handleClick">
    <slot />
  </button>
</template>

<style scoped>
.action-cta-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 2.2rem;
  padding: 0.52rem 0.92rem;
  border: 1px solid color-mix(in srgb, var(--brand-1) 78%, var(--line) 22%);
  border-radius: var(--radius-sm);
  background-color: color-mix(in srgb, var(--brand-1) 84%, white 16%);
  color: #000;
  text-decoration: none;
  font-size: 0.95rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.18);
  cursor: pointer;
  transform: translateY(0);
  transition: transform 0.16s ease, border-color 0.16s ease, box-shadow 0.16s ease;
}

.action-cta-button:hover {
  color: #000;
  text-decoration: none;
  transform: translateY(-2px);
  border-color: color-mix(in srgb, var(--brand-1) 86%, var(--line) 14%);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.26);
}

.action-cta-button:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--accent) 60%, white 40%);
  outline-offset: 2px;
}

.action-cta-button:disabled {
  cursor: not-allowed;
  opacity: 0.7;
  transform: none;
  box-shadow: none;
}
</style>
