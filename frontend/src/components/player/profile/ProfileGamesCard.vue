<script setup>
import { getRoleIcon } from '../../../lib/roles'

defineProps({
  profile: {
    type: Object,
    required: true,
  },
  canEdit: {
    type: Boolean,
    default: false,
  },
  editingOverwatch: {
    type: Boolean,
    default: false,
  },
  overwatchSummaryRows: {
    type: Array,
    default: () => [],
  },
  overwatchLogo: {
    type: String,
    required: true,
  },
})

defineEmits(['edit-overwatch'])
</script>

<template>
  <article class="card profile-games-card">
    <header class="games-header">
      <h3 class="games-title">
        <span class="material-symbols-rounded games-title-icon" aria-hidden="true">sports_esports</span>
        <span>Linked Games</span>
      </h3>
      <button
        v-if="canEdit && !editingOverwatch"
        type="button"
        class="action-btn games-manage-btn"
        @click="$emit('edit-overwatch')"
      >
        Manage Library
      </button>
    </header>

    <section class="game-panel">
      <div class="game-row">
        <h4 class="game-name-wrap">
          <img class="game-logo" :src="overwatchLogo" alt="Overwatch" />
          <span class="game-title-stack">
            <span class="game-name">Overwatch 2</span>
            <span class="game-publisher">Blizzard Entertainment</span>
          </span>
        </h4>
        <a class="game-external" href="https://overwatch.blizzard.com" target="_blank" rel="noreferrer noopener" aria-label="Open Overwatch official website">
          <span class="material-symbols-rounded" aria-hidden="true">open_in_new</span>
        </a>
      </div>

      <div v-if="editingOverwatch" class="game-edit-shell animated-panel">
        <slot name="overwatch-edit" />
      </div>

      <div v-else class="game-summary-shell animated-panel">
        <div class="battletag-state" :class="{ missing: !profile.battletag }">
          <span class="battletag-soon">Soon</span>
          <p v-if="profile.battletag" class="battletag-copy">
            Connected as <strong>{{ profile.battletag }}</strong>
          </p>
          <p v-else class="battletag-copy">
            No battletag configured yet. Connect your account to synchronize official ranks.
          </p>
          <button
            class="battletag-link battletag-link-disabled"
            type="button"
            disabled
          >
            Connect Battle.net Account
          </button>
        </div>

        <div class="rank-tile-grid">
          <article v-for="entry in overwatchSummaryRows" :key="entry.role" class="rank-tile">
            <p class="rank-role">
              <span>{{ entry.role }}</span>
              <span class="material-symbols-rounded rank-role-icon" aria-hidden="true">{{ getRoleIcon(entry.role) }}</span>
            </p>
            <p class="rank-value">
              <img class="rank-icon" :src="entry.icon" :alt="`${entry.rank} rank`" />
              <span>{{ entry.rank }}</span>
            </p>
          </article>
        </div>
      </div>
    </section>

    <section class="quick-actions">
      <p class="quick-actions-label">Quick Actions</p>
      <div class="quick-actions-row">
        <button type="button" class="action-btn" disabled>
          <span class="material-symbols-rounded" aria-hidden="true">add</span>
          <span>Add Game</span>
        </button>
        <button type="button" class="action-btn" disabled>
          <span class="material-symbols-rounded" aria-hidden="true">settings</span>
          <span>Account Settings</span>
        </button>
      </div>
    </section>
  </article>
</template>

<style scoped>
.profile-games-card {
  border: none;
  display: grid;
  gap: 0.9rem;
  padding: 1.2rem;
}

.games-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.7rem;
}

.games-title {
  margin: 0;
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.42rem;
}

.games-title-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.95rem;
  height: 1.95rem;
  border-radius: var(--radius-sm);
  color: color-mix(in srgb, var(--brand-1) 92%, #ffe7aa 8%);
  border: 1px solid color-mix(in srgb, var(--brand-1) 54%, transparent 46%);
  background: color-mix(in srgb, var(--brand-1) 18%, transparent 82%);
  font-size: 1.18rem;
  font-variation-settings: 'FILL' 1, 'wght' 650, 'GRAD' 0, 'opsz' 20;
}

