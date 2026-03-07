<script setup>
import { computed, inject } from 'vue'
import PugEventMatchesSection from './PugEventMatchesSection.vue'
import TourneyEventMatchesSection from './TourneyEventMatchesSection.vue'

const ctx = inject('eventCtx')
const isTourney = computed(() => Boolean(ctx.isTourneyEvent))
</script>

<template>
  <section>
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">sports_esports</span>
      <span>Matches and Matchups</span>
    </h3>

    <TourneyEventMatchesSection v-if="isTourney" />
    <PugEventMatchesSection v-else />
  </section>
</template>

<style scoped>
.section-title {
  margin: 0 0 0.3rem;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}
</style>
