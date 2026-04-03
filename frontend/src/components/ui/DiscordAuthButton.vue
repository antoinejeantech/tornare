<script setup lang="ts">
import DiscordIcon from './DiscordIcon.vue'

defineProps<{
  label: string
  recommended?: boolean
  disabled?: boolean
}>()

defineEmits<{ click: [] }>()
</script>

<template>
  <button
    type="button"
    class="btn-discord"
    :disabled="disabled"
    @click="$emit('click')"
  >
    <span v-if="recommended" class="btn-discord-recommended btn-discord-spacer" aria-hidden="true">{{ $t('auth.discordRecommended') }}</span>
    <DiscordIcon class="btn-discord-logo" />
    <span class="btn-discord-label">{{ label }}</span>
    <span v-if="recommended" class="btn-discord-recommended">{{ $t('auth.discordRecommended') }}</span>
  </button>
</template>

<style scoped>
.btn-discord {
  width: 100%;
  border: none;
  border-radius: var(--radius-md);
  padding: 0.72rem 1.1rem;
  background: #5865f2;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  cursor: pointer;
  transition: background 120ms ease, box-shadow 120ms ease;
  box-shadow: 0 2px 8px rgb(88 101 242 / 32%);
}

.btn-discord:hover {
  background: #6470f3;
  box-shadow: 0 3px 12px rgb(88 101 242 / 44%);
}

.btn-discord:active {
  background: #4752c4;
  box-shadow: 0 1px 4px rgb(88 101 242 / 24%);
}

.btn-discord-logo {
  width: 1.45rem;
  height: 1.45rem;
  flex-shrink: 0;
}

.btn-discord-label {
  font-weight: 700;
  font-size: 0.97rem;
  letter-spacing: 0.01em;
}

.btn-discord-recommended {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  background: rgb(255 255 255 / 22%);
  border-radius: 4px;
  padding: 0.15em 0.5em;
  line-height: 1.5;
  flex-shrink: 0;
}

.btn-discord-spacer {
  visibility: hidden;
}
</style>
