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

<style scoped>
.section-title {
  margin: 0 0 0.3rem;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.section-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.section-title-divider {
  width: 100%;
  height: 1px;
  background: color-mix(in srgb, var(--line) 84%, var(--brand-1) 16%);
  margin: 0.42rem 0 0.72rem;
}

.section-title-icon {
  font-size: 1.26rem;
  line-height: 1;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
}
</style>
