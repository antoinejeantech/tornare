<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { formatDayMonthYear, formatTime24, getDateTimestamp } from '../../lib/dates'
import AppBadge from '../ui/AppBadge.vue'

const props = defineProps({
  event: {
    type: Object,
    required: true,
  },
  to: {
    type: [String, Object],
    required: true,
  },
})

const startDateDisplay = computed(() => {
  return formatDayMonthYear(props.event?.start_date, '--/--/----')
})

const startTimeDisplay = computed(() => {
  return formatTime24(props.event?.start_date, '--:--')
})

const playerCount = computed(() => {
  return Array.isArray(props.event?.players) ? props.event.players.length : 0
})

const eventFormat = computed(() => {
  return props.event?.format || '5v5'
})

const maxPlayers = computed(() => {
  const value = Number(props.event?.max_players)
  return Number.isFinite(value) && value > 0 ? value : 0
})

const creatorProfileRoute = computed(() => {
  const creatorId = String(props.event?.creator_id || '').trim()
  if (!creatorId) {
    return null
  }

  return { name: 'profile', params: { id: creatorId } }
})

const statusLabel = computed(() => {
  if (props.event?.is_ended) {
    return 'Ended'
  }

  const maxPlayers = Number(props.event?.max_players) || 0
  const players = playerCount.value
  const startAt = getDateTimestamp(props.event?.start_date)

  if (maxPlayers > 0 && players >= maxPlayers) {
    return 'Full'
  }

  if (startAt !== null) {
    const now = Date.now()
    if (startAt <= now) {
      return 'Ongoing'
    }

    if (startAt - now <= 6 * 60 * 60 * 1000) {
      return 'Starting Soon'
    }
  }

  return 'Open'
})

const statusVariant = computed(() => {
  if (statusLabel.value === 'Ended') return 'muted'
  if (statusLabel.value === 'Full') return 'danger'
  if (statusLabel.value === 'Ongoing') return 'info'
  return 'ok'
})

</script>

<template>
  <li class="event-list-item">
    <div class="event-list-main">
      <span class="event-list-title-wrap">
        <span class="material-symbols-rounded event-list-trophy" aria-hidden="true">trophy</span>
        <span class="event-list-text-block">
          <RouterLink class="event-list-title-link" :to="to">
            <span class="event-list-title">{{ event.name }}</span>
          </RouterLink>
          <span class="muted event-list-meta-row">
            by
            <RouterLink v-if="creatorProfileRoute" class="event-creator-link" :to="creatorProfileRoute">
              {{ event.creator_name || 'Unknown' }}
            </RouterLink>
            <span v-else>{{ event.creator_name || 'Unknown' }}</span>
          </span>
        </span>
      </span>
    </div>

    <div class="event-format-col" aria-label="Event format">
      <span class="event-col-label muted">Format</span>
      <strong class="event-format-value">{{ event.event_type || 'PUG' }} ({{ eventFormat }})</strong>
    </div>

    <div class="event-players-col" aria-label="Players">
      <span class="material-symbols-rounded" aria-hidden="true">group</span>
      <strong>{{ playerCount }}/{{ maxPlayers || event.max_players }}</strong>
    </div>

    <div class="event-date-col" aria-label="Date and time">
      <span class="event-col-label muted">Date &amp; Time</span>
      <strong class="event-date-value">
        <span>{{ startDateDisplay }}</span>
        <span class="event-date-dot material-symbols-rounded" aria-hidden="true">fiber_manual_record</span>
        <span>{{ startTimeDisplay }}</span>
      </strong>
    </div>

    <div class="event-actions-col" aria-label="Status and actions">
      <AppBadge :variant="statusVariant" :label="statusLabel" />
      <RouterLink class="event-details-btn" :to="to">Details</RouterLink>
    </div>
  </li>
</template>

<style scoped>
.event-list-item {
  border: 1px solid color-mix(in srgb, var(--line-strong) 58%, var(--bg-0) 42%);
  background: color-mix(in srgb, var(--card) 62%, var(--bg-1) 38%);
  border-radius: var(--radius-md);
  padding: 1.02rem 0.95rem;
  display: grid;
  grid-template-columns: minmax(0, 2.2fr) minmax(0, 0.85fr) minmax(0, 0.95fr) minmax(0, 1.3fr) auto;
  align-items: center;
  gap: 0.85rem;
}

.event-list-main {
  display: grid;
  gap: 0;
  min-width: 0;
}

.event-list-title-link {
  text-decoration: none;
  min-width: 0;
}

.event-list-text-block {
  display: grid;
  gap: 0.14rem;
  min-width: 0;
}

.event-list-meta-row {
  font-size: 0.82rem;
  line-height: 1.18;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-creator-link {
  margin-left: 0.22rem;
  color: var(--brand-1);
  text-decoration: none;
}

.event-list-title-link:hover .event-list-title {
  color: var(--brand-1);
}

.event-list-title {
  font-weight: 600;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.02em;
  font-size: 0.9rem;
}

.event-list-title-wrap {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 0.5rem;
}

.event-list-trophy {
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
  font-size: 1.35rem;
  border: 1px solid color-mix(in srgb, var(--line-strong) 58%, var(--bg-0) 42%);
  border-radius: var(--radius-md);
  padding: 0.4rem;
  background: color-mix(in srgb, var(--bg-1) 66%, var(--card) 34%);
}

.event-format-col {
  display: grid;
  justify-items: center;
  gap: 0.16rem;
  min-width: 0;
  width: 100%;
  padding: 0.16rem 0.32rem;
  border-radius: var(--radius-sm);
}

.event-col-label {
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
}

.event-format-value {
  font-size: 0.82rem;
  line-height: 1;
  color: color-mix(in srgb, var(--brand-1) 86%, #ffe08f 14%);
}

.event-players-col {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  min-width: 0;
  width: 100%;
  justify-content: center;
  padding: 0.2rem 0.36rem;
  border-radius: var(--radius-sm);
  color: var(--ink-1);
  font-size: 0.82rem;
  font-weight: 760;
}

.event-players-col .material-symbols-rounded {
  font-size: 1rem;
  color: var(--ink-muted) !important;
}

.event-date-col {
  display: grid;
  justify-items: center;
  gap: 0.16rem;
  min-width: 0;
  width: 100%;
}

.event-date-value {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
  color: var(--ink-2);
  font-size: 0.84rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-date-value .event-date-dot.material-symbols-rounded {
  font-size: 0.5rem;
  color: var(--ink-muted) !important;
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 24;
}

.event-actions-col {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.46rem;
  min-width: 0;
  flex-shrink: 0;
}

.event-details-btn {
  border: 1px solid color-mix(in srgb, var(--line-strong) 82%, white 18%);
  background: color-mix(in srgb, var(--grey-900) 74%, black 26%);
  color: white;
  border-radius: var(--radius-sm);
  text-decoration: none;
  padding: 0.28rem 0.56rem;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.event-details-btn:hover {
  border-color: color-mix(in srgb, var(--line-strong) 72%, white 28%);
  background: color-mix(in srgb, var(--grey-900) 68%, black 32%);
  color: white;
}

@media (max-width: 920px) {
  .event-list-item {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  }

  .event-format-col {
    display: none;
  }

  .event-date-col {
    display: none;
  }

  .event-actions-col {
    min-width: 98px;
  }
}
</style>
