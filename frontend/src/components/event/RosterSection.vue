<script setup>
import { inject } from 'vue'

const ctx = inject('eventCtx')

function roleIcon(role) {
  if (role === 'Tank') {
    return 'shield'
  }
  if (role === 'Support') {
    return 'medical_services'
  }
  return 'swords'
}

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
</script>

<template>
  <section>
    <h3>Roster</h3>
    <p v-if="!ctx.canManageEvent" class="muted">Read-only roster. Sign in as the owner to add or edit players.</p>
    <form v-if="ctx.canManageEvent" class="player-form compact-form" @submit.prevent="ctx.addPlayer">
      <label>
        Player name
        <input v-model="ctx.newPlayerName" placeholder="Player123" />
      </label>
      <label>
        Role
        <select v-model="ctx.newPlayerRole">
          <option>Tank</option>
          <option>DPS</option>
          <option>Support</option>
        </select>
      </label>
      <label>
        Rank
        <select v-model="ctx.newPlayerRank">
          <option v-for="rank in ctx.overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
        </select>
      </label>
      <button type="submit" class="btn-primary" :disabled="!ctx.canAddPlayer || ctx.addingPlayer || ctx.eventIsFull">
        {{ ctx.addingPlayer ? 'Adding...' : 'Add player' }}
      </button>
    </form>

    <p v-if="ctx.event.players.length === 0" class="muted">Add players before creating matchups.</p>
    <ul v-else class="players-list roster-list">
      <li v-for="player in ctx.event.players" :key="player.id" class="player-row">
        <div class="player-main">
          <template v-if="ctx.editingPlayerId === player.id">
            <div class="inline-edit-grid">
              <input v-model="ctx.editPlayerName" placeholder="Player name" />
              <select v-model="ctx.editPlayerRole">
                <option>Tank</option>
                <option>DPS</option>
                <option>Support</option>
              </select>
              <select v-model="ctx.editPlayerRank">
                <option v-for="rank in ctx.overwatchRanks" :key="`edit-rank-${rank}`" :value="rank">{{ rank }}</option>
              </select>
            </div>
          </template>
          <template v-else>
            <strong class="player-name">{{ player.name }}</strong>
            <div class="player-meta-row">
              <span class="muted role-inline">
                <span class="material-symbols-rounded role-inline-icon" aria-hidden="true">{{ roleIcon(player.role) }}</span>
                <span>{{ player.role }}</span>
              </span>
              <span class="rank-chip" :title="player.rank" :aria-label="player.rank">
                <img class="rank-icon" :src="ctx.getRankIcon(player.rank)" :alt="`${player.rank} rank`" />
                <span>{{ player.rank }}</span>
              </span>
            </div>
          </template>
        </div>
        <div class="player-actions">
          <button
            v-if="ctx.canManageEvent && ctx.editingPlayerId === player.id"
            class="btn-primary icon-btn"
            :disabled="Boolean(ctx.savingPlayerEdits[player.id])"
            :title="ctx.savingPlayerEdits[player.id] ? 'Saving player' : 'Save player'"
            @click="ctx.savePlayerEdit(player.id)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ ctx.savingPlayerEdits[player.id] ? 'hourglass_top' : 'save' }}
            </span>
            <span class="sr-only">{{ ctx.savingPlayerEdits[player.id] ? 'Saving player' : 'Save player' }}</span>
          </button>
          <button
            v-if="ctx.canManageEvent && ctx.editingPlayerId === player.id"
            class="btn-secondary icon-btn"
            title="Cancel editing player"
            @click="cancelEditPlayer"
          >
            <span class="material-symbols-rounded" aria-hidden="true">close</span>
            <span class="sr-only">Cancel editing player</span>
          </button>
          <button
            v-if="ctx.canManageEvent && ctx.editingPlayerId !== player.id"
            class="btn-secondary icon-btn"
            title="Edit player"
            @click="startEditPlayer(player)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">edit</span>
            <span class="sr-only">Edit player</span>
          </button>
          <button
            v-if="ctx.canManageEvent && ctx.editingPlayerId !== player.id"
            class="btn-danger icon-btn"
            :disabled="Boolean(ctx.deletingPlayers[player.id])"
            :title="ctx.deletingPlayers[player.id] ? 'Removing player' : 'Remove player'"
            @click="ctx.removePlayer(player)"
          >
            <span class="material-symbols-rounded" aria-hidden="true">
              {{ ctx.deletingPlayers[player.id] ? 'hourglass_top' : 'person_remove' }}
            </span>
            <span class="sr-only">{{ ctx.deletingPlayers[player.id] ? 'Removing player' : 'Remove player' }}</span>
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.player-form {
  display: grid;
  gap: 0.56rem;
  margin-bottom: 0.72rem;
}

.player-form label {
  display: grid;
  gap: 0.28rem;
}

.inline-edit-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.38rem;
}

.players-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.player-row {
  border: 1px solid color-mix(in srgb, var(--line) 92%, var(--brand-1) 8%);
  background: color-mix(in srgb, var(--card) 90%, #f1f5ff 10%);
  border-radius: 10px;
  padding: 0.64rem 0.7rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.65rem;
}

.roster-list {
  max-height: 420px;
  overflow: auto;
  padding-right: 0.15rem;
}

.player-main {
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

.player-actions {
  display: flex;
  gap: 0.45rem;
  align-items: center;
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
  gap: 0.2rem;
  padding: 0.12rem 0.34rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-2) 16%);
  background: color-mix(in srgb, var(--card) 85%, #eaf1ff 15%);
  font-size: 0.78rem;
  color: var(--ink-2);
}

.rank-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
}

@media (max-width: 900px) {
  .inline-edit-grid {
    grid-template-columns: 1fr;
  }

  .player-row {
    flex-direction: column;
  }

  .player-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
