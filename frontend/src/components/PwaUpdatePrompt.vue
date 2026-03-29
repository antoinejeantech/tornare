<script setup lang="ts">
import { ref } from 'vue'
import { needRefresh, applyUpdate as doUpdate } from '@/lib/pwa'

const updating = ref(false)

async function applyUpdate() {
  updating.value = true
  try {
    await doUpdate()
  } finally {
    updating.value = false
  }
}
</script>

<template>
  <Transition name="pwa-prompt">
    <div v-if="needRefresh" class="pwa-prompt" role="status" aria-live="polite">
      <span class="material-symbols-rounded pwa-prompt-icon" aria-hidden="true">system_update</span>
      <span class="pwa-prompt-text">A new version is available.</span>
      <button class="pwa-prompt-btn" :disabled="updating" @click="applyUpdate">
        {{ updating ? 'Updating…' : 'Update now' }}
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.pwa-prompt {
  position: fixed;
  bottom: 1.25rem;
  left: 50%;
  transform: translateX(-50%);
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 0.9rem 0.6rem 0.75rem;
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--line-strong) 82%, white 18%);
  background: color-mix(in srgb, var(--grey-900) 90%, black 10%);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  color: var(--ink-1);
  font-size: 0.88rem;
  z-index: 9999;
  white-space: nowrap;
}

.pwa-prompt-icon {
  font-size: 1.1rem;
  color: color-mix(in srgb, var(--brand-1) 80%, #ffd869 20%);
}

.pwa-prompt-text {
  color: var(--ink-2);
}

.pwa-prompt-btn {
  padding: 0.3rem 0.7rem;
  border-radius: var(--radius-sm);
  border: 1px solid color-mix(in srgb, var(--brand-1) 50%, transparent 50%);
  background: color-mix(in srgb, var(--brand-1) 14%, transparent 86%);
  color: var(--brand-1);
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
}

.pwa-prompt-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--brand-1) 22%, transparent 78%);
}

.pwa-prompt-enter-active,
.pwa-prompt-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}

.pwa-prompt-enter-from,
.pwa-prompt-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(0.5rem);
}
</style>
