<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { apiCall, getDiscordGuilds } from '../../lib/api'
import { normalizeDatetimeLocalInput } from '../../lib/dates'
import { formatOptionsForType } from '../../lib/event-format'
import AppModal from '../ui/AppModal.vue'
import ActionCtaButton from '../ui/ActionCtaButton.vue'
import DiscordIcon from '../ui/DiscordIcon.vue'
import BnetIcon from '../ui/BnetIcon.vue'
import type { Event, EventFormat } from '../../types'

const { t } = useI18n()

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': [event: Event]
}>()

const newEventName = ref('')
const newEventDescription = ref('')
const newEventStartDate = ref('')
const newEventType = ref('PUG')
const newEventFormat = ref<EventFormat>('5v5')
const newEventSignupVisibility = ref('private')
const newEventMaxPlayers = ref(10)
const newEventRequireDiscord = ref(false)
const newEventRequireBattletag = ref(false)
const newEventDiscordAnnounce = ref(true)
const hasGuild = ref(false)
const creatingEvent = ref(false)
const error = ref('')

const availableFormatOptions = computed(() => formatOptionsForType(newEventType.value))

const isSelectedFormatValid = computed(() =>
  availableFormatOptions.value.includes(newEventFormat.value)
)

const canCreate = computed(() =>
  newEventName.value.trim().length > 0 &&
  newEventDescription.value.trim().length <= 5000 &&
  Number.isInteger(Number(newEventMaxPlayers.value)) &&
  Number(newEventMaxPlayers.value) >= 2 &&
  Number(newEventMaxPlayers.value) <= 99 &&
  isSelectedFormatValid.value
)

watch(newEventType, () => {
  if (!isSelectedFormatValid.value) {
    newEventFormat.value = availableFormatOptions.value[0]
  }
})

function reset() {
  newEventName.value = ''
  newEventDescription.value = ''
  newEventStartDate.value = ''
  newEventType.value = 'PUG'
  newEventFormat.value = '5v5'
  newEventSignupVisibility.value = 'private'
  newEventMaxPlayers.value = 10
  newEventRequireDiscord.value = false
  newEventRequireBattletag.value = false
  newEventDiscordAnnounce.value = true
  error.value = ''
}

watch(() => props.open, async (open) => {
  if (open) {
    try {
      const guilds = await getDiscordGuilds()
      // Guard against the modal being closed while the request was in-flight.
      if (props.open) {
        hasGuild.value = guilds.length > 0
      }
    } catch {
      if (props.open) {
        hasGuild.value = false
      }
    }
  } else {
    reset()
  }
})

async function submit() {
  if (!canCreate.value || creatingEvent.value) return

  let normalizedStartDate = null
  try {
    normalizedStartDate = normalizeDatetimeLocalInput(newEventStartDate.value, 'event start date')
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('createEvent.invalidDate')
    return
  }

  creatingEvent.value = true
  error.value = ''
  try {
    const created = await apiCall<Event>('/api/events', {
      method: 'POST',
      body: JSON.stringify({
        name: newEventName.value.trim(),
        description: newEventDescription.value.trim(),
        start_date: normalizedStartDate,
        event_type: newEventType.value,
        format: newEventFormat.value,
        public_signup_enabled: newEventSignupVisibility.value === 'public',
        max_players: Number(newEventMaxPlayers.value),
        require_discord: newEventRequireDiscord.value,
        require_battletag: newEventRequireBattletag.value,
        discord_announce: hasGuild.value ? newEventDiscordAnnounce.value : true,
      }),
    })
    emit('update:open', false)
    emit('created', created)
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('createEvent.createFailed')
  } finally {
    creatingEvent.value = false
  }
}
</script>

