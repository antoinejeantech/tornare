<script setup>
defineProps({
  profile: {
    type: Object,
    required: true,
  },
  canEdit: {
    type: Boolean,
    default: false,
  },
  editingAccount: {
    type: Boolean,
    default: false,
  },
  profileInitial: {
    type: String,
    default: 'A',
  },
})

defineEmits(['edit-account'])
</script>

<template>
  <article class="card profile-hero-card">
    <div class="hero-identity-row">
      <div class="hero-identity">
        <span class="hero-avatar" aria-hidden="true">{{ profileInitial }}</span>
        <div class="hero-name-wrap">
          <h2 class="hero-display-name">{{ profile.display_name }}</h2>
          <p class="hero-username">@{{ profile.username }}</p>
        </div>
      </div>

      <button
        v-if="canEdit && !editingAccount"
        type="button"
        class="btn-primary hero-edit-btn"
        @click="$emit('edit-account')"
      >
        Edit Profile
      </button>
    </div>

    <div v-if="editingAccount" class="hero-edit-shell">
      <slot name="account-edit" />
    </div>

    <dl v-else class="hero-details-grid">
      <div class="hero-details-row">
        <dt>Username</dt>
        <dd>{{ profile.username }}</dd>
      </div>
      <div class="hero-details-row">
        <dt>Display Name</dt>
        <dd>{{ profile.display_name }}</dd>
      </div>
      <div class="hero-details-row">
        <dt>Role</dt>
        <dd>{{ profile.role || 'user' }}</dd>
      </div>
      <div v-if="canEdit" class="hero-details-row">
        <dt>Email</dt>
        <dd>{{ profile.email }}</dd>
      </div>
    </dl>
  </article>
</template>

<style scoped>
.profile-hero-card {
  border: none;
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 1.08rem;
  padding: 1.2rem;
}

.hero-identity-row,
.hero-edit-shell,
.hero-details-grid {
  position: relative;
  z-index: 1;
}

.hero-identity-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.hero-identity {
  display: inline-flex;
  align-items: center;
  gap: 0.9rem;
  min-width: 0;
}

.hero-avatar {
  width: 3.34rem;
  height: 3.34rem;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  color: #101216;
  background: linear-gradient(145deg, color-mix(in srgb, var(--brand-1) 94%, white 6%), var(--brand-1));
  border: 1px solid color-mix(in srgb, var(--brand-1) 60%, var(--line) 40%);
  box-shadow: 0 12px 20px rgba(27, 20, 7, 0.32);
}

.hero-name-wrap {
  min-width: 0;
  display: grid;
  gap: 0.1rem;
}

.hero-display-name {
  margin: 0;
  font-size: clamp(1.4rem, 1.12rem + 0.75vw, 1.9rem);
}

.hero-username {
  margin: 0;
  color: var(--ink-muted);
  font-weight: 650;
}

.hero-edit-btn {
  white-space: nowrap;
  border-radius: 8px;
  padding: 0.42rem 1rem;
}

.hero-details-grid {
  display: grid;
  gap: 0.3rem;
  padding-top: 0.9rem;
}

.hero-details-row {
  display: grid;
  grid-template-columns: 128px minmax(0, 1fr);
  gap: 0.8rem;
  align-items: baseline;
  padding: 0.48rem 0;
  border-bottom: 1px solid color-mix(in srgb, var(--line) 72%, transparent 28%);
}

.hero-details-row dt,
.hero-details-row dd {
  margin: 0;
}

.hero-details-row dt {
  color: var(--ink-muted);
  font-size: 0.84rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.hero-details-row dd {
  font-weight: 700;
  color: var(--ink-1);
}

@media (max-width: 720px) {
  .hero-identity-row {
    flex-wrap: wrap;
  }

  .hero-details-row {
    grid-template-columns: 1fr;
    gap: 0.22rem;
  }
}
</style>
