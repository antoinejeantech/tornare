<script setup lang="ts">
import { computed, inject, ref } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import ActionCtaButton from '../ui/ActionCtaButton.vue'
import type { EventCtxType } from '../../lib/event-inject'

const ctx = inject<EventCtxType>('eventCtx')!
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
const pugRef = ref<InstanceType<typeof PugEventMatchesSection> | null>(null)
</script>

<template>
  <section style="min-width: 0">
    <EventSectionHeader icon="sports_score" title="Matches and Matchups">
      <ActionCtaButton
        v-if="!isTourney && ctx.canManageEvent"
        class="cta-new-match"
        type="button"
        @click="pugRef?.toggleCreateForm()"
      >
        <span class="material-symbols-rounded" aria-hidden="true">add</span>
        New Match
      </ActionCtaButton>
    </EventSectionHeader>

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else ref="pugRef" />
  </section>
</template>

<style scoped>
.cta-new-match {
  gap: 0.3rem;
  font-size: 0.82rem;
  padding: 0.38rem 0.8rem;
  min-height: unset;
}
</style>
