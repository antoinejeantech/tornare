<script setup lang="ts">
import { computed, inject } from 'vue'
import EventSectionHeader from './EventSectionHeader.vue'
import AppBadge from '../ui/AppBadge.vue'
import { getRoleIcon } from '../../lib/roles'
import DiscordIcon from '../ui/DiscordIcon.vue'
import type { EventCtxType } from '../../composables/event/event-inject'
import type { SignupRequest } from '../../types'

const ctx = inject<EventCtxType>('eventCtx')!
const isPublicRegistration = computed(() => Boolean(ctx.event?.public_signup_enabled))

function toTimestamp(value: unknown): number | null {
  if (!value) {
    return null
  }

  const parsed = Date.parse(String(value))
  return Number.isNaN(parsed) ? null : parsed
}

function oldestFirst(requests: SignupRequest[]): SignupRequest[] {
  const copy = [...requests]
  const hasDateField = copy.some((request) => {
    return toTimestamp(request?.created_at) !== null || toTimestamp(request?.updated_at) !== null
  })

  if (hasDateField) {
    return copy.sort((a, b) => {
      const aTs = toTimestamp(a?.created_at) ?? toTimestamp(a?.updated_at) ?? 0
      const bTs = toTimestamp(b?.created_at) ?? toTimestamp(b?.updated_at) ?? 0
      return aTs - bTs
    })
  }

  // Backend currently returns newest first; reversing shows oldest first.
  return copy.reverse()
}

const pendingRequests = computed(() => {
  const requests = Array.isArray(ctx.signupRequests) ? ctx.signupRequests : []
  return oldestFirst(requests.filter((request) => request.status === 'pending'))
})

const reviewedRequests = computed(() => {
  const requests = Array.isArray(ctx.signupRequests) ? ctx.signupRequests : []
  return oldestFirst(requests.filter((request) => request.status !== 'pending'))
})

function getRequestRoles(request: SignupRequest): Array<{ role: string; rank: string }> {
  return (request.roles as Array<{ role: string; rank: string }> | undefined) || []
}
</script>

