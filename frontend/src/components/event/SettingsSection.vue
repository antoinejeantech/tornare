<script setup lang="ts">
import { inject } from 'vue'
import { formatOptionsForType } from '../../lib/event-format'
import EventSectionHeader from './EventSectionHeader.vue'
import AppBadge from '../ui/AppBadge.vue'
import DiscordIcon from '../ui/DiscordIcon.vue'
import BnetIcon from '../ui/BnetIcon.vue'
import type { EventCtxType } from '../../composables/event/event-inject'

const ctx = inject<EventCtxType>('eventCtx')!
</script>

<template>
  <section class="event-settings-section">
    <div class="section-heading-block">
      <EventSectionHeader icon="settings" title="Settings" />
    </div>

    <div class="event-registration-toggle-box" :class="ctx.event?.public_signup_enabled ? 'is-public' : 'is-private'">
      <div class="event-registration-header">
        <p class="event-registration-kicker">Event registration</p>
        <AppBadge
          :variant="ctx.event?.public_signup_enabled ? 'ok' : 'danger'"
          :label="ctx.event?.public_signup_enabled ? 'Public' : 'Private'"
        />
      </div>

      <p class="event-registration-copy">
        {{ ctx.event?.public_signup_enabled
          ? 'Anyone can discover this event and use the Join button from event surfaces.'
          : 'Only people with a direct invite link can submit a signup request.' }}
      </p>

      <div class="event-registration-toggle-actions">
        <button
          class="btn-secondary"
          :disabled="ctx.updatingSignupVisibility"
          @click="ctx.setSignupVisibility(!ctx.event?.public_signup_enabled)"
        >
          {{ ctx.updatingSignupVisibility ? 'Updating...' : (ctx.event?.public_signup_enabled ? 'Make private' : 'Make public') }}
        </button>
      </div>

      <p class="muted event-registration-note">
        {{ ctx.event?.public_signup_enabled
          ? 'Switching to private hides the public Join button and rotates the signup token. Existing shared links stop working.'
          : 'Switch to public to show the Join button to everyone.' }}
      </p>
    </div>

    <div class="event-status-box" :class="`is-${(ctx.event?.status ?? 'ACTIVE').toLowerCase()}`">
      <div class="event-status-header">
        <p class="event-status-kicker">Event status</p>
        <AppBadge
          :variant="ctx.event?.status === 'ENDED' ? 'muted' : ctx.event?.status === 'DRAFT' ? 'warning' : 'ok'"
          :label="ctx.event?.status === 'ENDED' ? 'Ended' : ctx.event?.status === 'DRAFT' ? 'Draft' : 'Active'"
        />
      </div>

      <p class="event-status-copy">
        <template v-if="ctx.event?.status === 'DRAFT'">This event is a draft and is not visible in public listings. Registrations are disabled.</template>
        <template v-else-if="ctx.event?.status === 'ENDED'">This event has ended. It is visible in public listings under Past Events.</template>
        <template v-else>This event is active and visible in public listings. Registrations are controlled separately.</template>
      </p>

      <div class="event-status-actions">
        <button
          v-if="ctx.event?.status !== 'DRAFT'"
          class="btn-secondary"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.unpublishEvent()"
        >
          {{ ctx.updatingEventStatus ? 'Updating...' : 'Set as Draft' }}
        </button>
        <button
          v-if="ctx.event?.status !== 'ACTIVE'"
          class="btn-primary"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.publishEvent()"
        >
          {{ ctx.updatingEventStatus ? 'Updating...' : 'Set as Active' }}
        </button>
        <button
          v-if="ctx.event?.status !== 'ENDED'"
          class="btn-warning"
          :disabled="ctx.updatingEventStatus"
          type="button"
          @click="ctx.endEvent()"
        >
          {{ ctx.updatingEventStatus ? 'Updating...' : 'End event' }}
        </button>
      </div>

      <p class="muted event-status-note">
        <template v-if="ctx.event?.status === 'DRAFT'">Draft events are invisible to everyone except you. Set to Active when you are ready to go live.</template>
        <template v-else-if="ctx.event?.status === 'ENDED'">Ended events remain visible in listings under Past Events. You can reopen them at any time.</template>
        <template v-else>Active events are visible to everyone. End the event when it is over, or move it back to Draft to hide it.</template>
      </p>
    </div>

    <form class="event-edit-form" @submit.prevent="ctx.saveEventEdit">
      <label>
        Event name
        <input v-model="ctx.editEventName" placeholder="Event name" />
      </label>
      <label>
        Description
        <textarea v-model="ctx.editEventDescription" rows="4" placeholder="Rules, cashprize, check-in info..." />
      </label>
      <label>
        Start date
        <input v-model="ctx.editEventStartDate" type="datetime-local" />
      </label>
      <label>
        Format
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
      <label>
        Max players
        <input v-model.number="ctx.editEventMaxPlayers" type="number" min="2" max="99" step="1" />
      </label>

      <fieldset class="event-handle-requirements">
        <legend class="event-handle-requirements-legend">Signup requirements</legend>

        <label class="event-toggle-row">
          <span class="event-toggle-row-label"><DiscordIcon class="event-toggle-row-icon" />Require Discord username</span>
          <span class="event-toggle-row-hint">Submissions without a Discord handle will be rejected</span>
          <button
            type="button"
            role="switch"
            class="event-toggle-switch"
            :aria-checked="ctx.editEventRequireDiscord ? 'true' : 'false'"
            :class="{ 'is-on': ctx.editEventRequireDiscord }"
            @click="ctx.editEventRequireDiscord = !ctx.editEventRequireDiscord"
          >
            <span class="event-toggle-switch-thumb" />
          </button>
        </label>

        <label class="event-toggle-row">
          <span class="event-toggle-row-label"><BnetIcon class="event-toggle-row-icon" />Require Battle.net tag</span>
          <span class="event-toggle-row-hint">Submissions without a Battle.net tag will be rejected</span>
          <button
            type="button"
            role="switch"
            class="event-toggle-switch"
            :aria-checked="ctx.editEventRequireBattletag ? 'true' : 'false'"
            :class="{ 'is-on': ctx.editEventRequireBattletag }"
            @click="ctx.editEventRequireBattletag = !ctx.editEventRequireBattletag"
          >
            <span class="event-toggle-switch-thumb" />
          </button>
        </label>
      </fieldset>

      <div class="event-settings-actions">
        <button class="btn-primary" :disabled="ctx.updatingEvent || !ctx.canSaveEventMeta" type="submit">
          {{ ctx.updatingEvent ? 'Saving...' : 'Save event settings' }}
        </button>
        <button class="btn-secondary" :disabled="ctx.updatingEvent" type="button" @click="ctx.syncEventEditDraftFromEvent">
          Reset changes
        </button>
        <button class="btn-danger" :disabled="ctx.deletingEvent || ctx.updatingEvent" type="button" @click="ctx.deleteEvent">
          {{ ctx.deletingEvent ? 'Deleting event...' : 'Delete event' }}
        </button>
      </div>
    </form>
  </section>
