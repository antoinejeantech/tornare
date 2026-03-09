<script setup>
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConfirmsStore } from '../stores/confirms'

const confirmsStore = useConfirmsStore()
const { current } = storeToRefs(confirmsStore)

const isOpen = computed(() => Boolean(current.value))
const dialogRef = ref(null)
const cancelButtonRef = ref(null)
let previouslyFocusedElement = null

function confirmButtonClass() {
  if (!current.value) {
    return 'btn-primary'
  }

  if (current.value.tone === 'danger') {
    return 'btn-danger'
  }

  if (current.value.tone === 'warning') {
    return 'confirm-btn-warning'
  }

  return 'btn-primary'
}

function getFocusableElements() {
  if (!dialogRef.value) {
    return []
  }

  const selectors = [
    'button:not([disabled])',
    '[href]',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ]

  return Array.from(dialogRef.value.querySelectorAll(selectors.join(', '))).filter((el) => {
    return el.getAttribute('aria-hidden') !== 'true'
  })
}

function focusInitialElement() {
  nextTick(() => {
    if (cancelButtonRef.value) {
      cancelButtonRef.value.focus()
      return
    }

    if (dialogRef.value) {
      dialogRef.value.focus()
    }
  })
}

function restorePreviousFocus() {
  if (previouslyFocusedElement && typeof previouslyFocusedElement.focus === 'function') {
    previouslyFocusedElement.focus()
  }

  previouslyFocusedElement = null
}

function closeWith(value) {
  confirmsStore.respond(value)
}

function onBackdropClick(event) {
  if (event.target !== event.currentTarget) {
    return
  }

  closeWith(false)
}

function onKeydown(event) {
  if (!isOpen.value) {
    return
  }

  if (event.key === 'Escape') {
    event.preventDefault()
    closeWith(false)
    return
  }

  if (event.key !== 'Tab') {
    return
  }

  const focusableElements = getFocusableElements()
  if (focusableElements.length === 0) {
    event.preventDefault()
    if (dialogRef.value) {
      dialogRef.value.focus()
    }
    return
  }

  const first = focusableElements[0]
  const last = focusableElements[focusableElements.length - 1]
  const active = document.activeElement

  if (!dialogRef.value?.contains(active)) {
    event.preventDefault()
    first.focus()
    return
  }

  if (event.shiftKey && active === first) {
    event.preventDefault()
    last.focus()
    return
  }

  if (!event.shiftKey && active === last) {
    event.preventDefault()
    first.focus()
  }
}

watch(isOpen, (open) => {
  if (typeof document === 'undefined') {
    return
  }

  document.body.style.overflow = open ? 'hidden' : ''

  if (typeof window !== 'undefined') {
    if (open) {
      const active = document.activeElement
      if (active instanceof HTMLElement) {
        previouslyFocusedElement = active
      } else {
        previouslyFocusedElement = null
      }

      window.addEventListener('keydown', onKeydown)
      focusInitialElement()
    } else {
      window.removeEventListener('keydown', onKeydown)
      restorePreviousFocus()
    }
  }
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }

  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', onKeydown)
  }

  restorePreviousFocus()
})
</script>

<template>
  <Teleport to="body">
    <transition name="confirm-fade">
      <section
        v-if="current"
        class="confirm-backdrop"
        role="presentation"
        @click="onBackdropClick"
      >
        <article ref="dialogRef" class="confirm-dialog" role="dialog" aria-modal="true" :aria-label="current.title" tabindex="-1">
          <h3 class="confirm-title">{{ current.title }}</h3>
          <p class="confirm-message">{{ current.message }}</p>
          <div class="confirm-actions">
            <button ref="cancelButtonRef" class="btn-secondary" type="button" @click="closeWith(false)">
              {{ current.cancelText }}
            </button>
            <button
              type="button"
              :class="confirmButtonClass()"
              @click="closeWith(true)"
            >
              {{ current.confirmText }}
            </button>
          </div>
        </article>
      </section>
    </transition>
  </Teleport>
</template>

<style scoped>
.confirm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 220;
  display: grid;
  place-items: center;
  padding: 1rem;
  background: rgba(4, 8, 14, 0.64);
  backdrop-filter: blur(2px);
}

.confirm-dialog {
  width: min(520px, calc(100vw - 2rem));
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--line) 74%, var(--brand-2) 26%);
  background:
    radial-gradient(180px 90px at 100% 0%, color-mix(in srgb, var(--brand-2) 14%, transparent 86%), transparent 70%),
    linear-gradient(180deg, color-mix(in srgb, var(--card) 94%, #f0f6ff 6%), color-mix(in srgb, var(--card) 98%, #f8fbff 2%));
  box-shadow:
    0 18px 42px rgba(2, 6, 14, 0.46),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
  padding: 0.95rem;
  display: grid;
  gap: 0.75rem;
}

.confirm-title {
  margin: 0;
  color: var(--heading-ink);
}

.confirm-message {
  margin: 0;
  color: var(--ink-1);
  line-height: 1.4;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.confirm-btn-warning {
  border: 1px solid color-mix(in srgb, var(--warn) 46%, var(--line) 54%);
  background: color-mix(in srgb, var(--warn) 24%, var(--card) 76%);
  color: color-mix(in srgb, var(--warn) 76%, var(--ink-1) 24%);
}

.confirm-btn-warning:hover {
  border-color: color-mix(in srgb, var(--warn) 62%, var(--line) 38%);
  background: color-mix(in srgb, var(--warn) 34%, var(--card) 66%);
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 150ms ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}

@media (max-width: 640px) {
  .confirm-dialog {
    padding: 0.85rem;
  }

  .confirm-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .confirm-actions button {
    width: 100%;
  }
}
</style>
