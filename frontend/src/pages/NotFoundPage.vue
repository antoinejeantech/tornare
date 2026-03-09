<script setup>
import { RouterLink } from 'vue-router'
import winstonImage from '../assets/branding/winston.png'
</script>

<template>
  <main class="app-shell not-found-shell">
    <section class="card not-found-card">
      <p class="not-found-eyebrow">Signal Lost</p>
      <h1 class="not-found-title">404</h1>
      <h2 class="not-found-subtitle">Winston Could Not Locate This Page</h2>
      <p class="muted not-found-copy">
        The route you requested is not in the mission log. Return to home base or jump straight to the Event Hub.
      </p>

      <div class="not-found-actions">
        <RouterLink class="btn-primary" to="/">Go Home</RouterLink>
        <RouterLink class="btn-secondary" to="/events">Open Event Hub</RouterLink>
      </div>

      <img class="not-found-art" :src="winstonImage" alt="Winston illustration" />
    </section>
  </main>
</template>

<style scoped>
.not-found-shell {
  max-width: 1820px;
  width: min(96vw, 1820px);
}

.not-found-card {
  position: relative;
  overflow: hidden;
  min-height: clamp(320px, 42vh, 460px);
  padding: clamp(1rem, 2vw, 1.6rem);
  padding-right: clamp(1rem, 33vw, 20rem);
  display: grid;
  align-content: start;
  gap: 0.5rem;
  border-color: color-mix(in srgb, var(--brand-1) 36%, var(--line) 64%);
  background:
    radial-gradient(700px 180px at 0% 0%, rgba(255, 255, 255, 0.08), transparent 64%),
    linear-gradient(145deg, color-mix(in srgb, var(--card) 92%, #2f2f2f 8%), var(--card));
}

.not-found-eyebrow {
  margin: 0;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-size: 0.78rem;
  font-weight: 700;
}

.not-found-title {
  margin: 0;
  font-size: clamp(2.9rem, 6vw, 5.8rem);
  line-height: 0.9;
  color: color-mix(in srgb, var(--heading-ink) 90%, #fff 10%);
}

.not-found-subtitle {
  margin: 0;
  font-size: clamp(1.05rem, 1.3vw + 0.75rem, 1.8rem);
  letter-spacing: -0.01em;
  max-width: 22ch;
}

.not-found-copy {
  margin: 0.25rem 0 0;
  max-width: 52ch;
  line-height: 1.48;
}

.not-found-actions {
  margin-top: 0.9rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.not-found-actions :deep(a) {
  text-decoration: none;
}

.not-found-art {
  position: absolute;
  right: clamp(0.35rem, 1.8vw, 1rem);
  bottom: 0;
  width: clamp(210px, 30vw, 420px);
  max-height: 96%;
  object-fit: contain;
  pointer-events: none;
  filter: drop-shadow(0 10px 28px rgba(0, 0, 0, 0.4));
  opacity: 0.95;
}

@media (max-width: 940px) {
  .not-found-card {
    padding-right: 1rem;
    padding-bottom: clamp(8rem, 34vw, 11rem);
  }

  .not-found-art {
    right: 50%;
    transform: translateX(50%);
    width: clamp(170px, 56vw, 280px);
    opacity: 0.8;
  }
}
</style>
