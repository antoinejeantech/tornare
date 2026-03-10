<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import testBackground from '../../assets/branding/test.jpg'
import ActionCtaButton from '../ui/ActionCtaButton.vue'

const props = defineProps({
  event: {
    type: Object,
    required: true,
  },
  badgeLabel: {
    type: String,
    default: 'Spotlight Event',
  },
})

const eventLink = computed(() => {
  return { name: 'event', params: { id: props.event.id } }
})

const eventDateText = computed(() => {
  const value = props.event?.start_date
  if (!value) {
    return 'Date TBD'
  }

  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) {
    return 'Date TBD'
  }

  return parsed.toLocaleDateString([], {
    month: 'short',
    day: '2-digit',
    year: 'numeric',
  })
})

const eventTypeText = computed(() => {
  return String(props.event?.event_type || 'PUG')
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
      `url(${testBackground})`,
    ].join(', '),
    backgroundSize: 'auto, auto, auto, cover',
    backgroundPosition: '0 0, 92% 52%, 0 0, center',
    backgroundRepeat: 'no-repeat',
  }
})

const eventPlayersText = computed(() => {
  return `${getPlayerCount(props.event)}/${Number(props.event?.max_players) || 0} players`
})

function getPlayerCount(event) {
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
      <ActionCtaButton class="spotlight-cta" :to="eventLink">SIGN UP NOW</ActionCtaButton>
      <p class="spotlight-cta-note">LIMITED SLOTS AVAILABLE</p>
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
  padding: 1.25rem 1.25rem 1.4rem;
  padding-right: 1.25rem;
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

.spotlight-meta-icon {
  font-size: 0.9rem;
  color: color-mix(in srgb, white 92%, var(--brand-1) 8%);
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
    padding: 1rem;
    padding-right: 1rem;
    padding-bottom: 1rem;
  }

  .spotlight-cta-stack {
    right: 50%;
    transform: translateX(50%);
    bottom: 0.78rem;
    top: auto;
  }
}
</style>
