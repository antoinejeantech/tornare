<script setup lang="ts">
import { inject, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatOptionsForType } from '../../lib/event-format'
import EventSectionHeader from './EventSectionHeader.vue'
import AppBadge from '../ui/AppBadge.vue'
import DiscordIcon from '../ui/DiscordIcon.vue'
import BnetIcon from '../ui/BnetIcon.vue'
import type { EventCtxType } from '../../composables/event/event-inject'
import { getDiscordGuilds } from '../../lib/api'

const ctx = inject<EventCtxType>('eventCtx')!
const { t } = useI18n()

const hasGuild = ref(false)
onMounted(async () => {
  try {
    const guilds = await getDiscordGuilds()
    hasGuild.value = guilds.length > 0
  } catch {
    // silent: toggle stays hidden if fetch fails
  }
})
</script>

<template>
  <section class="settings-section">
    <div class="section-heading-block">
      <EventSectionHeader icon="settings" :title="t('settings.sectionTitle')" />
    </div>

    <!-- ── Registration card ─────────────────────────────────────────── -->
    <div class="settings-card" :class="ctx.event?.public_signup_enabled ? 'card-ok' : 'card-danger'">
      <div class="card-header">
        <span class="card-label">{{ t('settings.registrationKicker') }}</span>
        <AppBadge
          :variant="ctx.event?.public_signup_enabled ? 'ok' : 'danger'"
          :label="ctx.event?.public_signup_enabled ? t('settings.publicLabel') : t('settings.privateLabel')"
        />
      </div>
      <p class="card-body">
        {{ ctx.event?.public_signup_enabled ? t('settings.publicDesc') : t('settings.privateDesc') }}
      </p>
      <div class="card-actions">
        <button
          class="btn-secondary"
          :disabled="ctx.updatingSignupVisibility"
          @click="ctx.setSignupVisibility(!ctx.event?.public_signup_enabled)"
        >
          <span class="material-symbols-rounded" aria-hidden="true">{{ ctx.event?.public_signup_enabled ? 'lock' : 'lock_open' }}</span>
          {{ ctx.updatingSignupVisibility
            ? t('settings.updatingVisibility')
            : ctx.event?.public_signup_enabled ? t('settings.makePrivate') : t('settings.makePublic') }}
        </button>
      </div>
      <p class="card-note muted">
        {{ ctx.event?.public_signup_enabled ? t('settings.switchToPrivateHint') : t('settings.switchToPublicHint') }}
      </p>
    </div>

    <!-- ── Status card ─────────────────────────────────────────────────── -->
    <div class="settings-card" :class="`card-status-${(ctx.event?.status ?? 'ACTIVE').toLowerCase()}`">
      <div class="card-header">
        <span class="card-label">{{ t('settings.statusKicker') }}</span>
        <AppBadge
          :variant="ctx.event?.status === 'ENDED' ? 'muted' : ctx.event?.status === 'DRAFT' ? 'warning' : 'ok'"
          :label="ctx.event?.status === 'ENDED' ? t('common.statusEnded') : ctx.event?.status === 'DRAFT' ? t('common.statusDraft') : t('common.statusActive')"
        />
      </div>
      <p class="card-body">
        <template v-if="ctx.event?.status === 'DRAFT'">{{ t('settings.draftDesc') }}</template>
        <template v-else-if="ctx.event?.status === 'ENDED'">{{ t('settings.endedDesc') }}</template>
        <template v-else>{{ t('settings.activeDesc') }}</template>
      </p>
      <div class="card-actions">
        <button
          v-if="ctx.event?.status === 'ACTIVE'"
          class="btn-secondary"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.unpublishEvent()"
        >
          <span class="material-symbols-rounded" aria-hidden="true">draft</span>
          {{ ctx.updatingEventStatus ? t('settings.updating') : t('settings.setDraft') }}
        </button>
        <button
          v-if="ctx.event?.status === 'DRAFT'"
          class="btn-primary"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.publishEvent()"
        >
          <span class="material-symbols-rounded" aria-hidden="true">rocket_launch</span>
          {{ ctx.updatingEventStatus ? t('settings.updating') : t('settings.setActive') }}
        </button>
        <button
          v-if="ctx.event?.status === 'ACTIVE'"
          class="btn-warning"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.endEvent()"
        >
          <span class="material-symbols-rounded" aria-hidden="true">sports_score</span>
          {{ ctx.updatingEventStatus ? t('settings.updating') : t('settings.endEvent') }}
        </button>
      </div>
      <p class="card-note muted">
        <template v-if="ctx.event?.status === 'DRAFT'">{{ t('settings.draftNote') }}</template>
        <template v-else-if="ctx.event?.status === 'ENDED'">{{ t('settings.endedNote') }}</template>
        <template v-else>{{ t('settings.activeNote') }}</template>
      </p>
    </div>

    <!-- ── Edit form card ──────────────────────────────────────────────── -->
    <form class="settings-card settings-form" @submit.prevent="ctx.saveEventEdit">
      <p class="card-label">{{ t('settings.eventDetailsLabel') }}</p>

      <!-- Basic info section -->
      <div class="form-section">
        <label class="field">
          <span class="field-label">{{ t('settings.eventName') }}</span>
          <input v-model="ctx.editEventName" :placeholder="t('settings.eventNamePlaceholder')" />
        </label>
        <label class="field">
          <span class="field-label">{{ t('settings.description') }}</span>
          <textarea v-model="ctx.editEventDescription" rows="3" :placeholder="t('settings.descriptionPlaceholder')" />
        </label>
      </div>

      <!-- Format section -->
      <div class="form-section">
        <p class="form-section-kicker">{{ t('settings.formatSection') }}</p>
        <label class="field">
          <span class="field-label">{{ t('settings.startDate') }}</span>
          <input v-model="ctx.editEventStartDate" type="datetime-local" />
        </label>
        <div class="field-row">
          <label class="field">
            <span class="field-label">{{ t('settings.formatLabel') }}</span>
            <select v-model="ctx.editEventFormat">
              <option
                v-for="format in formatOptionsForType(ctx.event?.event_type || 'PUG')"
                :key="`edit-event-format-${format}`"
                :value="format"
              >
                {{ format }}
              </option>
            </select>
          </label>
          <label class="field">
            <span class="field-label">{{ t('settings.maxPlayers') }}</span>
            <input v-model.number="ctx.editEventMaxPlayers" type="number" min="2" max="99" step="1" />
          </label>
        </div>
      </div>

      <!-- Signup requirements section -->
      <div class="form-section">
        <p class="form-section-kicker">{{ t('settings.signupRequirements') }}</p>
        <div class="toggle-list">
          <label class="toggle-row">
            <span class="toggle-row-content">
              <span class="toggle-row-label"><DiscordIcon class="toggle-icon" />{{ t('settings.requireDiscord') }}</span>
              <span class="toggle-row-hint">{{ t('settings.requireDiscordHint') }}</span>
            </span>
            <button
              type="button" role="switch"
              class="toggle-switch"
              :aria-checked="ctx.editEventRequireDiscord ? 'true' : 'false'"
              :class="{ 'is-on': ctx.editEventRequireDiscord }"
              @click="ctx.editEventRequireDiscord = !ctx.editEventRequireDiscord"
            ><span class="toggle-thumb" /></button>
          </label>
          <label class="toggle-row">
            <span class="toggle-row-content">
              <span class="toggle-row-label"><BnetIcon class="toggle-icon" />{{ t('settings.requireBnet') }}</span>
              <span class="toggle-row-hint">{{ t('settings.requireBnetHint') }}</span>
            </span>
            <button
              type="button" role="switch"
              class="toggle-switch"
              :aria-checked="ctx.editEventRequireBattletag ? 'true' : 'false'"
              :class="{ 'is-on': ctx.editEventRequireBattletag }"
              @click="ctx.editEventRequireBattletag = !ctx.editEventRequireBattletag"
            ><span class="toggle-thumb" /></button>
          </label>
        </div>
      </div>

      <!-- Discord section -->
      <div class="form-section">
        <p class="form-section-kicker">{{ t('settings.discordSection') }}</p>
        <template v-if="hasGuild">
          <div class="toggle-list">
            <label class="toggle-row">
              <span class="toggle-row-content">
                <span class="toggle-row-label"><DiscordIcon class="toggle-icon" />{{ t('settings.discordAnnounce') }}</span>
                <span class="toggle-row-hint">{{ t('settings.discordAnnounceHint') }}</span>
              </span>
              <button
                type="button" role="switch"
                class="toggle-switch"
                :aria-checked="ctx.editEventDiscordAnnounce ? 'true' : 'false'"
                :class="{ 'is-on': ctx.editEventDiscordAnnounce }"
                @click="ctx.editEventDiscordAnnounce = !ctx.editEventDiscordAnnounce"
              ><span class="toggle-thumb" /></button>
            </label>
          </div>
        </template>
        <p v-else class="no-guild-hint">
          <span class="material-symbols-rounded no-guild-icon" aria-hidden="true">info</span>
          {{ t('settings.noGuildHint') }}
          <router-link to="/discord" class="no-guild-link">{{ t('settings.addGuildLink') }}</router-link>
        </p>
      </div>

      <div class="form-actions">
        <button class="btn-primary" :disabled="ctx.updatingEvent || !ctx.canSaveEventMeta" type="submit">
          <span class="material-symbols-rounded" aria-hidden="true">{{ ctx.updatingEvent ? 'hourglass_empty' : 'save' }}</span>
          {{ ctx.updatingEvent ? t('settings.savingSettings') : t('settings.saveSettings') }}
        </button>
        <button class="btn-secondary" :disabled="ctx.updatingEvent" type="button" @click="ctx.syncEventEditDraftFromEvent">
          <span class="material-symbols-rounded" aria-hidden="true">restart_alt</span>
          {{ t('settings.resetChanges') }}
        </button>
        <button class="btn-danger" :disabled="ctx.deletingEvent || ctx.updatingEvent" type="button" @click="ctx.deleteEvent">
          <span class="material-symbols-rounded" aria-hidden="true">{{ ctx.deletingEvent ? 'hourglass_empty' : 'delete' }}</span>
          {{ ctx.deletingEvent ? t('settings.deletingEvent') : t('settings.deleteEvent') }}
        </button>
      </div>
    </form>
  </section>
