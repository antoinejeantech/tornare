<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import type { EventCtxType } from '../../lib/event-inject'

const ctx = inject<EventCtxType>('eventCtx')!
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
const pugRef = ref<InstanceType<typeof PugEventMatchesSection> | null>(null)
</script>

<template>
  <section style="min-width: 0">
    <EventSectionHeader icon="sports_score" title="Matches and Matchups">
      <button
        v-if="!isTourney && ctx.canManageEvent"
        class="cta-new-match"
        type="button"
        @click="pugRef?.toggleCreateForm()"
      >
        <span class="material-symbols-rounded" aria-hidden="true">add</span>
        New Match
      </button>
    </EventSectionHeader>

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else ref="pugRef" />
  </section>
</template>

<style scoped>
.cta-new-match {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  padding: 0.48rem 1rem 0.48rem 0.62rem;
  border-radius: var(--radius-md);
  border: none;
  background: linear-gradient(135deg, var(--brand-1) 0%, color-mix(in srgb, var(--brand-1) 72%, #7340e8 28%) 100%);
  color: #101216;
  font-size: 0.86rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  cursor: pointer;
  box-shadow: 0 2px 10px color-mix(in srgb, var(--brand-1) 35%, transparent 65%);
  transition: box-shadow 0.15s, transform 0.12s;
  white-space: nowrap;
  line-height: 1;
}

.cta-new-match:hover {
  box-shadow: 0 4px 18px color-mix(in srgb, var(--brand-1) 48%, transparent 52%);
  transform: translateY(-1px);
}

.cta-new-match:active {
  transform: translateY(0);
}

.cta-new-match .material-symbols-rounded {
  font-size: 1.05rem;
}
</style>