<template>
  <section>
    <EventSectionHeader icon="mail" title="Signup Requests" />

    <div class="signup-link-box">
      <div class="signup-visibility-row">
        <p class="muted signup-visibility-label">
          Registration is currently
          <strong>{{ isPublicRegistration ? 'Public' : 'Private' }}</strong>.
        </p>
        <button class="btn-secondary" type="button" @click="ctx.openSection('settings')">
          Go to settings
        </button>
      </div>

      <p class="muted signup-settings-hint">
        If you want to change event registration visibility, go to Settings.
      </p>

      <p class="muted">
        {{ isPublicRegistration ? 'Share this public link so players can request to join this event.' : 'You can still copy and share the current direct link while private. The public Join button is hidden.' }}
      </p>

      <div class="signup-link-row">
        <input :value="ctx.signupShareUrl || ''" readonly placeholder="Loading signup link..." />
        <button class="btn-secondary" :disabled="!ctx.signupShareUrl || ctx.rotatingSignupLink" @click="ctx.copySignupLink">
          Copy link
        </button>
        <button class="btn-danger" :disabled="ctx.rotatingSignupLink" @click="ctx.rotateSignupLink">
          {{ ctx.rotatingSignupLink ? 'Rotating...' : 'Rotate link' }}
        </button>
      </div>
    </div>

    <div class="signup-request-groups">
      <article class="signup-request-card">
        <h4>Pending</h4>
        <p v-if="ctx.loadingSignupRequests" class="muted">Loading signup requests...</p>
        <p v-else-if="pendingRequests.length === 0" class="muted">No pending requests yet.</p>
        <ul v-else class="signup-request-list">
          <li v-for="request in pendingRequests" :key="request.id" class="signup-request-item">
            <div class="signup-request-identity">
              <strong class="signup-request-name">{{ request.name }}</strong>
              <div class="signup-request-roles">
                <span
                  v-for="(rp, i) in getRequestRoles(request)"
                  :key="i"
                  class="req-role-badge"
                  :class="{ 'is-preferred': i === 0 }"
                >
                  <span class="material-symbols-rounded req-role-icon" aria-hidden="true">{{ getRoleIcon(rp.role) }}</span>
                  <span class="req-role-label">{{ rp.role }}</span>
                  <span class="req-role-sep" aria-hidden="true">·</span>
                  <span class="req-role-rank">{{ rp.rank }}</span>
                </span>
              </div>
              <div
                v-if="request.linked_user?.discord_username || request.linked_user?.battletag || request.reported_discord || request.reported_battletag"
                class="req-accounts-row"
              >
                <span v-if="request.linked_user?.discord_username" class="req-account-chip req-account-chip--discord req-account-chip--verified">
                  <DiscordIcon class="req-account-icon" />
                  {{ request.linked_user.discord_username }}
                  <span class="req-account-verified" aria-label="Verified connected account">
                    <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                  </span>
                </span>
                <span v-else-if="request.reported_discord" class="req-account-chip req-account-chip--discord">
                  <DiscordIcon class="req-account-icon" />
                  {{ request.reported_discord }}
                </span>
                <span v-if="request.linked_user?.battletag" class="req-account-chip req-account-chip--bnet req-account-chip--verified">
                  <svg class="req-account-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12.008 0C5.377 0 0 5.373 0 12c0 6.625 5.377 12 12.008 12C18.629 24 24 18.625 24 12 24 5.373 18.629 0 12.008 0zm.99 4.5c.305.002.607.025.906.069a6.564 6.564 0 0 1 4.04 2.376 6.544 6.544 0 0 1 1.33 3.91c0 .797-.143 1.584-.424 2.325l-.007.016c-.08.21-.18.438-.3.665l-.012.02a5.24 5.24 0 0 1-.572.84c-.02.022-.038.046-.057.068l-.055.06a5.233 5.233 0 0 1-3.86 1.685 5.24 5.24 0 0 1-3.244-1.122l-3.48 3.48a.878.878 0 0 1-1.24 0 .877.877 0 0 1 0-1.239l3.48-3.48a5.233 5.233 0 0 1-1.122-3.245 5.243 5.243 0 0 1 5.242-5.235l.012-.001.002-.001zm0 1.755a3.48 3.48 0 0 0-3.48 3.48 3.48 3.48 0 0 0 3.48 3.48 3.48 3.48 0 0 0 3.48-3.48 3.48 3.48 0 0 0-3.48-3.48z"/></svg>
                  {{ request.linked_user.battletag }}
                  <span class="req-account-verified" aria-label="Verified connected account">
                    <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                  </span>
                </span>
                <span v-else-if="request.reported_battletag" class="req-account-chip req-account-chip--bnet">
                  <svg class="req-account-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12.008 0C5.377 0 0 5.373 0 12c0 6.625 5.377 12 12.008 12C18.629 24 24 18.625 24 12 24 5.373 18.629 0 12.008 0zm.99 4.5c.305.002.607.025.906.069a6.564 6.564 0 0 1 4.04 2.376 6.544 6.544 0 0 1 1.33 3.91c0 .797-.143 1.584-.424 2.325l-.007.016c-.08.21-.18.438-.3.665l-.012.02a5.24 5.24 0 0 1-.572.84c-.02.022-.038.046-.057.068l-.055.06a5.233 5.233 0 0 1-3.86 1.685 5.24 5.24 0 0 1-3.244-1.122l-3.48 3.48a.878.878 0 0 1-1.24 0 .877.877 0 0 1 0-1.239l3.48-3.48a5.233 5.233 0 0 1-1.122-3.245 5.243 5.243 0 0 1 5.242-5.235l.012-.001.002-.001zm0 1.755a3.48 3.48 0 0 0-3.48 3.48 3.48 3.48 0 0 0 3.48 3.48 3.48 3.48 0 0 0 3.48-3.48 3.48 3.48 0 0 0-3.48-3.48z"/></svg>
                  {{ request.reported_battletag }}
                </span>
              </div>
            </div>
            <div class="signup-request-actions">
              <button
                class="btn-primary signup-action-btn"
                :disabled="Boolean(ctx.reviewingSignupRequests[request.id])"
                @click="ctx.acceptSignupRequest(request.id)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">
                  {{ ctx.reviewingSignupRequests[request.id] ? 'hourglass_top' : 'check_circle' }}
                </span>
                {{ ctx.reviewingSignupRequests[request.id] ? 'Saving…' : 'Accept' }}
              </button>
              <button
                class="btn-secondary signup-action-btn signup-decline-btn"
                :disabled="Boolean(ctx.reviewingSignupRequests[request.id])"
                @click="ctx.declineSignupRequest(request.id)"
              >
                <span class="material-symbols-rounded" aria-hidden="true">cancel</span>
                Decline
              </button>
            </div>
          </li>
        </ul>
      </article>

      <article class="signup-request-card">
        <h4>Reviewed</h4>
        <p v-if="reviewedRequests.length === 0" class="muted">No reviewed requests yet.</p>
        <ul v-else class="signup-request-list">
          <li v-for="request in reviewedRequests" :key="request.id" class="signup-request-item reviewed">
            <div class="signup-request-identity">
              <strong class="signup-request-name">{{ request.name }}</strong>
              <div class="signup-request-roles">
                <span
                  v-for="(rp, i) in getRequestRoles(request)"
                  :key="i"
                  class="req-role-badge"
                  :class="{ 'is-preferred': i === 0 }"
                >
                  <span class="material-symbols-rounded req-role-icon" aria-hidden="true">{{ getRoleIcon(rp.role) }}</span>
                  <span class="req-role-label">{{ rp.role }}</span>
                  <span class="req-role-sep" aria-hidden="true">·</span>
                  <span class="req-role-rank">{{ rp.rank }}</span>
                </span>
              </div>
              <div
                v-if="request.linked_user?.discord_username || request.linked_user?.battletag || request.reported_discord || request.reported_battletag"
                class="req-accounts-row"
              >
                <span v-if="request.linked_user?.discord_username" class="req-account-chip req-account-chip--discord req-account-chip--verified">
                  <DiscordIcon class="req-account-icon" />
                  {{ request.linked_user.discord_username }}
                  <span class="req-account-verified" aria-label="Verified connected account">
                    <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                  </span>
                </span>
                <span v-else-if="request.reported_discord" class="req-account-chip req-account-chip--discord">
                  <DiscordIcon class="req-account-icon" />
                  {{ request.reported_discord }}
                </span>
                <span v-if="request.linked_user?.battletag" class="req-account-chip req-account-chip--bnet req-account-chip--verified">
                  <svg class="req-account-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12.008 0C5.377 0 0 5.373 0 12c0 6.625 5.377 12 12.008 12C18.629 24 24 18.625 24 12 24 5.373 18.629 0 12.008 0zm.99 4.5c.305.002.607.025.906.069a6.564 6.564 0 0 1 4.04 2.376 6.544 6.544 0 0 1 1.33 3.91c0 .797-.143 1.584-.424 2.325l-.007.016c-.08.21-.18.438-.3.665l-.012.02a5.24 5.24 0 0 1-.572.84c-.02.022-.038.046-.057.068l-.055.06a5.233 5.233 0 0 1-3.86 1.685 5.24 5.24 0 0 1-3.244-1.122l-3.48 3.48a.878.878 0 0 1-1.24 0 .877.877 0 0 1 0-1.239l3.48-3.48a5.233 5.233 0 0 1-1.122-3.245 5.243 5.243 0 0 1 5.242-5.235l.012-.001.002-.001zm0 1.755a3.48 3.48 0 0 0-3.48 3.48 3.48 3.48 0 0 0 3.48 3.48 3.48 3.48 0 0 0 3.48-3.48 3.48 3.48 0 0 0-3.48-3.48z"/></svg>
                  {{ request.linked_user.battletag }}
                  <span class="req-account-verified" aria-label="Verified connected account">
                    <span class="material-symbols-rounded" aria-hidden="true">verified</span>
                  </span>
                </span>
                <span v-else-if="request.reported_battletag" class="req-account-chip req-account-chip--bnet">
                  <svg class="req-account-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12.008 0C5.377 0 0 5.373 0 12c0 6.625 5.377 12 12.008 12C18.629 24 24 18.625 24 12 24 5.373 18.629 0 12.008 0zm.99 4.5c.305.002.607.025.906.069a6.564 6.564 0 0 1 4.04 2.376 6.544 6.544 0 0 1 1.33 3.91c0 .797-.143 1.584-.424 2.325l-.007.016c-.08.21-.18.438-.3.665l-.012.02a5.24 5.24 0 0 1-.572.84c-.02.022-.038.046-.057.068l-.055.06a5.233 5.233 0 0 1-3.86 1.685 5.24 5.24 0 0 1-3.244-1.122l-3.48 3.48a.878.878 0 0 1-1.24 0 .877.877 0 0 1 0-1.239l3.48-3.48a5.233 5.233 0 0 1-1.122-3.245 5.243 5.243 0 0 1 5.242-5.235l.012-.001.002-.001zm0 1.755a3.48 3.48 0 0 0-3.48 3.48 3.48 3.48 0 0 0 3.48 3.48 3.48 3.48 0 0 0 3.48-3.48 3.48 3.48 0 0 0-3.48-3.48z"/></svg>
                  {{ request.reported_battletag }}
                </span>
              </div>
            </div>
            <AppBadge
              :variant="request.status === 'accepted' ? 'ok' : 'danger'"
              :label="request.status"
              radius="pill"
            />
          </li>
        </ul>
      </article>
    </div>
  </section>
