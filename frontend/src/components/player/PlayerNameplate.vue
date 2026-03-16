<script setup>
import { computed } from 'vue'
import { getRankIcon } from '../../lib/ranks'
import { getRoleIcon } from '../../lib/roles'

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
  role: {
    type: String,
    default: '',
  },
  rank: {
    type: String,
    default: '',
  },
  compact: {
    type: Boolean,
    default: false,
  },
})

const rankAlt = computed(() => `${props.rank} rank`)
const rankIcon = computed(() => getRankIcon(props.rank))
</script>

<template>
  <div class="player-identity">
    <strong class="player-name">{{ name }}</strong>
    <div class="player-meta-row">
      <span class="muted role-inline">
        <span class="material-symbols-rounded role-inline-icon" aria-hidden="true">{{ getRoleIcon(role) }}</span>
        <span>{{ role }}</span>
      </span>
      <span class="rank-chip" :class="{ compact }" :title="rank" :aria-label="rank">
        <img class="rank-icon" :src="rankIcon" :alt="rankAlt" />
        <span>{{ rank }}</span>
      </span>
    </div>
  </div>
</template>

<style scoped>
.player-identity {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.28rem;
}

.player-name {
  line-height: 1.2;
}

.player-meta-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.34rem;
}

.role-inline {
  display: inline-flex;
  align-items: center;
  gap: 0.14rem;
}

.role-inline-icon {
  font-size: 1rem;
}

.rank-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
  margin-left: 0.42rem;
  padding: 0.1rem 0.45rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: color-mix(in srgb, var(--card) 88%, #19253a 12%);
  font-size: 0.84rem;
}

.rank-chip.compact {
  gap: 0.2rem;
  margin-left: 0;
  padding: 0.12rem 0.34rem;
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 85%, #eaf1ff 15%);
  font-size: 0.78rem;
  color: var(--ink-2);
}

.rank-icon {
  width: 18px;
  height: 18px;
  object-fit: contain;
}

.rank-chip.compact .rank-icon {
  width: 16px;
  height: 16px;
}
</style>
