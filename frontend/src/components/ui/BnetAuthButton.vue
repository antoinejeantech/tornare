<script setup lang="ts">
import BnetIcon from './BnetIcon.vue'

defineProps<{
  label: string
  disabled?: boolean
}>()

defineEmits<{ click: [] }>()
</script>

<template>
  <button
    type="button"
    class="btn-bnet"
    :disabled="disabled"
    @click="$emit('click')"
  >
    <BnetIcon class="btn-bnet-logo" />
    <span class="btn-bnet-label">{{ label }}</span>
  </button>
</template>

<style scoped>
.btn-bnet {
  width: 100%;
  border: none;
  border-radius: var(--radius-md);
  padding: 0.72rem 1.1rem;
  background: #148eff;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  cursor: pointer;
  transition: background 120ms ease, box-shadow 120ms ease;
  box-shadow: 0 2px 8px rgb(20 142 255 / 32%);
}

.btn-bnet:hover {
  background: #1a9aff;
  box-shadow: 0 3px 12px rgb(20 142 255 / 44%);
}

.btn-bnet:active {
  background: #0e7de0;
  box-shadow: 0 1px 4px rgb(20 142 255 / 24%);
}

.btn-bnet-logo {
  width: 1.55rem;
  height: 1.55rem;
  display: block;
  flex-shrink: 0;
  color: white;
}

.btn-bnet-label {
  font-weight: 700;
  font-size: 0.97rem;
  letter-spacing: 0.01em;
}
</style>