</template>

<style scoped>
.signup-link-box {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--card) 92%, #f0f6ff 8%);
  padding: 0.62rem;
  margin-bottom: 0.72rem;
  display: grid;
  gap: 0.5rem;
}

.signup-link-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 0.45rem;
}

.signup-visibility-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.6rem;
}

.signup-visibility-label {
  margin: 0;
}

.signup-settings-hint {
  margin: 0;
  font-size: 0.84rem;
}

.signup-request-groups {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.55rem;
  align-items: start;
}

.signup-request-card {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--card) 92%, #f0f6ff 8%);
  padding: 0.58rem 0.62rem;
  display: grid;
  gap: 0.45rem;
  align-content: start;
}

.signup-request-card h4 {
  margin: 0;
}

.signup-request-card > .muted,
.signup-request-card > .signup-request-list {
  margin: 0;
}

.signup-request-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.45rem;
}

.signup-request-item {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  border-radius: 9px;
  background: color-mix(in srgb, var(--card) 94%, #f6f9ff 6%);
  padding: 0.6rem 0.65rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
}

.signup-request-item.reviewed {
  opacity: 0.82;
}

.signup-request-identity {
  display: grid;
  gap: 0.3rem;
  min-width: 0;
}

.signup-request-name {
  font-size: 0.9rem;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.signup-request-roles {
  display: flex;
  flex-wrap: wrap;
  gap: 0.26rem;
}

.req-role-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  padding: 0.14rem 0.48rem 0.14rem 0.34rem;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--line) 84%, transparent 16%);
  background: transparent;
  font-size: 0.75rem;
  font-weight: 600;
  color: color-mix(in srgb, var(--ink-2) 72%, transparent 28%);
  letter-spacing: 0.01em;
}

