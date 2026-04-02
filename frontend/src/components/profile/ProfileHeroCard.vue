<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AuthUser } from '../../types'
import AvatarPickerPopup from './AvatarPickerPopup.vue'

withDefaults(defineProps<{
  profile: AuthUser
  canEdit?: boolean
  editingAccount?: boolean
  profileInitial?: string
  savingAvatar?: boolean
}>(), {
  canEdit: false,
  editingAccount: false,
  profileInitial: 'A',
  savingAvatar: false,
})

defineEmits<{
  (e: 'edit-account'): void
  (e: 'update-avatar', avatarUrl: string | null): void
}>()

const showPicker = ref(false)
const { t } = useI18n()
</script>

<template>
  <article class="card profile-hero-card">
    <div class="hero-identity-row">
      <div class="hero-identity">
        <div class="hero-avatar-wrap" :class="{ 'hero-avatar-wrap--editable': canEdit }">
          <span v-if="!profile.avatar_url" class="hero-avatar" aria-hidden="true">{{ profileInitial }}</span>
          <img v-else class="hero-avatar hero-avatar--img" :src="profile.avatar_url" :alt="profile.display_name" referrerpolicy="no-referrer" />

          <button
            v-if="canEdit"
            type="button"
            class="hero-avatar-edit-btn"
            :class="{ 'hero-avatar-edit-btn--saving': savingAvatar }"
            :disabled="savingAvatar"
            :title="savingAvatar ? t('profileCard.changePictureSaving') : t('profileCard.changePicture')"
            aria-haspopup="dialog"
            @click.stop="showPicker = !showPicker"
          >
            <span class="material-symbols-rounded" aria-hidden="true">{{ savingAvatar ? 'hourglass_empty' : 'photo_camera' }}</span>
            <span class="sr-only">{{ t('profileCard.changePicture') }}</span>
          </button>

          <AvatarPickerPopup
            v-if="showPicker"
            :current-avatar-url="profile.avatar_url ?? null"
            :has-discord="profile.has_discord_identity"
            @pick="(url) => { $emit('update-avatar', url); showPicker = false }"
            @close="showPicker = false"
          />
        </div>
        <div class="hero-name-wrap">
          <h2 class="hero-display-name">{{ profile.display_name }}</h2>
          <p class="hero-username">@{{ profile.username }}</p>
        </div>
      </div>

      <div class="hero-actions">
        <button
          v-if="canEdit && !editingAccount"
          type="button"
          class="hero-icon-btn"
          title="Edit profile"
          @click="$emit('edit-account')"
        >
          <span class="material-symbols-rounded" aria-hidden="true">edit</span>
          <span class="sr-only">{{ t('profileCard.editProfile') }}</span>
        </button>
        <slot name="hero-actions" />
      </div>
    </div>

    <div v-if="editingAccount" class="hero-edit-shell">
      <slot name="account-edit" />
    </div>

    <dl v-else class="hero-details-grid">
      <div class="hero-details-row">
        <dt>{{ t('profileCard.usernameLabel') }}</dt>
        <dd>{{ profile.username }}</dd>
      </div>
      <div class="hero-details-row">
        <dt>{{ t('profileCard.displayNameLabel') }}</dt>
        <dd>{{ profile.display_name }}</dd>
      </div>
      <div class="hero-details-row">
        <dt>{{ t('profileCard.roleLabel') }}</dt>
        <dd>{{ profile.role || 'user' }}</dd>
      </div>
      <div v-if="canEdit" class="hero-details-row">
        <dt>{{ t('profileCard.emailLabel') }}</dt>
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
  align-content: start;
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
  border-radius: var(--radius-pill);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  color: #101216;
  background: linear-gradient(145deg, color-mix(in srgb, var(--brand-1) 94%, white 6%), var(--brand-1));
  border: 1px solid color-mix(in srgb, var(--brand-1) 60%, var(--line) 40%);
  box-shadow: 0 12px 20px rgba(27, 20, 7, 0.32);
  flex-shrink: 0;
}

.hero-avatar--img {
  object-fit: cover;
}

.hero-avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.hero-avatar-edit-btn {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border-radius: var(--radius-pill);
  border: none;
  background: rgba(0, 0, 0, 0.52);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 140ms;
}

.hero-avatar-wrap--editable:hover .hero-avatar-edit-btn,
.hero-avatar-edit-btn:focus-visible {
  opacity: 1;
}

.hero-avatar-edit-btn--saving {
  opacity: 1;
  cursor: default;
}

.hero-avatar-edit-btn .material-symbols-rounded {
  font-size: 1.1rem;
  font-variation-settings: 'FILL' 1, 'wght' 400, 'GRAD' 0, 'opsz' 20;
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

.hero-actions {
  display: flex;
  flex-direction: row;
  gap: 0.35rem;
  flex-shrink: 0;
  align-items: center;
}

.hero-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.1rem;
  height: 2.1rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--line);
  background: transparent;
  color: var(--ink-muted);
  cursor: pointer;
  transition: background 120ms, color 120ms, border-color 120ms;
}

.hero-icon-btn:hover {
  background: color-mix(in srgb, var(--brand-1) 12%, transparent);
  border-color: var(--brand-1);
  color: var(--brand-1);
}

.hero-icon-btn .material-symbols-rounded {
  font-size: 1.1rem;
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20;
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