<template>
  <AppModal :open="open" :title="t('createEvent.title')" @update:open="emit('update:open', $event)">
    <form class="create-event-form" @submit.prevent="submit">

      <div class="create-form-section">
        <label>
          {{ t('createEvent.eventName') }}
          <input v-model="newEventName" :placeholder="t('createEvent.eventNamePlaceholder')" />
        </label>
        <label>
          {{ t('createEvent.description') }}
          <textarea v-model="newEventDescription" rows="3" :placeholder="t('createEvent.descriptionPlaceholder')" />
        </label>
      </div>

      <div class="create-form-section">
        <p class="create-form-section-kicker">{{ t('createEvent.formatSection') }}</p>
        <div class="create-form-row-2">
          <label>
            {{ t('createEvent.eventType') }}
            <select v-model="newEventType">
              <option value="PUG">{{ t('events.typePug') }}</option>
              <option value="TOURNEY">{{ t('events.typeTourney') }}</option>
            </select>
          </label>
          <label>
            {{ t('createEvent.formatLabel') }}
            <select v-model="newEventFormat">
              <option v-for="format in availableFormatOptions" :key="`new-event-format-${format}`" :value="format">
                {{ format }}
              </option>
            </select>
          </label>
        </div>
        <div class="create-form-row-2">
          <label>
            {{ t('createEvent.startDate') }}
            <input v-model="newEventStartDate" type="datetime-local" />
          </label>
          <label>
            {{ t('createEvent.maxPlayers') }}
            <input v-model.number="newEventMaxPlayers" min="2" max="99" type="number" />
          </label>
        </div>
      </div>

      <div class="create-form-section">
        <p class="create-form-section-kicker">{{ t('createEvent.registrationSection') }}</p>
        <label>
          {{ t('createEvent.signupVisibility') }}
          <select v-model="newEventSignupVisibility">
            <option value="private">{{ t('createEvent.visibilityPrivate') }}</option>
            <option value="public">{{ t('createEvent.visibilityPublic') }}</option>
          </select>
        </label>
        <div class="create-form-toggles">
          <label class="create-form-toggle-row">
            <span class="create-form-toggle-label">
              <DiscordIcon class="create-form-toggle-icon" />
              {{ t('createEvent.requireDiscord') }}
            </span>
            <span class="create-form-toggle-hint">{{ t('createEvent.requireDiscordHint') }}</span>
            <button
              type="button"
              role="switch"
              class="create-form-toggle-switch"
              :aria-checked="newEventRequireDiscord ? 'true' : 'false'"
              :class="{ 'is-on': newEventRequireDiscord }"
              @click="newEventRequireDiscord = !newEventRequireDiscord"
            >
              <span class="create-form-toggle-thumb" />
            </button>
          </label>
          <label class="create-form-toggle-row">
            <span class="create-form-toggle-label">
              <BnetIcon class="create-form-toggle-icon" />
              {{ t('createEvent.requireBnet') }}
            </span>
            <span class="create-form-toggle-hint">{{ t('createEvent.requireBnetHint') }}</span>
            <button
              type="button"
              role="switch"
              class="create-form-toggle-switch"
              :aria-checked="newEventRequireBattletag ? 'true' : 'false'"
              :class="{ 'is-on': newEventRequireBattletag }"
              @click="newEventRequireBattletag = !newEventRequireBattletag"
            >
              <span class="create-form-toggle-thumb" />
            </button>
          </label>
        </div>
      </div>

      <div class="create-form-section">
        <p class="create-form-section-kicker">{{ t('createEvent.discordSection') }}</p>
        <template v-if="hasGuild">
          <div class="create-form-toggles">
            <label class="create-form-toggle-row">
              <span class="create-form-toggle-label">
                <DiscordIcon class="create-form-toggle-icon" />
                {{ t('createEvent.discordAnnounce') }}
              </span>
              <span class="create-form-toggle-hint">{{ t('createEvent.discordAnnounceHint') }}</span>
              <button
                type="button"
                role="switch"
                class="create-form-toggle-switch"
                :aria-checked="newEventDiscordAnnounce ? 'true' : 'false'"
                :class="{ 'is-on': newEventDiscordAnnounce }"
                @click="newEventDiscordAnnounce = !newEventDiscordAnnounce"
              >
                <span class="create-form-toggle-thumb" />
              </button>
            </label>
          </div>
        </template>
        <p v-else class="create-no-guild-hint">
          <span class="material-symbols-rounded" style="font-size:1rem;flex-shrink:0" aria-hidden="true">info</span>
          {{ t('createEvent.noGuildHint') }}
          <router-link to="/discord" class="create-no-guild-link">{{ t('createEvent.addGuildLink') }}</router-link>
        </p>
      </div>

      <p v-if="error" class="status status-error">{{ error }}</p>

      <div class="create-form-actions">
        <ActionCtaButton type="submit" :disabled="!canCreate || creatingEvent">
          {{ creatingEvent ? t('createEvent.creating') : t('createEvent.createBtn') }}
        </ActionCtaButton>
        <button type="button" class="btn-secondary" :disabled="creatingEvent" @click="emit('update:open', false)">
          {{ t('createEvent.cancel') }}
        </button>
      </div>
    </form>
  </AppModal>