.req-role-badge.is-preferred {
  color: var(--primary-300);
  border-color: color-mix(in srgb, var(--primary-500) 55%, var(--line) 45%);
  background: color-mix(in srgb, var(--primary-700) 20%, transparent 80%);
  font-weight: 700;
}

.req-role-icon {
  font-size: 0.86rem;
  line-height: 1;
}

.req-role-sep {
  color: color-mix(in srgb, currentColor 44%, transparent 56%);
  font-size: 0.65rem;
}

.signup-request-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 0.3rem;
}

.signup-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  white-space: nowrap;
  font-size: 0.82rem;
  padding: 0.34rem 0.68rem;
}

.signup-action-btn .material-symbols-rounded {
  font-size: 1rem;
  line-height: 1;
}

.signup-decline-btn {
  color: color-mix(in srgb, var(--ink-2) 90%, transparent 10%);
}

@media (max-width: 960px) {
  .signup-request-groups {
    grid-template-columns: 1fr;
  }

  .signup-link-row {
    grid-template-columns: 1fr;
  }

  .signup-visibility-row {
    flex-direction: column;
    align-items: flex-start;
  }
}

.req-accounts-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.26rem;
  margin-top: 0.1rem;
}

.req-account-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.22rem;
  border-radius: var(--radius-pill);
  padding: 0.1rem 0.4rem;
  font-size: 0.68rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  border: 1px solid;
}

.req-account-chip--discord {
  border-color: color-mix(in srgb, #5865f2 42%, var(--line) 58%);
  color: color-mix(in srgb, #adb3ff 85%, white 15%);
  background: color-mix(in srgb, #5865f2 8%, transparent 92%);
}

.req-account-chip--bnet {
  border-color: color-mix(in srgb, #148eff 42%, var(--line) 58%);
  color: color-mix(in srgb, #74bbff 85%, white 15%);
  background: color-mix(in srgb, #148eff 8%, transparent 92%);
}

.req-account-chip--verified {
  border-color: color-mix(in srgb, currentColor 55%, var(--line) 45%);
}

.req-account-icon {
  width: 0.68rem;
  height: 0.68rem;
  fill: currentColor;
  flex-shrink: 0;
}

.req-account-verified {
  display: inline-flex;
  align-items: center;
  opacity: 0.9;
}

.req-account-verified .material-symbols-rounded {
  font-size: 0.76rem;
  line-height: 1;
  font-variation-settings: 'FILL' 1, 'wght' 700, 'GRAD' 0, 'opsz' 24;
}
</style>
