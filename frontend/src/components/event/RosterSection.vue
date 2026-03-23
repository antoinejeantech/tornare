<script setup lang="ts">
import { computed, inject } from 'vue'
import PlayerCard from '../player/PlayerCard.vue'
import EventSectionHeader from './EventSectionHeader.vue'
import type { EventCtxType } from '../../lib/event-inject'
import type { EventPlayer, OverwatchRole, RoleRank } from '../../types'

const ctx = inject<EventCtxType>('eventCtx')!

function startEditPlayer(player: EventPlayer) {
  ctx.editingPlayerId = player.id
  ctx.editPlayerName = player.name
  ctx.editPlayerRole = player.role
  ctx.editPlayerRank = player.rank
  // Populate role preferences: use roles if present, else single role/rank
  if (player.roles?.length) {
    ctx.editPlayerRoles = player.roles.map(rp => ({ role: rp.role, rank: rp.rank }))
  } else {
    ctx.editPlayerRoles = [{ role: player.role, rank: player.rank }]
  }
}

function cancelEditPlayer() {
  ctx.editingPlayerId = null
  ctx.editPlayerName = ''
  ctx.editPlayerRole = 'DPS'
  ctx.editPlayerRank = 'Unranked'
  ctx.editPlayerRoles = [{ role: 'DPS', rank: 'Unranked' }]
}

const activeEditPlayer = computed(() => {
  if (!ctx.event || !ctx.editingPlayerId) {
    return null
  }

  return ctx.event.players.find((player) => player.id === ctx.editingPlayerId) || null
})

function openPlayerEditModal(player: EventPlayer) {
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

function padRosterCount(value: unknown): string {
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

const usedEditRoles = computed(() => (ctx.editPlayerRoles || []).map(rp => rp.role))

function isEditRoleTaken(role: string, currentIndex: number): boolean {
  return usedEditRoles.value.some((r, i) => i !== currentIndex && r === role)
}

const availableEditRoles = computed(() => {
  const all: OverwatchRole[] = ['Tank', 'DPS', 'Support']
  return all.filter(r => !usedEditRoles.value.includes(r))
})

function addEditRole() {
  if ((ctx.editPlayerRoles?.length || 0) < 3 && availableEditRoles.value.length > 0) {
    ctx.editPlayerRoles.push({ role: availableEditRoles.value[0], rank: 'Unranked' } as RoleRank)
  }
}

function removeEditRole(index: number) {
  if (ctx.editPlayerRoles?.length > 1) {
    ctx.editPlayerRoles.splice(index, 1)
  }
}

const canSavePlayerEdit = computed(() => {
  if (!ctx.editPlayerName?.trim()) return false
  const roles = ctx.editPlayerRoles || []
  return roles.length > 0 && roles.every(rp => rp.role && rp.rank)
})

</script>

<template>
  <section>
    <EventSectionHeader icon="groups" title="Roster Management">
      <p class="section-total muted">
        <span class="section-total-value">{{ padRosterCount(ctx.event?.players?.length || 0) }}/{{ padRosterCount(ctx.event?.max_players || 0) }}</span>
        <span>players</span>
      </p>
    </EventSectionHeader>
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

    <p v-if="(ctx.event?.players.length ?? 0) === 0" class="muted">Add players before creating matchups.</p>
    <ul v-else class="roster-list">
      <li v-for="player in ctx.event?.players" :key="player.id" class="roster-list-item">
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

          <div class="modal-roles-section">
            <span class="modal-roles-label">ROLE PREFERENCES</span>
            <ul class="modal-roles-list">
              <li
                v-for="(entry, index) in ctx.editPlayerRoles"
                :key="index"
                class="modal-role-row"
              >
                <label class="modal-role-field">
                  <span class="modal-role-field-lbl">
                    Role
                    <span v-if="index === 0" class="modal-role-pref-hint">preferred</span>
                  </span>
                  <select v-model="entry.role">
                    <option value="" disabled hidden></option>
                    <option value="Tank" :disabled="isEditRoleTaken('Tank', index)">Tank</option>
                    <option value="DPS" :disabled="isEditRoleTaken('DPS', index)">DPS</option>
                    <option value="Support" :disabled="isEditRoleTaken('Support', index)">Support</option>
                  </select>
                </label>
                <label class="modal-role-field">
                  Rank
                  <select v-model="entry.rank">
                    <option value="" disabled hidden></option>
                    <option v-for="rank in ctx.overwatchRanks" :key="rank" :value="rank">{{ rank }}</option>
                  </select>
                </label>
                <div class="modal-role-remove-col">
                  <span class="modal-role-remove-spacer" aria-hidden="true">Role</span>
                  <button
                    v-if="ctx.editPlayerRoles.length > 1"
                    type="button"
                    class="modal-role-remove"
                    :aria-label="`Remove role preference ${index + 1}`"
                    @click="removeEditRole(index)"
                  >
                    <span class="material-symbols-rounded" aria-hidden="true">delete</span>
                  </button>
                </div>
              </li>
            </ul>
            <button
              v-if="ctx.editPlayerRoles.length < 3 && availableEditRoles.length > 0"
              type="button"
              class="modal-add-role"
              @click="addEditRole"
            >
              <span class="material-symbols-rounded" aria-hidden="true">add</span>
              Add role
            </button>
          </div>

          <div class="player-modal-actions">
            <button
              class="btn-danger"
              type="button"
              :disabled="Boolean(ctx.deletingPlayers[activeEditPlayer.id])"
              @click="removePlayerFromModal"
            >
              <span class="material-symbols-rounded" aria-hidden="true">delete</span>
              {{ ctx.deletingPlayers[activeEditPlayer.id] ? 'Removing…' : 'Delete' }}
            </button>
            <button class="btn-secondary" type="button" @click="cancelEditPlayer">
              <span class="material-symbols-rounded" aria-hidden="true">close</span>
              Cancel
            </button>
            <button
              class="btn-primary"
              type="submit"
              :disabled="!canSavePlayerEdit || Boolean(ctx.savingPlayerEdits[activeEditPlayer.id])"
            >
              <span class="material-symbols-rounded" aria-hidden="true">{{ ctx.savingPlayerEdits[activeEditPlayer.id] ? 'hourglass_empty' : 'save' }}</span>
              {{ ctx.savingPlayerEdits[activeEditPlayer.id] ? 'Saving…' : 'Save' }}
            </button>
          </div>
        </form>
      </section>
    </div>
  </section>
</template>

<style scoped>
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

.player-form {
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(0, 0.9fr) minmax(0, 1fr) auto;
  align-items: end;
  gap: 0.62rem;
  margin: 0 0 1.35rem;
  padding: 1.02rem 1.08rem;
  border: 1px solid var(--surface-card-border);
  border-radius: var(--radius-md);
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
  border-radius: var(--radius-md);
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
  gap: 1rem;
}

.player-modal-form label {
  display: grid;
  gap: 0.3rem;
}

.player-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.55rem;
  flex-wrap: wrap;
  margin-top: 0.75rem;
}

.player-modal-actions button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.player-modal-actions .material-symbols-rounded {
  font-size: 1.1rem;
}

.player-modal-actions .btn-danger {
  margin-right: auto;
}

/* ── Multi-role editor inside the modal ─────────────────── */
.modal-roles-section {
  display: grid;
  gap: 0.6rem;
}

.modal-roles-label {
  display: block;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.modal-roles-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.52rem;
}

.modal-role-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: end;
  gap: 0.45rem;
}

