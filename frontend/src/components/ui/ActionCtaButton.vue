<script setup>
import { RouterLink } from 'vue-router'

const props = defineProps({
  to: {
    type: [String, Object],
    required: true,
  },
})
</script>

<template>
  <RouterLink :to="props.to" class="action-cta-button">
    <slot />
  </RouterLink>
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
</style>