</template>

<style scoped>
/* ── Layout ──────────────────────────────────────────────────────────── */
.settings-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

/* ── Cards ──────────────────────────────────────────────────────────── */
.settings-card {
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  padding: 1rem;
  background: color-mix(in srgb, var(--card) 65%, var(--bg-1) 35%);
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.settings-card.card-ok {
  border-color: color-mix(in srgb, #4ca84c 24%, var(--line) 76%);
  background: color-mix(in srgb, var(--card) 72%, #0e1f0e 28%);
}

.settings-card.card-danger {
  border-color: color-mix(in srgb, #e36b55 24%, var(--line) 76%);
  background: color-mix(in srgb, var(--card) 68%, #3a1f1a 32%);
}

.settings-card.card-status-active {
  border-color: color-mix(in srgb, #4ca84c 24%, var(--line) 76%);
  background: color-mix(in srgb, var(--card) 72%, #0e1f0e 28%);
}

.settings-card.card-status-draft {
  border-color: color-mix(in srgb, #e09c2a 24%, var(--line) 76%);
  background: color-mix(in srgb, var(--card) 72%, #27200a 28%);
}

.settings-card.card-status-ended {
  border-color: color-mix(in srgb, var(--line) 72%, #555 28%);
  background: color-mix(in srgb, var(--card) 56%, #1e1e1e 44%);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.card-label {
  font-size: 0.78rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--ink-2) 80%, var(--brand-1) 20%);
}

.card-body {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-1);
  line-height: 1.45;
}

.card-actions {
  display: flex;
  gap: 0.45rem;
  flex-wrap: wrap;
}

.card-note {
  margin: 0;
  font-size: 0.82rem;
  line-height: 1.4;
}

/* ── Form card ───────────────────────────────────────────────────────── */
.settings-form {
  gap: 0.65rem;
}

.form-section {
  display: grid;
  gap: 0.52rem;
  padding: 0.8rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
}

.form-section-kicker {
  margin: 0 0 0.1rem;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.22rem;
}

.field-label {
  font-size: 0.84rem;
  font-weight: 500;
  color: var(--ink-2);
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.52rem;
}

/* ── Toggle list (inside form-section) ───────────────────────────────── */
.toggle-list {
  display: grid;
  gap: 0;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.6rem 0.72rem;
  border-bottom: 1px solid var(--line);
  cursor: pointer;
  background: color-mix(in srgb, var(--card) 80%, var(--bg-1) 20%);
  transition: background 0.14s;
}

.toggle-row:hover {
  background: color-mix(in srgb, var(--card) 60%, var(--bg-1) 40%);
}

.toggle-row:last-child {
  border-bottom: none;
}

.toggle-row-content {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
}

.toggle-row-label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--ink-1);
  display: flex;
  align-items: center;
  gap: 0.38rem;
}

.toggle-icon {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
  color: var(--ink-2);
  opacity: 0.75;
}

.toggle-row-hint {
  font-size: 0.78rem;
  color: var(--ink-muted);
  line-height: 1.35;
}

/* ── Toggle switch ───────────────────────────────────────────────────── */
.toggle-switch {
  width: 2.4rem;
  height: 1.3rem;
  min-width: 2.4rem;
  border-radius: 999px;
  background: var(--line);
  border: none;
  padding: 0;
  position: relative;
  cursor: pointer;
  transition: background 0.18s;
  align-self: center;
}

.toggle-switch.is-on {
  background: var(--primary-500, #6366f1);
}

.toggle-thumb {
  position: absolute;
  top: 0.14rem;
  left: 0.14rem;
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0,0,0,0.25);
  transition: transform 0.18s;
  display: block;
  pointer-events: none;
}

.toggle-switch.is-on .toggle-thumb {
  transform: translateX(1.1rem);
}

/* ── Button icons ────────────────────────────────────────────────────── */
.card-actions button .material-symbols-rounded,
.form-actions button .material-symbols-rounded {
  font-size: 1rem;
}

/* ── No-guild hint ───────────────────────────────────────────────────── */
.no-guild-hint {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 0.38rem;
  font-size: 0.84rem;
  color: var(--ink-muted);
  line-height: 1.4;
  flex-wrap: wrap;
}

.no-guild-icon {
  font-size: 1rem;
  flex-shrink: 0;
}

.no-guild-link {
  color: var(--brand-1);
  text-decoration: none;
  font-weight: 500;
}

.no-guild-link:hover {
  text-decoration: underline;
}

/* ── Form actions ────────────────────────────────────────────────────── */
.form-actions {
  display: flex;
  gap: 0.45rem;
  flex-wrap: wrap;
  justify-content: flex-end;
}

/* ── Mobile ──────────────────────────────────────────────────────────── */
@media (max-width: 600px) {
  .settings-card {
    padding: 0.85rem;
  }

  .card-actions,
  .card-actions button {
    width: 100%;
  }

  .form-section {
    padding: 0.65rem;
  }

  .field-row {
    grid-template-columns: 1fr;
  }

  .form-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .form-actions button {
    width: 100%;
  }
}
</style>
