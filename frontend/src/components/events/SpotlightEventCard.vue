<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import spotlightEventBackground from '../../assets/branding/testbanner2.webp'
import { formatMediumDate } from '../../lib/dates'
import ActionCtaButton from '../ui/ActionCtaButton.vue'
import type { Event } from '../../types'

const { t } = useI18n()
const props = withDefaults(defineProps<{
  event: Event
  badgeLabel?: string
}>(), {
  badgeLabel: 'Spotlight Event',
})

const eventLink = computed(() => {
  return { name: 'event', params: { id: props.event.id } }
})

const eventDateText = computed(() => {
  return formatMediumDate(props.event?.start_date, t('spotlight.dateTbd'))
})

const eventTypeText = computed(() => {
  return props.event?.event_type === 'TOURNEY' ? t('events.typeTourney') : t('events.typePug')
})

const eventFormatText = computed(() => {
  return String(props.event?.format || '5v5')
})

const eventModeText = computed(() => {
  return `${eventTypeText.value} ${eventFormatText.value}`
})

const spotlightCardStyle = computed(() => {
  return {
    backgroundImage: [
      'radial-gradient(700px 110px at 0% 0%, rgba(255, 255, 255, 0.08), transparent 62%)',
      'radial-gradient(360px 170px at 92% 52%, rgba(0, 0, 0, 0.24), transparent 72%)',
      'linear-gradient(90deg, rgba(14, 18, 30, 0.78) 0%, rgba(16, 20, 34, 0.7) 62%, rgba(14, 18, 30, 0.82) 100%)',
      `url(${spotlightEventBackground})`,
    ].join(', '),
    backgroundSize: 'auto, auto, auto, cover',
    backgroundPosition: '0 0, 92% 52%, 0 0, center',
    backgroundRepeat: 'no-repeat',
  }
})

const eventPlayersText = computed(() => {
  return t('spotlight.players', { count: getPlayerCount(props.event), max: Number(props.event?.max_players) || 0 })
})

function getPlayerCount(event: typeof props.event): number {
  return Array.isArray(event?.players) ? event.players.length : 0
}
</script>

<template>
  <section class="card spotlight-event-card" :style="spotlightCardStyle">
    <div class="spotlight-head">
      <span class="spotlight-badge">{{ badgeLabel }}</span>
    </div>

    <h2 class="spotlight-title">
      <RouterLink class="spotlight-title-link" :to="eventLink">{{ event.name }}</RouterLink>
    </h2>
    <p class="spotlight-meta" aria-label="Event details">
      <span class="spotlight-meta-item spotlight-meta-badge">
        <span class="material-symbols-rounded spotlight-meta-icon" aria-hidden="true">calendar_month</span>
        <span>{{ eventDateText }}</span>
      </span>
      <span class="spotlight-meta-item spotlight-meta-badge">
        <span class="material-symbols-rounded spotlight-meta-icon" aria-hidden="true">trophy</span>
        <span>{{ eventModeText }}</span>
      </span>
      <span class="spotlight-meta-item spotlight-meta-badge">
        <span class="material-symbols-rounded spotlight-meta-icon" aria-hidden="true">group</span>
        <span>{{ eventPlayersText }}</span>
      </span>
    </p>
    <div class="spotlight-cta-stack">
      <ActionCtaButton class="spotlight-cta" :to="eventLink">{{ t('spotlight.signUpNow') }}</ActionCtaButton>
      <p class="spotlight-cta-note">{{ t('spotlight.limitedSlots') }}</p>
    </div>
  </section>
</template>

<style scoped>
.spotlight-event-card {
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 0.38rem;
  min-height: 136px;
  padding: 1.55rem 1.6rem 1.7rem;
  padding-right: 1.6rem;
  border-color: color-mix(in srgb, var(--brand-1) 40%, var(--line) 60%);
  box-shadow: none;
}

.spotlight-head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.5rem;
  margin-bottom: var(--space-1);
  z-index: 2;
}

.spotlight-badge {
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--brand-1);
}

.spotlight-title,
.spotlight-meta {
  margin: 0;
  position: relative;
  z-index: 2;
}

.spotlight-title-link {
  display: block;
  max-width: 16ch;
  text-decoration: none;
  color: inherit;
}

.spotlight-title-link:hover {
  text-decoration: none;
}

.spotlight-cta {
  text-decoration: none;
}

.spotlight-title {
  font-size: clamp(1.7rem, 1.1vw + 1.2rem, 2.3rem);
  margin-bottom: var(--space-2);
}

.spotlight-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.36rem 0.62rem;
  line-height: 1.3;
  font-size: 0.86rem;
  color: white;
}

.spotlight-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
}

.spotlight-meta-badge {
  padding: 0.26rem 0.5rem;
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--card) 48%, transparent 52%);
}

.spotlight-event-card .spotlight-meta-icon.material-symbols-rounded {
  font-size: 0.9rem;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%) !important;
}

.spotlight-cta-stack {
  position: absolute;
  right: 1rem;
  top: 50%;
  transform: translateY(-50%);
  z-index: 2;
  display: grid;
  justify-items: center;
  gap: 0.35rem;
  cursor: pointer;
  user-select: none;
}

.spotlight-cta-note {
  margin: 0;
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--brand-1) 90%, white 10%);
}

@media (max-width: 980px) {
  .spotlight-event-card {
    padding: 1.2rem;
  }

  .spotlight-cta-stack {
    position: static;
    transform: none;
    justify-self: center;
    margin-top: 0.5rem;
  }
}
</style>