.games-manage-btn {
  border-radius: var(--radius-sm);
  padding: 0.42rem 1rem;
}

.game-panel {
  border: none;
  border-radius: 12px;
  margin: 0.45rem 0;
  padding: 1.55rem 1.05rem;
  display: grid;
  gap: 1.05rem;
  position: relative;
}

.game-panel::before,
.game-panel::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  height: 1px;
  background: color-mix(in srgb, var(--line) 26%, transparent 74%);
}

.game-panel::before {
  top: 0;
}

.game-panel::after {
  bottom: 0;
}

.game-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.85rem;
}

.game-name-wrap {
  margin: 0;
  display: inline-flex;
  align-items: center;
  gap: 0.56rem;
}

.game-title-stack {
  display: grid;
  gap: 0.08rem;
  line-height: 1.1;
}

.game-name {
  font-size: 1.08rem;
  font-weight: 700;
}

.game-logo {
  width: 2rem;
  height: 2rem;
  object-fit: contain;
}

.game-publisher {
  margin: 0;
  color: var(--ink-muted);
  font-size: 0.82rem;
}

.game-external {
  color: var(--ink-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 30%, transparent 70%);
}

.battletag-state {
  display: grid;
  gap: 0.68rem;
  border: 1px dashed color-mix(in srgb, var(--line) 42%, transparent 58%);
  border-radius: var(--radius-item);
  padding: 0.9rem;
  margin-bottom: 0.8rem;
  justify-items: center;
  text-align: center;
}

.battletag-state.missing {
  border-color: color-mix(in srgb, var(--line) 52%, transparent 48%);
}

.battletag-copy {
  margin: 0;
  color: var(--ink-2);
  font-size: 0.9rem;
}

.battletag-soon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed color-mix(in srgb, var(--line) 56%, transparent 44%);
  border-radius: 999px;
  padding: 0.12rem 0.46rem;
  font-size: 0.66rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.battletag-link {
  justify-self: center;
  background: none;
  border: none;
  padding: 0;
  color: color-mix(in srgb, var(--brand-1) 92%, #ffefbf 8%);
  font-size: 0.86rem;
  font-weight: 700;
  text-decoration: none;
}

.battletag-link:hover {
  text-decoration: underline;
}

.battletag-link-disabled,
.battletag-link-disabled:hover {
  color: color-mix(in srgb, var(--ink-muted) 82%, transparent 18%);
  text-decoration: none;
  cursor: not-allowed;
}

.rank-tile-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.72rem;
}

.rank-tile {
  display: grid;
  gap: 0.46rem;
  padding: 0.84rem 0.84rem 0.72rem;
  border-radius: var(--radius-item);
  border: 1px solid color-mix(in srgb, var(--line) 26%, transparent 74%);
  position: relative;
}

.rank-tile::after {
  content: '';
  display: block;
  width: 78%;
  height: 2px;
  border-radius: 3px;
}

.rank-role,
.rank-value {
  margin: 0;
}

.rank-role {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.72rem;
  color: var(--ink-1);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 700;
}

.rank-role-icon {
  font-size: 0.92rem;
  color: color-mix(in srgb, var(--brand-1) 92%, #ffe7aa 8%);
}

.rank-value {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  font-weight: 760;
  color: var(--ink-1);
}

.rank-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.quick-actions {
  padding-top: 0.75rem;
  margin-top: 0.25rem;
  display: grid;
  gap: 0.5rem;
}

.quick-actions-label {
  margin: 0;
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-muted);
  font-weight: 700;
}

.quick-actions-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.55rem;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.32rem;
  padding: 0.48rem 0.66rem;
  border: 1px solid color-mix(in srgb, var(--line) 34%, transparent 66%);
  background: color-mix(in srgb, var(--card) 84%, var(--bg-0) 16%);
  color: var(--ink-1);
}

.action-btn:disabled {
  opacity: 1;
  cursor: not-allowed;
}

@media (max-width: 720px) {
  .games-header {
    flex-wrap: wrap;
  }

  .rank-tile-grid,
  .quick-actions-row {
    grid-template-columns: 1fr;
  }
}
</style>
