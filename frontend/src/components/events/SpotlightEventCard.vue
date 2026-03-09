<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { formatEventStartDate } from '../../lib/dates'
import tracerImage from '../../assets/branding/tracer.png'

const props = defineProps({
  event: {
    type: Object,
    required: true,
  },
  badgeLabel: {
    type: String,
    default: 'Spotlight Event',
  },
  artSrc: {
    type: String,
    default: '',
  },
  artAlt: {
    type: String,
    default: '',
  },
})

const eventLink = computed(() => {
  return { name: 'event', params: { id: props.event.id } }
})

const metaText = computed(() => {
  const startText = formatEventStartDate(props.event?.start_date)
  const parts = [
    String(props.event?.event_type || 'PUG'),
    String(props.event?.format || '5v5'),
    `${getPlayerCount(props.event)}/${Number(props.event?.max_players) || 0} players`,
  ]

  if (startText) {
    parts.push(startText)
  }

  return parts.join(' · ')
})

function getPlayerCount(event) {
  return Array.isArray(event?.players) ? event.players.length : 0
}
</script>

<template>
  <section class="card spotlight-event-card">
    <RouterLink class="spotlight-open-link" :to="eventLink">
      <span>View Event</span>
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </RouterLink>

    <div class="spotlight-head">
      <span class="spotlight-badge">{{ badgeLabel }}</span>
    </div>

    <h2 class="spotlight-title">{{ event.name }}</h2>
    <p class="muted spotlight-meta">{{ metaText }}</p>
    <RouterLink class="btn-primary spotlight-cta" :to="eventLink">Register Now</RouterLink>

    <img class="spotlight-art" :src="artSrc || tracerImage" :alt="artAlt || 'Tracer spotlight art'" />
  </section>
</template>

<style scoped>
.spotlight-event-card {
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 0.38rem;
  min-height: 136px;
  padding-right: clamp(9.8rem, 27vw, 18rem);
  border-color: color-mix(in srgb, var(--brand-1) 40%, var(--line) 60%);
  background:
    radial-gradient(700px 110px at 0% 0%, rgba(255, 255, 255, 0.08), transparent 62%),
    radial-gradient(360px 170px at 92% 52%, rgba(0, 0, 0, 0.24), transparent 72%),
    linear-gradient(90deg, color-mix(in srgb, var(--card) 94%, #232323 6%) 0%, color-mix(in srgb, var(--card) 96%, #2a2a2a 4%) 68%, color-mix(in srgb, #181818 72%, var(--card) 28%) 100%);
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, var(--brand-1) 20%, transparent 80%),
    0 0 14px color-mix(in srgb, #dbe8ff 28%, transparent 72%),
    0 10px 22px rgba(27, 82, 160, 0.2),
    0 2px 8px rgba(21, 44, 88, 0.12);
}

.spotlight-head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.5rem;
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

.spotlight-cta {
  text-decoration: none;
}

.spotlight-open-link {
  position: absolute;
  top: 0.7rem;
  right: 0.95rem;
  z-index: 3;
  display: inline-flex;
  align-items: center;
  gap: 0.26rem;
  text-decoration: none;
  padding: 0;
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--brand-1);
  border: 0;
}

.spotlight-open-link svg {
  width: 0.78rem;
  height: 0.78rem;
  transition: transform 180ms ease;
}

.spotlight-open-link:hover {
  color: color-mix(in srgb, var(--brand-1) 80%, #fff 20%);
  text-decoration: underline;
}

.spotlight-open-link:hover svg {
  transform: translateX(2px);
}

.spotlight-title {
  font-size: clamp(1.45rem, 0.8vw + 1.05rem, 2rem);
}

.spotlight-meta {
  line-height: 1.3;
  font-size: 0.86rem;
}

.spotlight-cta {
  position: absolute;
  right: 1rem;
  bottom: 1rem;
  z-index: 2;
  font-size: 1.05rem;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: 8px;
  padding: 0.56rem 1.02rem;
  box-shadow: 0 10px 22px rgba(123, 89, 30, 0.36);
}

.spotlight-art {
  position: absolute;
  right: 0.72rem;
  bottom: 0;
  width: clamp(124px, 19vw, 210px);
  max-height: 96%;
  object-fit: contain;
  pointer-events: none;
  opacity: 0.92;
  filter: drop-shadow(0 6px 16px rgba(9, 18, 38, 0.34));
  z-index: 1;
}

@media (max-width: 980px) {
  .spotlight-event-card {
    padding-right: 1.15rem;
    padding-bottom: 7.2rem;
  }

  .spotlight-art {
    right: 50%;
    transform: translateX(50%);
    width: clamp(120px, 40vw, 190px);
    opacity: 0.72;
  }

  .spotlight-cta {
    right: 50%;
    transform: translateX(50%);
    bottom: 0.78rem;
    font-size: 0.88rem;
  }
}
</style>
