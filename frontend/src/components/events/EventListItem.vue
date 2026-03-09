<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import overwatchLogo from '../../assets/branding/overwatch-logo-gold.png'
import { formatEventStartDate } from '../../lib/dates'

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

const formattedStartDate = computed(() => {
  return formatEventStartDate(props.event?.start_date) || ''
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
  const maxPlayers = Number(props.event?.max_players) || 0
  const players = playerCount.value
  const startAt = props.event?.start_date ? new Date(props.event.start_date).getTime() : null

  if (maxPlayers > 0 && players >= maxPlayers) {
    return 'Full'
  }

  if (startAt && !Number.isNaN(startAt)) {
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

const statusClass = computed(() => {
  if (statusLabel.value === 'Full') {
    return 'is-full'
  }
  if (statusLabel.value === 'Ongoing') {
    return 'is-ongoing'
  }
  if (statusLabel.value === 'Starting Soon') {
    return 'is-soon'
  }

  return 'is-open'
})

</script>

<template>
  <li class="event-list-item">
    <div class="event-list-main">
      <RouterLink class="event-list-title-link" :to="to">
        <span class="event-list-title-wrap">
          <img class="event-list-logo" :src="overwatchLogo" alt="Overwatch logo" />
          <span class="event-list-title">{{ event.name }}</span>
        </span>
      </RouterLink>
      <span class="muted event-list-meta-row">
        by
        <RouterLink v-if="creatorProfileRoute" class="event-creator-link" :to="creatorProfileRoute">
          {{ event.creator_name || 'Unknown' }}
        </RouterLink>
        <span v-else>{{ event.creator_name || 'Unknown' }}</span>
      </span>
    </div>

    <div class="event-format-col" aria-label="Event format">
      <strong class="event-format-value">{{ event.event_type || 'PUG' }} ({{ eventFormat }})</strong>
    </div>

    <div class="event-players-col" aria-label="Players">
      <span class="material-symbols-rounded" aria-hidden="true">group</span>
      <strong>{{ playerCount }}/{{ maxPlayers || event.max_players }}</strong>
    </div>

    <div class="event-date-col" aria-label="Start date">
      <strong>{{ formattedStartDate || 'No date' }}</strong>
    </div>

    <div class="event-status-col" aria-label="Status">
      <span class="event-status-chip" :class="statusClass">{{ statusLabel }}</span>
    </div>

    <div class="event-list-side">
      <RouterLink class="event-details-btn" :to="to">Details</RouterLink>
    </div>
  </li>
</template>

<style scoped>
.event-list-item {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.54rem 0.62rem;
  display: grid;
  grid-template-columns: minmax(0, 2.1fr) minmax(0, 0.8fr) minmax(0, 0.9fr) minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.62rem;
}

.event-list-main {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.event-list-title-link {
  text-decoration: none;
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

.event-creator-link:hover {
  text-decoration: underline;
}

.event-list-title-link:hover .event-list-title {
  color: var(--brand-1);
}

.event-list-title {
  font-weight: 680;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.02em;
  font-size: 0.9rem;
}

.event-list-title-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.34rem;
}

.event-list-logo {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex: 0 0 auto;
}

.event-status-chip {
  border-radius: 999px;
  padding: 0.2rem 0.62rem;
  font-size: 0.74rem;
  font-weight: 620;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border: 1px solid transparent;
}

.event-status-chip.is-open {
  color: #9ce9b8;
  background: #123224;
  border-color: #2e7a4f;
}

.event-status-chip.is-soon {
  color: #7a3b00;
  background: #ffe8c9;
  border-color: #ffc57f;
}

.event-status-chip.is-full {
  color: #ffb9a2;
  background: #3c1b16;
  border-color: #8b4433;
}

.event-status-chip.is-ongoing {
  color: #c4dcff;
  background: #1c2f4b;
  border-color: #3f5f8d;
}

.event-list-side {
  display: inline-flex;
  align-items: center;
  gap: 0.36rem;
  flex-shrink: 0;
}

.event-format-col {
  display: grid;
  justify-items: center;
  gap: 0.06rem;
  min-width: 0;
  width: 100%;
  padding: 0.16rem 0.32rem;
  border-radius: 8px;
}

.event-format-value {
  font-size: 0.88rem;
  line-height: 1;
  color: var(--brand-1);
}

.event-players-col {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  min-width: 0;
  width: 100%;
  justify-content: center;
  padding: 0.2rem 0.36rem;
  border-radius: 8px;
  color: var(--ink-1);
  font-size: 0.82rem;
  font-weight: 760;
}

.event-players-col .material-symbols-rounded {
  font-size: 1rem;
  color: var(--ink-2);
}

.event-status-col {
  min-width: 0;
  width: 100%;
  display: inline-flex;
  justify-content: center;
}

.event-date-col {
  min-width: 0;
  width: 100%;
  display: inline-flex;
  justify-content: center;
  color: var(--ink-2);
  font-size: 0.84rem;
}

.event-date-col strong {
  font-weight: 640;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-details-btn {
  border: 1px solid color-mix(in srgb, #0b1019 62%, var(--line) 38%);
  background: linear-gradient(180deg, #090d14 0%, #0c121d 100%);
  color: var(--ink-1);
  border-radius: 8px;
  text-decoration: none;
  padding: 0.34rem 0.62rem;
  font-size: 0.8rem;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.event-details-btn:hover {
  border-color: color-mix(in srgb, var(--brand-1) 36%, #0b1019 64%);
  color: color-mix(in srgb, var(--brand-1) 72%, #fff 28%);
}

@media (max-width: 920px) {
  .event-list-item {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto auto;
  }

  .event-format-col {
    display: none;
  }

  .event-date-col {
    display: none;
  }

  .event-status-col {
    min-width: 80px;
  }
}
</style>
