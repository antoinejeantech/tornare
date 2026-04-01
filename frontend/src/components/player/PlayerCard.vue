<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { getRoleIcon } from '../../lib/roles'
import DiscordIcon from '../ui/DiscordIcon.vue'
import BnetIcon from '../ui/BnetIcon.vue'
import type { EventPlayer, RoleRank } from '../../types'

const props = withDefaults(defineProps<{
  player: EventPlayer
  clickable?: boolean
  showSocials?: boolean
}>(), {
  clickable: false,
  showSocials: false,
})

const emit = defineEmits<{
  (e: 'select', player: EventPlayer): void
  (e: 'selectRole', player: EventPlayer, rp: RoleRank): void
}>()

function emitSelectRole(rp: RoleRank, event: Event) {
  if (!props.clickable) return
  event.stopPropagation()
  emit('selectRole', props.player, rp)
}

function playerInitials(name: string) {
  const tokens = String(name || '').trim().split(/\s+/).filter(Boolean)
  if (tokens.length === 0) {
    return '??'
  }

  if (tokens.length === 1) {
    return tokens[0].slice(0, 2).toUpperCase()
  }

  return `${tokens[0][0] || ''}${tokens[1][0] || ''}`.toUpperCase()
}

function rankTierClass(rank: string) {
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

const copied = ref<'discord' | 'bnet' | null>(null)
let copyTimer: ReturnType<typeof setTimeout> | null = null

async function copy(text: string, field: 'discord' | 'bnet') {
  if (!navigator.clipboard) return
  try {
    await navigator.clipboard.writeText(text)
    if (copyTimer) clearTimeout(copyTimer)
    copied.value = field
    copyTimer = setTimeout(() => { copied.value = null }, 1500)
  } catch {
    // silently ignore permission denied / insecure-context errors
  }
}

onUnmounted(() => {
  if (copyTimer) clearTimeout(copyTimer)
})
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
    <div class="player-top-row">
      <span class="player-avatar" aria-hidden="true">
        <img
          v-if="player.linked_user?.avatar_url"
          :src="player.linked_user.avatar_url"
          :alt="player.name"
          class="player-avatar-img"
        />
        <template v-else>{{ playerInitials(player.name) }}</template>
      </span>
      <strong class="player-name">{{ player.name }}</strong>
      <div
        v-if="showSocials && (player.linked_user || player.reported_discord || player.reported_battletag)"
        class="player-social-cluster"
      >
        <RouterLink
          v-if="player.linked_user"
          class="social-icon-btn social-icon-btn--profile"
          :to="`/profiles/${player.linked_user.id}`"
          title="View profile"
          @click.stop
        >
          <span class="material-symbols-rounded social-icon-sym">person</span>
        </RouterLink>
        <span
          v-if="player.linked_user && (player.linked_user?.discord_username || player.reported_discord || player.linked_user?.battletag || player.reported_battletag)"
          class="social-sep"
          aria-hidden="true"
        ></span>
        <button
          v-if="player.linked_user?.discord_username || player.reported_discord"
          type="button"
          class="social-icon-btn social-icon-btn--discord"
          :class="{ 'is-verified': player.linked_user?.discord_username, 'is-copied': copied === 'discord' }"
          :data-tip="copied === 'discord' ? 'Copied!' : (player.linked_user?.discord_username ?? player.reported_discord ?? '')"
          @click.stop="copy(player.linked_user?.discord_username ?? player.reported_discord ?? '', 'discord')"
        >
          <DiscordIcon class="social-icon" />
          <span v-if="player.linked_user?.discord_username" class="material-symbols-rounded social-verified-badge" aria-hidden="true">check_circle</span>
        </button>
        <button
          v-if="player.linked_user?.battletag || player.reported_battletag"
          type="button"
          class="social-icon-btn social-icon-btn--bnet"
          :class="{ 'is-verified': player.linked_user?.battletag, 'is-copied': copied === 'bnet' }"
          :data-tip="copied === 'bnet' ? 'Copied!' : (player.linked_user?.battletag ?? player.reported_battletag ?? '')"
          @click.stop="copy(player.linked_user?.battletag ?? player.reported_battletag ?? '', 'bnet')"
        >
          <BnetIcon class="social-icon" />
          <span v-if="player.linked_user?.battletag" class="material-symbols-rounded social-verified-badge" aria-hidden="true">check_circle</span>
        </button>
      </div>
    </div>
    <div class="player-roles-row">
      <div v-if="!player.team_id && player.roles?.length > 1" class="player-pref-roles">
        <span
          v-for="(rp, i) in player.roles"
          :key="i"
          class="pref-role-chip"
          :class="{ 'is-top': i === 0, 'is-interactive': clickable }"
          :role="clickable ? 'button' : undefined"
          :tabindex="clickable ? 0 : undefined"
          :title="clickable ? `Add as ${rp.role} · ${rp.rank}` : undefined"
          @click="emitSelectRole(rp, $event)"
          @keydown.enter.stop.prevent="clickable && emitSelectRole(rp, $event)"
          @keydown.space.stop.prevent="clickable && emitSelectRole(rp, $event)"
        >
          <span class="material-symbols-rounded pref-role-icon" aria-hidden="true">{{ getRoleIcon(rp.role) }}</span>
          {{ rp.role }} · {{ rp.rank }}
        </span>
      </div>
      <div v-else class="player-meta-pills">
        <span class="role-pill">
          <span class="material-symbols-rounded role-inline-icon" aria-hidden="true">{{ getRoleIcon(player.role) }}</span>
          <span>{{ player.role }}</span>
        </span>
        <span class="rank-pill" :class="rankTierClass(player.rank)">{{ player.rank }}</span>
      </div>
    </div>
  </article>
</template>

<style scoped>
.player-card {
  width: 100%;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: var(--radius-md);
  padding: 0.72rem 0.78rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
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

.player-top-row {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  min-width: 0;
}

.player-roles-row {
  display: flex;
  min-width: 0;
  height: 1.6rem;
  align-items: center;
  overflow: hidden;
}

.player-avatar {
  width: 2.8rem;
  height: 2.8rem;
  border-radius: var(--radius-pill);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.88rem;
  font-weight: 800;
  letter-spacing: 0.04em;
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
  border: 1px solid color-mix(in srgb, var(--line-strong) 74%, var(--bg-0) 26%);
  background: color-mix(in srgb, var(--bg-1) 82%, var(--card) 18%);
  flex-shrink: 0;
  overflow: hidden;
}

.player-avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.player-name {
  font-size: 1.02rem;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 1;
}

.player-meta-pills {
  display: inline-flex;
  flex-wrap: nowrap;
  align-items: center;
  gap: 0.35rem;
  overflow: hidden;
}

.role-pill,
.rank-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-pill);
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

.player-pref-roles {
  display: inline-flex;
  flex-wrap: nowrap;
  gap: 0.26rem;
  overflow: hidden;
}

.pref-role-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  border-radius: var(--radius-pill);
  padding: 0.1rem 0.44rem;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  border: 1px solid color-mix(in srgb, var(--line) 88%, transparent 12%);
  background: transparent;
  color: color-mix(in srgb, var(--ink-2) 70%, transparent 30%);
  user-select: none;
  white-space: nowrap;
  flex-shrink: 0;
}

