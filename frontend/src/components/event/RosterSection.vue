<script setup>
import { computed, inject } from 'vue'
import PlayerCard from './PlayerCard.vue'

const ctx = inject('eventCtx')

function startEditPlayer(player) {
  ctx.editingPlayerId = player.id
  ctx.editPlayerName = player.name
  ctx.editPlayerRole = player.role
  ctx.editPlayerRank = player.rank
}

function cancelEditPlayer() {
  ctx.editingPlayerId = null
  ctx.editPlayerName = ''
  ctx.editPlayerRole = 'DPS'
  ctx.editPlayerRank = 'Unranked'
}

const activeEditPlayer = computed(() => {
  if (!ctx.event || !ctx.editingPlayerId) {
    return null
  }

  return ctx.event.players.find((player) => player.id === ctx.editingPlayerId) || null
})

function openPlayerEditModal(player) {
  if (!ctx.canManageEvent) {
    return
  }

  startEditPlayer(player)
}

async function removePlayerFromModal() {
  const player = activeEditPlayer.value
  if (!player) {
    return
  }

  await ctx.removePlayer(player)

  const stillExists = Array.isArray(ctx.event?.players)
    ? ctx.event.players.some((entry) => entry.id === player.id)
    : false

  if (!stillExists) {
    cancelEditPlayer()
  }
}

function padRosterCount(value) {
  const numeric = Number(value)
  if (!Number.isFinite(numeric) || numeric < 0) {
    return '00'
  }

  const floored = Math.floor(numeric)
  if (floored < 10) {
    return `0${floored}`
  }

  return String(floored)
}

</script>

<template>
  <section>
    <div class="section-header-row">
      <h3 class="section-title">
        <span class="material-symbols-rounded section-title-icon" aria-hidden="true">groups</span>
        <span>Roster Management</span>
      </h3>
      <p class="section-total muted">
        <span class="section-total-value">{{ padRosterCount(ctx.event?.players?.length || 0) }}/{{ padRosterCount(ctx.event?.max_players || 0) }}</span>
        <span>players</span>
      </p>
    </div>
    <div class="section-title-divider" aria-hidden="true"></div>
    <form v-if="ctx.canManageEvent" class="player-form" @submit.prevent="ctx.addPlayer">
      <label class="player-form-field">
        Player name
        <input v-model="ctx.newPlayerName" placeholder="Player123" />
      </label>
      <label class="player-form-field">
        Role
        <select v-model="ctx.newPlayerRole">
          <option>Tank</option>
          <option>DPS</option>
          <option>Support</option>
        </select>
      </label>
      <label class="player-form-field">
        Rank
        <select v-model="ctx.newPlayerRank">
          <option v-for="rank in ctx.overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
        </select>
      </label>
      <button type="submit" class="btn-primary player-form-submit" :disabled="!ctx.canAddPlayer || ctx.addingPlayer || ctx.eventIsFull">
        {{ ctx.addingPlayer ? 'Adding...' : 'Add player' }}
      </button>
    </form>

    <p v-if="ctx.event.players.length === 0" class="muted">Add players before creating matchups.</p>
    <ul v-else class="roster-list">
      <li v-for="player in ctx.event.players" :key="player.id" class="roster-list-item">
        <PlayerCard :player="player" :clickable="ctx.canManageEvent" @select="openPlayerEditModal" />
      </li>
    </ul>

    <div v-if="ctx.canManageEvent && activeEditPlayer" class="player-modal-backdrop" @click.self="cancelEditPlayer">
      <section class="player-modal" role="dialog" aria-modal="true" :aria-label="`Edit ${activeEditPlayer.name}`">
        <div class="player-modal-header">
          <h4>Edit player</h4>
          <button class="btn-secondary icon-btn" title="Close player editor" @click="cancelEditPlayer">
            <span class="material-symbols-rounded" aria-hidden="true">close</span>
            <span class="sr-only">Close player editor</span>
          </button>
        </div>

        <form class="player-modal-form" @submit.prevent="ctx.savePlayerEdit(activeEditPlayer.id)">
          <label>
            Player name
            <input v-model="ctx.editPlayerName" placeholder="Player name" />
          </label>
          <label>
            Role
            <select v-model="ctx.editPlayerRole">
              <option>Tank</option>
              <option>DPS</option>
              <option>Support</option>
            </select>
          </label>
          <label>
            Rank
            <select v-model="ctx.editPlayerRank">
              <option v-for="rank in ctx.overwatchRanks" :key="`modal-edit-rank-${rank}`" :value="rank">{{ rank }}</option>
            </select>
          </label>

          <div class="player-modal-actions">
            <button
              class="btn-danger"
              type="button"
              :disabled="Boolean(ctx.deletingPlayers[activeEditPlayer.id])"
              @click="removePlayerFromModal"
            >
              {{ ctx.deletingPlayers[activeEditPlayer.id] ? 'Removing...' : 'Delete player' }}
            </button>
            <button class="btn-secondary" type="button" @click="cancelEditPlayer">Cancel</button>
            <button
              class="btn-primary"
              type="submit"
              :disabled="!ctx.editPlayerName.trim() || Boolean(ctx.savingPlayerEdits[activeEditPlayer.id])"
            >
              {{ ctx.savingPlayerEdits[activeEditPlayer.id] ? 'Saving...' : 'Save player' }}
            </button>
          </div>
        </form>
      </section>
    </div>
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

.player-form {
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(0, 0.9fr) minmax(0, 1fr) auto;
  align-items: end;
  gap: 0.62rem;
  margin: 1.1rem 0 1.35rem;
  padding: 1.02rem 1.08rem;
  border: 1px solid var(--surface-card-border);
  border-radius: 10px;
  background: var(--surface-card-bg);
  box-shadow: none;
}

.player-form-field {
  display: grid;
  gap: 0.28rem;
  font-size: 0.76rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.player-form-submit {
  white-space: nowrap;
  min-height: 2.3rem;
}

.roster-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  align-items: start;
  max-height: none;
  overflow: visible;
  padding-right: 0;
}

.roster-list-item {
  min-width: 0;
}

@media (max-width: 1200px) {
  .player-form {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: stretch;
  }

  .player-form-submit {
    width: 100%;
  }

  .roster-list {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.player-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
  background: color-mix(in srgb, #02040a 72%, transparent 28%);
  display: grid;
  place-items: center;
  padding: 1rem;
}

.player-modal {
  width: min(520px, 100%);
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: 12px;
  padding: 0.92rem;
  display: grid;
  gap: 0.72rem;
}

.player-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
}

.player-modal-header h4 {
  margin: 0;
}

.player-modal-form {
  display: grid;
  gap: 0.56rem;
}

.player-modal-form label {
  display: grid;
  gap: 0.24rem;
}

.player-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.45rem;
  flex-wrap: wrap;
}

@media (max-width: 900px) {
  .player-form {
    grid-template-columns: 1fr;
  }

  .roster-list {
    grid-template-columns: 1fr;
    max-height: 56vh;
    overflow: auto;
    padding-right: 0.15rem;
  }

  .player-modal {
    width: 100%;
  }

  .player-modal-actions {
    width: 100%;
    justify-content: stretch;
  }

  .player-modal-actions .btn-danger,
  .player-modal-actions .btn-secondary,
  .player-modal-actions .btn-primary {
    width: 100%;
  }
}
</style>