</template>

<style scoped>
.event-edit-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 0.5rem;
  margin: 0.55rem 0 0.7rem;
}

.event-edit-form label {
  display: grid;
  gap: 0.24rem;
}

.event-handle-requirements {
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  padding: 0.7rem 0.82rem;
  display: grid;
  gap: 0;
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
}

.event-handle-requirements-legend {
  font-size: 0.78rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
  padding: 0 0.2rem;
  margin-bottom: 0.55rem;
}

.event-toggle-row {
  display: grid;
  grid-template-columns: 1fr auto;
  grid-template-rows: auto auto;
  align-items: center;
  gap: 0.08rem 0.75rem;
  padding: 0.52rem 0;
  cursor: pointer;
  border-bottom: 1px solid var(--line);
}

.event-toggle-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.event-toggle-row:first-of-type {
  padding-top: 0;
}

.event-toggle-row-label {
  grid-column: 1;
  grid-row: 1;
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--ink-1);
  line-height: 1.3;
  display: flex;
  align-items: center;
  gap: 0.38rem;
}

.event-toggle-row-icon {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
  color: var(--ink-2);
  opacity: 0.75;
}

.event-toggle-row-hint {
  grid-column: 1;
  grid-row: 2;
  font-size: 0.78rem;
  color: var(--ink-muted);
  line-height: 1.4;
}

.event-toggle-switch {
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

.event-toggle-switch.is-on {
  background: var(--primary-500, #6366f1);
}

.event-toggle-switch-thumb {
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

.event-toggle-switch.is-on .event-toggle-switch-thumb {
  transform: translateX(1.1rem);
}

.event-settings-section {
  display: grid;
  gap: 0;
}

.event-registration-toggle-box {
  border: 1px solid color-mix(in srgb, var(--line) 72%, var(--brand-2) 28%);
  border-radius: var(--radius-lg);
  padding: 0.82rem;
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
  display: grid;
  gap: 0.62rem;
  box-shadow: none;
}

.event-registration-toggle-box.is-private {
  border-color: color-mix(in srgb, #e36b55 26%, var(--line) 74%);
  background: color-mix(in srgb, var(--card) 66%, #3a1f1a 34%);
}

.event-registration-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.event-registration-kicker {
  margin: 0;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
}

.event-registration-copy {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-1);
}

.event-registration-toggle-actions {
  display: flex;
  justify-content: flex-start;
}

.event-registration-note {
  margin: 0;
  font-size: 0.84rem;
  line-height: 1.35;
}

.event-settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.45rem;
}

.event-status-box {
  border: 1px solid color-mix(in srgb, var(--line) 72%, var(--brand-2) 28%);
  border-radius: var(--radius-lg);
  padding: 0.82rem;
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
  display: grid;
  gap: 0.62rem;
  margin-top: 0.75rem;
}

.event-status-box.is-ended {
  border-color: color-mix(in srgb, var(--line) 72%, #555 28%);
  background: color-mix(in srgb, var(--card) 56%, #1e1e1e 44%);
}

.event-status-box.is-active {
  border-color: color-mix(in srgb, #4ca84c 22%, var(--line) 78%);
  background: color-mix(in srgb, var(--card) 72%, #0e1f0e 28%);
}

.event-status-box.is-draft {
  border-color: color-mix(in srgb, #e09c2a 22%, var(--line) 78%);
  background: color-mix(in srgb, var(--card) 72%, #27200a 28%);
}

.event-status-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.event-status-kicker {
  margin: 0;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
  color: color-mix(in srgb, var(--ink-2) 82%, var(--brand-1) 18%);
}

.event-status-copy {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-1);
}

.event-status-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.event-status-note {
  margin: 0;
  font-size: 0.84rem;
  line-height: 1.35;
}

@media (max-width: 720px) {
  .event-registration-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .event-registration-toggle-actions,
  .event-registration-toggle-actions button {
    width: 100%;
  }

  .event-settings-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .event-settings-actions button {
    width: 100%;
  }

  .event-status-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .event-status-actions,
  .event-status-actions button {
    width: 100%;
  }
}
</style>
