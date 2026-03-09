<script setup>
import { computed, inject } from 'vue'

const ctx = inject('eventCtx')

const pendingRequests = computed(() => {
  const requests = Array.isArray(ctx.signupRequests) ? ctx.signupRequests : []
  return requests.filter((request) => request.status === 'pending')
})

const reviewedRequests = computed(() => {
  const requests = Array.isArray(ctx.signupRequests) ? ctx.signupRequests : []
  return requests.filter((request) => request.status !== 'pending')
})
</script>

<template>
  <section>
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">mail</span>
      <span>Signup Requests</span>
    </h3>

    <div class="signup-link-box">
      <p class="muted">Share this public link so players can request to join this event.</p>
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
            <div class="signup-request-main">
              <strong>{{ request.name }}</strong>
              <span class="muted">{{ request.role }} · {{ request.rank }}</span>
            </div>
            <div class="signup-request-actions">
              <button
                class="btn-primary"
                :disabled="Boolean(ctx.reviewingSignupRequests[request.id])"
                @click="ctx.acceptSignupRequest(request.id)"
              >
                {{ ctx.reviewingSignupRequests[request.id] ? 'Saving...' : 'Accept' }}
              </button>
              <button
                class="btn-danger"
                :disabled="Boolean(ctx.reviewingSignupRequests[request.id])"
                @click="ctx.declineSignupRequest(request.id)"
              >
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
            <div class="signup-request-main">
              <strong>{{ request.name }}</strong>
              <span class="muted">{{ request.role }} · {{ request.rank }}</span>
            </div>
            <span class="status-badge" :class="request.status">{{ request.status }}</span>
          </li>
        </ul>
      </article>
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

.section-title-icon {
  font-size: 1.12rem;
  line-height: 1;
}

.signup-link-box {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  border-radius: 10px;
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

.signup-request-groups {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.55rem;
  align-items: start;
}

.signup-request-card {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-1) 10%);
  border-radius: 10px;
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
  padding: 0.5rem;
  display: flex;
  justify-content: space-between;
  gap: 0.6rem;
  align-items: center;
}

.signup-request-item.reviewed {
  opacity: 0.88;
}

.signup-request-main {
  display: grid;
  gap: 0.2rem;
}

.signup-request-actions {
  display: flex;
  gap: 0.4rem;
}

.signup-request-actions .btn-primary,
.signup-request-actions .btn-danger {
  min-width: 78px;
}

.status-badge {
  border-radius: 999px;
  padding: 0.14rem 0.48rem;
  border: 1px solid var(--line);
  text-transform: uppercase;
  font-size: 0.75rem;
  font-weight: 700;
}

.status-badge.accepted {
  color: var(--ok-ink);
  background: color-mix(in srgb, var(--ok-bg) 60%, transparent 40%);
  border-color: color-mix(in srgb, var(--ok-bg) 72%, var(--line) 28%);
}

.status-badge.declined {
  color: var(--err-ink);
  background: color-mix(in srgb, var(--err-bg) 62%, transparent 38%);
  border-color: color-mix(in srgb, var(--err-bg) 72%, var(--line) 28%);
}

@media (max-width: 960px) {
  .signup-request-groups {
    grid-template-columns: 1fr;
  }

  .signup-link-row {
    grid-template-columns: 1fr;
  }
}
</style>
