<script setup>
import { computed, inject } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'

const ctx = inject('eventCtx')
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
</script>

<template>
  <section>
    <div class="section-header-row">
      <h3 class="section-title">
        <span class="material-symbols-rounded section-title-icon" aria-hidden="true">sports_score</span>
        <span>Matches and Matchups</span>
      </h3>
    </div>
    <div class="section-title-divider" aria-hidden="true"></div>

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else />
  </section>
</template>
