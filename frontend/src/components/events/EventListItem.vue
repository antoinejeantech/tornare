<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import overwatchLogo from '../../assets/branding/overwatch-logo.png'
import { formatEventStartDate } from '../../lib/dates'

const props = defineProps({
  event: {
    type: Object,
    required: true,
  },
  as: {
    type: String,
    default: 'button',
  },
  to: {
    type: [String, Object],
    default: null,
  },
  showCreator: {
    type: Boolean,
    default: false,
  },
  fallbackFormat: {
    type: String,
    default: '5v5',
  },
})

const emit = defineEmits(['select'])

const formattedStartDate = computed(() => {
  return formatEventStartDate(props.event?.start_date) || ''
})

const playerCount = computed(() => {
  return Array.isArray(props.event?.players) ? props.event.players.length : 0
})

const eventFormat = computed(() => {
  return props.event?.format || props.fallbackFormat
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

const isLink = computed(() => props.as === 'link')

function onSelect() {
  emit('select')
}
</script>

<template>
  <li class="event-list-item">
    <RouterLink
      v-if="isLink"
      class="event-list-main"
      :to="to"
    >
      <span class="event-list-title-wrap">
        <img class="event-list-logo" :src="overwatchLogo" alt="Overwatch logo" />
        <span class="event-list-title">{{ event.name }}</span>
        <span class="event-status-chip" :class="statusClass">{{ statusLabel }}</span>
      </span>
      <span class="muted">
        {{ event.event_type }} · {{ eventFormat }}
        <template v-if="showCreator"> · by {{ event.creator_name || 'Unknown' }}</template>
        <template v-if="formattedStartDate"> · {{ formattedStartDate }}</template>
        · {{ playerCount }}/{{ event.max_players }} players
      </span>
    </RouterLink>

    <button
      v-else
      type="button"
      class="event-list-main"
      @click="onSelect"
    >
      <span class="event-list-title-wrap">
        <img class="event-list-logo" :src="overwatchLogo" alt="Overwatch logo" />
        <span class="event-list-title">{{ event.name }}</span>
        <span class="event-status-chip" :class="statusClass">{{ statusLabel }}</span>
      </span>
      <span class="muted">
        {{ event.event_type }} · {{ eventFormat }}
        <template v-if="showCreator"> · by {{ event.creator_name || 'Unknown' }}</template>
        <template v-if="formattedStartDate"> · {{ formattedStartDate }}</template>
        · {{ playerCount }}/{{ event.max_players }} players
      </span>
    </button>

    <slot name="actions" />
  </li>
</template>

<style scoped>
.event-list-item {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.7rem;
}

.event-list-main {
  all: unset;
  display: grid;
  gap: 0.2rem;
  min-width: 0;
  flex: 1;
  cursor: pointer;
}

.event-list-main:hover .event-list-title {
  color: var(--brand-1);
}

.event-list-title {
  font-weight: 800;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.event-list-title-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.event-list-logo {
  width: 18px;
  height: 18px;
  object-fit: contain;
  flex: 0 0 auto;
}

.event-status-chip {
  border-radius: 999px;
  padding: 0.12rem 0.48rem;
  font-size: 0.66rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border: 1px solid transparent;
}

.event-status-chip.is-open {
  color: #0b5a1e;
  background: #daf4e2;
  border-color: #95d9a9;
}

.event-status-chip.is-soon {
  color: #7a3b00;
  background: #ffe8c9;
  border-color: #ffc57f;
}

.event-status-chip.is-full {
  color: #7a2a0a;
  background: #ffd9ce;
  border-color: #ffad95;
}

.event-status-chip.is-ongoing {
  color: #fff;
  background: linear-gradient(130deg, #0f4f99, var(--brand-1));
  border-color: color-mix(in srgb, #0f4f99 75%, var(--brand-1) 25%);
}
</style>