</template>

<style scoped>
.create-event-form {
  display: grid;
  gap: 1rem;
}

.create-form-section {
  display: grid;
  gap: 0.52rem;
  padding: 0.8rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
}

.create-form-section-kicker {
  margin: 0 0 0.18rem;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
}

.create-form-row-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.52rem;
}

.create-form-toggles {
  display: grid;
  gap: 0;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.create-form-toggle-row {
  display: grid;
  grid-template-columns: 1fr auto;
  grid-template-rows: auto auto;
  align-items: center;
  gap: 0.06rem 0.75rem;
  padding: 0.6rem 0.72rem;
  cursor: pointer;
  border-bottom: 1px solid var(--line);
  background: color-mix(in srgb, var(--card) 80%, var(--bg-1) 20%);
  transition: background 0.14s;
  font-weight: normal;
}

.create-form-toggle-row:last-child {
  border-bottom: none;
}

.create-form-toggle-row:hover {
  background: color-mix(in srgb, var(--card) 60%, var(--bg-1) 40%);
}

.create-form-toggle-label {
  grid-column: 1;
  grid-row: 1;
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--ink-1);
  line-height: 1.3;
  display: flex;
  align-items: center;
  gap: 0.38rem;
}

.create-form-toggle-icon {
  width: 0.95rem;
  height: 0.95rem;
  flex-shrink: 0;
  color: var(--ink-2);
  opacity: 0.72;
}

.create-form-toggle-hint {
  grid-column: 1;
  grid-row: 2;
  font-size: 0.76rem;
  color: var(--ink-muted);
  line-height: 1.4;
}

.create-form-toggle-switch {
  grid-column: 2;
  grid-row: 1 / 3;
  width: 2.4rem;
  height: 1.3rem;
  border-radius: 999px;
  background: var(--line);
  border: none;
  padding: 0;
  position: relative;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.18s;
  align-self: center;
}

.create-form-toggle-switch.is-on {
  background: var(--primary-500, #6366f1);
}

.create-form-toggle-thumb {
  position: absolute;
  top: 0.14rem;
  left: 0.14rem;
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
  transition: transform 0.18s;
  display: block;
  pointer-events: none;
}

.create-form-toggle-switch.is-on .create-form-toggle-thumb {
  transform: translateX(1.1rem);
}

.create-form-actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  flex-wrap: wrap;
}

.create-no-guild-hint {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 0.38rem;
  font-size: 0.82rem;
  color: var(--ink-muted);
  line-height: 1.4;
  flex-wrap: wrap;
}

.create-no-guild-link {
  color: var(--brand-1);
  text-decoration: none;
  font-weight: 500;
}

.create-no-guild-link:hover {
  text-decoration: underline;
}
</style>
