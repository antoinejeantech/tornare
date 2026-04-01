<script setup lang="ts">
import { getCurrentInstance, nextTick, onBeforeUnmount, ref, watch } from 'vue'

const props = withDefaults(defineProps<{
  open: boolean
  title: string
  maxWidth?: string
}>(), {
  maxWidth: 'min(760px, 100%)',
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const uid = getCurrentInstance()?.uid ?? Math.random()
const dialogRef = ref<HTMLElement | null>(null)
let previouslyFocusedElement: HTMLElement | null = null

function close() {
  emit('update:open', false)
}

function getFocusable(): HTMLElement[] {
  if (!dialogRef.value) return []
  return Array.from(
    dialogRef.value.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    )
  ).filter(el => el.getAttribute('aria-hidden') !== 'true')
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) close()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
    return
  }

  if (e.key !== 'Tab') return

  const els = getFocusable()
  if (!els.length) {
    e.preventDefault()
    dialogRef.value?.focus()
    return
  }

  const first = els[0]
  const last = els[els.length - 1]
  const active = document.activeElement

  if (!dialogRef.value?.contains(active)) {
    e.preventDefault()
    first.focus()
    return
  }

  if (e.shiftKey && active === first) {
    e.preventDefault()
    last.focus()
    return
  }

  if (!e.shiftKey && active === last) {
    e.preventDefault()
    first.focus()
  }
}

watch(() => props.open, (open) => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = open ? 'hidden' : ''
  if (typeof window === 'undefined') return

  if (open) {
    const active = document.activeElement
    previouslyFocusedElement = active instanceof HTMLElement ? active : null
    window.addEventListener('keydown', onKeydown)
    nextTick(() => {
      const els = getFocusable()
      if (els.length) els[0].focus()
      else dialogRef.value?.focus()
    })
  } else {
    window.removeEventListener('keydown', onKeydown)
    previouslyFocusedElement?.focus()
    previouslyFocusedElement = null
  }
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') document.body.style.overflow = ''
  if (typeof window !== 'undefined') window.removeEventListener('keydown', onKeydown)
  previouslyFocusedElement?.focus()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="app-modal">
      <div
        v-if="open"
        class="app-modal-backdrop"
        role="presentation"
        @click="onBackdropClick"
      >
        <section
          ref="dialogRef"
          class="app-modal card"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="`app-modal-title-${uid}`"
          :style="{ width: maxWidth }"
          tabindex="-1"
        >
          <header class="app-modal-header">
            <h2 :id="`app-modal-title-${uid}`" class="app-modal-title">{{ title }}</h2>
            <button
              class="app-modal-close"
              type="button"
              aria-label="Close"
              @click="close"
            >
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
            </button>
          </header>
          <div class="app-modal-body">
            <slot />
          </div>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.app-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(7, 14, 30, 0.55);
  backdrop-filter: blur(3px);
  z-index: 70;
  display: grid;
  place-items: center;
  padding: 1rem;
}

.app-modal {
  max-height: calc(100vh - 2rem);
  overflow: auto;
  outline: none;
}

.app-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  margin-bottom: 0.85rem;
}

.app-modal-title {
  margin: 0;
}

.app-modal-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  padding: 0;
  flex-shrink: 0;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--ink-2);
  cursor: pointer;
  transition: background 0.14s, color 0.14s, border-color 0.14s;
}

.app-modal-close:hover {
  background: color-mix(in srgb, var(--line) 40%, transparent 60%);
  color: var(--ink-1);
  border-color: color-mix(in srgb, var(--line) 80%, var(--brand-2) 20%);
}

.app-modal-close .material-symbols-rounded {
  font-size: 1.1rem;
}

/* Transition */
.app-modal-enter-active {
  transition: opacity 0.2s ease;
}
.app-modal-leave-active {
  transition: opacity 0.15s ease;
}
.app-modal-enter-from,
.app-modal-leave-to {
  opacity: 0;
}
.app-modal-enter-active .app-modal {
  transition: transform 0.22s cubic-bezier(0.34, 1.36, 0.64, 1), opacity 0.2s ease;
}
.app-modal-leave-active .app-modal {
  transition: transform 0.15s ease, opacity 0.15s ease;
}
.app-modal-enter-from .app-modal {
  transform: translateY(12px) scale(0.98);
  opacity: 0;
}
.app-modal-leave-to .app-modal {
  transform: translateY(6px) scale(0.99);
  opacity: 0;
}
</style>
