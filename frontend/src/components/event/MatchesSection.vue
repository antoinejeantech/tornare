<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import ActionCtaButton from '../ui/ActionCtaButton.vue'
import type { EventCtxType } from '../../composables/event/event-inject'

const ctx = inject<EventCtxType>('eventCtx')!
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
const pugRef = ref<InstanceType<typeof PugEventMatchesSection> | null>(null)
</script>

<template>
  <section style="min-width: 0">
    <EventSectionHeader icon="sports_score" title="Matches">
      <div class="header-right">
        <p class="section-total muted">
          <span class="section-total-value">{{ ctx.event?.matches?.length ?? 0 }}</span>
          <span>matches</span>
        </p>
        <ActionCtaButton
          v-if="!isTourney && ctx.canManageEvent"
          class="cta-new-match"
          type="button"
          @click="pugRef?.toggleCreateForm()"
        >
          <span class="material-symbols-rounded" aria-hidden="true">add</span>
          <span class="cta-new-match-label">New match</span>
        </ActionCtaButton>
      </div>
    </EventSectionHeader>

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else ref="pugRef" />
  </section>
</template>

<style scoped>
.header-right {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  flex-shrink: 0;
}

.section-total {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 0.35rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 0.72rem;
  font-weight: 700;
}

.section-total-value {
  font-size: 0.98rem;
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
}

.cta-new-match {
  gap: 0.3rem;
  font-size: 0.82rem;
  padding: 0.38rem 0.8rem;
  min-height: unset;
}

@media (max-width: 900px) {
  .cta-new-match-label { display: none; }
  .cta-new-match { padding: 0.42rem 0.62rem; }
}
</style>