.modal-role-field {
  display: grid;
  gap: 0.22rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--ink-2);
}

.modal-role-field-lbl {
  display: flex;
  align-items: center;
  gap: 0.36rem;
}

.modal-role-pref-hint {
  font-size: 0.66rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-transform: lowercase;
  color: var(--primary-300);
  border: 1px solid color-mix(in srgb, var(--primary-500) 52%, var(--line) 48%);
  background: color-mix(in srgb, var(--primary-700) 22%, transparent 78%);
  padding: 0.04rem 0.32rem;
  border-radius: var(--radius-pill);
}

.modal-role-remove-col {
  display: grid;
  gap: 0.22rem;
}

.modal-role-remove-spacer {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: transparent;
  pointer-events: none;
  user-select: none;
}

.modal-role-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  padding: 0.3rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  color: var(--ink-2);
  transition: color 0.14s, background 0.14s;
  min-height: 2.12rem;
}

.modal-role-remove:hover {
  color: var(--danger, #e05c5c);
  background: color-mix(in srgb, var(--danger, #e05c5c) 10%, transparent 90%);
}

.modal-role-remove .material-symbols-rounded {
  font-size: 1.1rem;
}

.modal-add-role {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
  background: color-mix(in srgb, var(--bg-1) 60%, var(--card) 40%);
  border: 1px solid var(--line);
  color: var(--ink-2);
  border-radius: var(--radius-sm);
  padding: 0.4rem 0.65rem;
  font-size: 0.76rem;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.14s, color 0.14s;
}

.modal-add-role:hover {
  border-color: var(--primary-500);
  color: var(--primary-300);
}

.modal-add-role .material-symbols-rounded {
  font-size: 1rem;
}

/* kept for compat (old prefs section removed) */
.player-modal-signup-prefs-label,
.player-modal-signup-prefs-list,
.player-modal-signup-pref-tag { display: none; }


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
