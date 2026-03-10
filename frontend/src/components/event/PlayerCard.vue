<script setup>
import { getRoleIcon } from '../../lib/roles'

const props = defineProps({
  player: {
    type: Object,
    required: true,
  },
  clickable: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['select'])

function playerInitials(name) {
  const tokens = String(name || '').trim().split(/\s+/).filter(Boolean)
  if (tokens.length === 0) {
    return '??'
  }

  if (tokens.length === 1) {
    return tokens[0].slice(0, 2).toUpperCase()
  }

  return `${tokens[0][0] || ''}${tokens[1][0] || ''}`.toUpperCase()
}

function rankTierClass(rank) {
  const normalized = String(rank || '').trim().toLowerCase()
  if (normalized.startsWith('bronze')) return 'rank-tier-bronze'
  if (normalized.startsWith('silver')) return 'rank-tier-silver'
  if (normalized.startsWith('gold')) return 'rank-tier-gold'
  if (normalized.startsWith('platinum')) return 'rank-tier-platinum'
  if (normalized.startsWith('diamond')) return 'rank-tier-diamond'
  if (normalized.startsWith('master')) return 'rank-tier-master'
  if (normalized.startsWith('grandmaster')) return 'rank-tier-grandmaster'
  if (normalized.startsWith('champion')) return 'rank-tier-champion'
  if (normalized.startsWith('top 500') || normalized.startsWith('top500')) return 'rank-tier-top500'
  return 'rank-tier-unranked'
}

function emitSelect() {
  if (!props.clickable) {
    return
  }

  emit('select', props.player)
}
</script>

<template>
  <article
    class="player-card"
    :class="{ 'is-clickable': clickable }"
    :role="clickable ? 'button' : undefined"
    :tabindex="clickable ? 0 : undefined"
    @click="emitSelect"
    @keydown.enter.prevent="emitSelect"
    @keydown.space.prevent="emitSelect"
  >
    <div class="player-identity-row">
      <span class="player-avatar" aria-hidden="true">{{ playerInitials(player.name) }}</span>
      <div class="player-copy">
        <strong class="player-name">{{ player.name }}</strong>
        <div class="player-meta-pills">
          <span class="role-pill">
            <span class="material-symbols-rounded role-inline-icon" aria-hidden="true">{{ getRoleIcon(player.role) }}</span>
            <span>{{ player.role }}</span>
          </span>
          <span class="rank-pill" :class="rankTierClass(player.rank)">{{ player.rank }}</span>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
.player-card {
  width: 100%;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: 10px;
  padding: 1.08rem 0.78rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: center;
  gap: 0.65rem;
}

.player-card.is-clickable {
  cursor: pointer;
}

.player-card.is-clickable:hover {
  border-color: color-mix(in srgb, var(--line-strong) 78%, var(--brand-1) 22%);
}

.player-card.is-clickable:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--brand-1) 74%, #ffd869 26%);
  outline-offset: 2px;
}

.player-identity-row {
  width: 100%;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.6rem;
  align-items: center;
}

.player-avatar {
  width: 2.28rem;
  height: 2.28rem;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.76rem;
  font-weight: 800;
  letter-spacing: 0.04em;
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
  border: 1px solid color-mix(in srgb, var(--line-strong) 74%, var(--bg-0) 26%);
  background: color-mix(in srgb, var(--bg-1) 82%, var(--card) 18%);
}

.player-copy {
  min-width: 0;
  display: grid;
  gap: 0.28rem;
}

.player-name {
  font-size: 0.92rem;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.player-meta-pills {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem;
}

.role-pill,
.rank-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  padding: 0.1rem 0.48rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  border: 1px solid;
}

.role-pill {
  gap: 0.18rem;
  text-transform: uppercase;
  border-color: color-mix(in srgb, var(--line) 82%, var(--brand-1) 18%);
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
  background: color-mix(in srgb, var(--card) 88%, var(--bg-1) 12%);
}

.role-inline-icon {
  font-size: 0.92rem;
  line-height: 1;
}

.rank-pill {
  background: color-mix(in srgb, var(--card) 88%, var(--bg-1) 12%);
}

.rank-tier-bronze {
  border-color: color-mix(in srgb, #a56b3a 72%, var(--line) 28%);
  color: #d39a63;
}

.rank-tier-silver {
  border-color: color-mix(in srgb, #9ea7b5 72%, var(--line) 28%);
  color: #d1d7e0;
}

.rank-tier-gold {
  border-color: color-mix(in srgb, #c9a458 78%, var(--line) 22%);
  color: #efd08a;
}

.rank-tier-platinum {
  border-color: color-mix(in srgb, #43b3b8 72%, var(--line) 28%);
  color: #8be1e5;
}

.rank-tier-diamond {
  border-color: color-mix(in srgb, #7ea6ff 72%, var(--line) 28%);
  color: #b4cbff;
}

.rank-tier-master {
  border-color: color-mix(in srgb, #b987ff 72%, var(--line) 28%);
  color: #d9b9ff;
}

.rank-tier-grandmaster {
  border-color: color-mix(in srgb, #ff8a8a 72%, var(--line) 28%);
  color: #ffb0b0;
}

.rank-tier-champion {
  border-color: color-mix(in srgb, #ff77bc 76%, var(--line) 24%);
  color: #ffc4e3;
}

.rank-tier-top500 {
  border-color: color-mix(in srgb, #ff6f47 76%, var(--line) 24%);
  color: #ffb29c;
}

.rank-tier-unranked {
  border-color: color-mix(in srgb, var(--line) 82%, var(--brand-1) 18%);
  color: var(--ink-2);
}
</style>
