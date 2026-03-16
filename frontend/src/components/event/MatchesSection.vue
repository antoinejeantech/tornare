<script setup>
import { computed, inject } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'
import EventSectionHeader from './EventSectionHeader.vue'

const ctx = inject('eventCtx')
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
</script>

<template>
  <section style="min-width: 0">
    <EventSectionHeader icon="sports_score" title="Matches and Matchups" />

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else />
  </section>
</template>
