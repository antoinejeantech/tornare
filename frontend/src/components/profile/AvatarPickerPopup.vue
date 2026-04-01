<script setup lang="ts">
import AppModal from '../ui/AppModal.vue'

const props = defineProps<{
  currentAvatarUrl: string | null
  hasDiscord?: boolean
}>()

const emit = defineEmits<{
  (e: 'pick', avatarUrl: string | null): void
  (e: 'close'): void
}>()

const PRESET_AVATARS = [
  { key: '/avatars/ana.webp',        label: 'Ana'         },
  { key: '/avatars/ashe.webp',       label: 'Ashe'        },
  { key: '/avatars/baptiste.webp',   label: 'Baptiste'    },
  { key: '/avatars/bastion.webp',    label: 'Bastion'     },
  { key: '/avatars/brigitte.webp',   label: 'Brigitte'    },
  { key: '/avatars/cassidy.webp',    label: 'Cassidy'     },
  { key: '/avatars/echo.webp',       label: 'Echo'        },
  { key: '/avatars/freja.webp',      label: 'Freja'       },
  { key: '/avatars/genji.webp',      label: 'Genji'       },
  { key: '/avatars/hanzo.webp',      label: 'Hanzo'       },
  { key: '/avatars/illari.webp',     label: 'Illari'      },
  { key: '/avatars/junkrat.webp',    label: 'Junkrat'     },
  { key: '/avatars/kiriko.webp',     label: 'Kiriko'      },
  { key: '/avatars/lifeweaver.webp', label: 'Lifeweaver'  },
  { key: '/avatars/lucio.webp',      label: 'Lúcio'       },
  { key: '/avatars/mei.webp',        label: 'Mei'         },
  { key: '/avatars/mercy.webp',      label: 'Mercy'       },
  { key: '/avatars/moira.webp',      label: 'Moira'       },
  { key: '/avatars/sojourn.webp',    label: 'Sojourn'     },
  { key: '/avatars/soldier76.webp',  label: 'Soldier: 76' },
  { key: '/avatars/symmetra.webp',   label: 'Symmetra'    },
  { key: '/avatars/torbjorn.webp',   label: 'Torbjörn'    },
  { key: '/avatars/tracer.webp',     label: 'Tracer'      },
  { key: '/avatars/venture.webp',    label: 'Venture'     },
  { key: '/avatars/widowmaker.webp', label: 'Widowmaker'  },
  { key: '/avatars/wuyang.webp',     label: 'Wuyang'      },
  { key: '/avatars/zenyatta.webp',   label: 'Zenyatta'    },
]

function pick(key: string | null) {
  emit('pick', key)
}
</script>

<template>
  <AppModal :open="true" title="Choose profile picture" max-width="min(92vw, 520px)" @update:open="!$event && emit('close')">
      <p v-if="hasDiscord" class="avatar-picker-discord-note">
        <span class="material-symbols-rounded" aria-hidden="true">info</span>
        Your Discord avatar syncs automatically each time you log in with Discord.
      </p>
      <p v-else class="avatar-picker-discord-note">
        <span class="material-symbols-rounded" aria-hidden="true">info</span>
        Connect your Discord account to use your Discord avatar — it syncs automatically on login.
      </p>

      <div class="avatar-picker-grid">
        <button
          v-for="preset in PRESET_AVATARS"
          :key="preset.key"
          type="button"
          class="avatar-picker-option"
          :class="{ 'avatar-picker-option--active': currentAvatarUrl === preset.key }"
          :aria-pressed="currentAvatarUrl === preset.key"
          :title="preset.label"
          @click="pick(preset.key)"
        >
          <img :src="preset.key" :alt="preset.label" />
        </button>

        <button
          type="button"
          class="avatar-picker-option avatar-picker-option--none"
          :class="{ 'avatar-picker-option--active': !currentAvatarUrl || !PRESET_AVATARS.some(p => p.key === currentAvatarUrl) }"
          :aria-pressed="!currentAvatarUrl || !PRESET_AVATARS.some(p => p.key === currentAvatarUrl)"
          title="Default (initials)"
          @click="pick(null)"
        >
          <span class="material-symbols-rounded" aria-hidden="true">person</span>
        </button>
      </div>
  </AppModal>
</template>

<style scoped>
.avatar-picker-grid {
  display: grid;
  grid-template-columns: repeat(5, 82px);
  grid-auto-rows: 82px;
  gap: 0.5rem;
  justify-content: center;
  padding: 0.1rem 0.5rem 0.1rem 0.1rem;
}

@media (max-width: 480px) {
  .avatar-picker-grid {
    grid-template-columns: repeat(4, 72px);
    grid-auto-rows: 72px;
  }

  .avatar-picker-option {
    width: 72px;
    height: 72px;
  }
}

.avatar-picker-option {
  width: 82px;
  height: 82px;
  border-radius: var(--radius-md, 10px);
  border: 3px solid transparent;
  background: color-mix(in srgb, var(--line) 30%, transparent);
  cursor: pointer;
  padding: 0;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 120ms, transform 120ms, box-shadow 120ms;
}

.avatar-picker-option img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-picker-option:hover {
  border-color: var(--brand-1);
  transform: scale(1.04);
}

.avatar-picker-option--active {
  border-color: var(--brand-1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--brand-1) 40%, transparent);
}

.avatar-picker-option--none {
  color: var(--ink-muted);
}

.avatar-picker-option--none .material-symbols-rounded {
  font-size: 2rem;
}

.avatar-picker-discord-note {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  margin: 0 0 0.9rem;
  padding: 0.6rem 0.8rem;
  flex-shrink: 0;
  border-radius: var(--radius-sm, 6px);
  background: color-mix(in srgb, var(--brand-discord, #5865f2) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--brand-discord, #5865f2) 30%, transparent);
  color: var(--ink-muted);
  font-size: 0.85rem;
  line-height: 1.45;
}

.avatar-picker-discord-note .material-symbols-rounded {
  font-size: 1.1rem;
  flex-shrink: 0;
  margin-top: 0.1em;
  color: var(--brand-discord, #5865f2);
}
</style>