.pref-role-chip.is-top {
  color: var(--primary-300);
  border-color: color-mix(in srgb, var(--primary-500) 52%, var(--line) 48%);
  background: color-mix(in srgb, var(--primary-700) 18%, transparent 82%);
  font-weight: 700;
}

.pref-role-chip.is-interactive {
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}

.pref-role-chip.is-interactive:hover,
.pref-role-chip.is-interactive:focus-visible {
  background: color-mix(in srgb, var(--primary-700) 32%, transparent 68%);
  border-color: var(--primary-400);
  color: var(--primary-200);
  outline: none;
}

.pref-role-chip.is-interactive:not(.is-top):hover,
.pref-role-chip.is-interactive:not(.is-top):focus-visible {
  background: color-mix(in srgb, var(--bg-1) 24%, transparent 76%);
  border-color: color-mix(in srgb, var(--line-strong) 72%, transparent 28%);
  color: var(--ink-1);
}

.pref-role-icon {
  font-size: 0.82rem;
  line-height: 1;
}

.linked-user-strip {
  display: none;
}

.player-social-cluster {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 0.1rem;
}

.social-verified-badge {
  position: absolute;
  bottom: 0px;
  right: -1px;
  font-size: 0.5rem;
  line-height: 1;
  color: #4ade80;
  pointer-events: none;
  font-variation-settings: 'FILL' 1;
}

.social-sep {
  width: 1px;
  height: 0.75rem;
  background: var(--line);
  margin: 0 0.22rem;
  flex-shrink: 0;
}

.social-icon-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.9rem;
  height: 1.9rem;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--ink-3);
  transition: color 0.12s, background 0.12s;
  padding: 0;
  flex-shrink: 0;
}

/* Custom tooltip via ::before (tag text) */
.social-icon-btn::before {
  content: attr(data-tip);
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
  font-size: 0.68rem;
  font-weight: 600;
  line-height: 1;
  padding: 0.28rem 0.48rem;
  border-radius: var(--radius-sm);
  background: var(--bg-0, #111);
  color: var(--ink-1);
  border: 1px solid var(--line);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.1s;
  z-index: 10;
}

.social-icon-btn:hover::before {
  opacity: 1;
}

.social-icon-btn.is-copied::before {
  opacity: 1;
  color: #86efac;
}

.social-icon-btn:hover {
  background: color-mix(in srgb, currentColor 10%, transparent 90%);
}

.social-icon-btn--discord:hover {
  color: #adb3ff;
}

.social-icon-btn--bnet:hover {
  color: #74bbff;
}

.social-icon-btn--discord.is-verified {
  color: #adb3ff;
}

.social-icon-btn--bnet.is-verified {
  color: #74bbff;
}

.social-icon-btn--profile {
  color: var(--ink-3);
  text-decoration: none;
}

.social-icon-btn--profile::before {
  content: none;
}

.social-icon-btn--profile:hover {
  color: var(--ink-1);
  background: color-mix(in srgb, currentColor 10%, transparent 90%);
}

.social-icon-sym {
  font-size: 1.15rem;
  line-height: 1;
  font-variation-settings: 'FILL' 1;
}

.social-icon {
  width: 1.05rem;
  height: 1.05rem;
  fill: currentColor;
  flex-shrink: 0;
}

/* BNet path has a 32×32 viewBox vs Discord's 24×24 — render it slightly
   larger and boost opacity so it feels equally heavy at small sizes. */
.social-icon-btn--bnet .social-icon {
  width: 1.22rem;
  height: 1.22rem;
  opacity: 1;
}
</style>
