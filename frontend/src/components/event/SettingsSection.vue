<script setup>
import { inject } from 'vue'
import { formatOptionsForType } from '../../lib/event-format'
import EventSectionHeader from './EventSectionHeader.vue'

const ctx = inject('eventCtx')
</script>

<template>
  <section class="event-settings-section">
    <div class="section-heading-block">
      <EventSectionHeader icon="settings" title="Settings" />
    </div>

    <div class="event-registration-toggle-box" :class="ctx.event.public_signup_enabled ? 'is-public' : 'is-private'">
      <div class="event-registration-header">
        <p class="event-registration-kicker">Event registration</p>
        <span class="event-registration-state-pill" :class="ctx.event.public_signup_enabled ? 'is-public' : 'is-private'">
          {{ ctx.event.public_signup_enabled ? 'Public' : 'Private' }}
        </span>
      </div>

      <p class="event-registration-copy">
        {{ ctx.event.public_signup_enabled
          ? 'Anyone can discover this event and use the Join button from event surfaces.'
          : 'Only people with a direct invite link can submit a signup request.' }}
      </p>

      <div class="event-registration-toggle-actions">
        <button
          class="btn-secondary"
          :disabled="ctx.updatingSignupVisibility"
          @click="ctx.setSignupVisibility(!ctx.event.public_signup_enabled)"
        >
          {{ ctx.updatingSignupVisibility ? 'Updating...' : (ctx.event.public_signup_enabled ? 'Make private' : 'Make public') }}
        </button>
      </div>

      <p class="muted event-registration-note">
        {{ ctx.event.public_signup_enabled
          ? 'Switching to private hides the public Join button and rotates the signup token. Existing shared links stop working.'
          : 'Switch to public to show the Join button to everyone.' }}
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
            v-for="format in formatOptionsForType(ctx.event.event_type)"
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

.event-registration-state-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  padding: 0.22rem 0.62rem;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border: 1px solid transparent;
}

.event-registration-state-pill.is-public {
  color: #edfdf4;
  background: color-mix(in srgb, #1c7a4f 82%, #0f2d1f 18%);
  border-color: color-mix(in srgb, #36b376 58%, #0f2d1f 42%);
}

.event-registration-state-pill.is-private {
  color: #fff3f1;
  background: color-mix(in srgb, #8f3427 84%, #2c1411 16%);
  border-color: color-mix(in srgb, #d96a57 56%, #2c1411 44%);
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
}
</style>
