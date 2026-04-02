<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import tornareLogo from '../assets/branding/tornare-logo-pulse.svg'

const { t } = useI18n()
</script>

<template>
  <footer class="app-footer">
    <div class="app-footer-inner">
      <!-- Left: brand -->
      <div class="app-footer-brand">
        <RouterLink class="app-footer-logo-link" to="/" :aria-label="t('nav.tornareHome')">
          <img class="app-footer-logo" :src="tornareLogo" alt="" aria-hidden="true" />
          <span class="app-footer-wordmark">tornare</span>
        </RouterLink>
        <p class="app-footer-tagline">{{ t('footer.tagline') }}</p>
        <div class="app-footer-social">
          <a
            href="https://discord.gg/tornare"
            target="_blank"
            rel="noopener noreferrer"
            class="app-footer-social-link"
            :aria-label="t('footer.joinDiscord')"
          >
            <svg class="app-footer-social-icon" viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
              <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057.1 18.078.112 18.1.13 18.11a19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>
            </svg>
          </a>
          <a
            href="https://github.com/antoinejeantech/tornare"
            target="_blank"
            rel="noopener noreferrer"
            class="app-footer-social-link"
            :aria-label="t('footer.viewGithub')"
          >
            <svg class="app-footer-social-icon" viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
              <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2z"/>
            </svg>
          </a>
        </div>
      </div>

      <!-- Right: grouped links -->
      <div class="app-footer-links-groups">
        <div class="app-footer-group">
          <span class="app-footer-group-label">{{ t('footer.platform') }}</span>
          <RouterLink to="/events">{{ t('nav.events') }}</RouterLink>
          <RouterLink to="/about">{{ t('nav.about') }}</RouterLink>
          <RouterLink to="/news">{{ t('nav.news') }}</RouterLink>
        </div>
        <div class="app-footer-group">
          <span class="app-footer-group-label">{{ t('footer.help') }}</span>
          <RouterLink to="/faq">{{ t('footer.faq') }}</RouterLink>
          <RouterLink to="/support">{{ t('footer.support') }}</RouterLink>
        </div>
        <div class="app-footer-group">
          <span class="app-footer-group-label">{{ t('footer.legal') }}</span>
          <RouterLink to="/privacy">{{ t('footer.privacy') }}</RouterLink>
          <RouterLink to="/terms">{{ t('footer.terms') }}</RouterLink>
        </div>
      </div>
    </div>

    <div class="app-footer-bottom">
      <span>{{ t('footer.copyright') }}</span>
    </div>
  </footer>
</template>

<style>
.app-footer {
  margin-top: auto;
  border-top: 1px solid var(--line);
  background: color-mix(in srgb, var(--bg-0) 72%, var(--card) 28%);
}

.app-footer-inner {
  max-width: 1020px;
  margin: 0 auto;
  padding: 2.5rem 1.5rem 1.5rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 2.5rem;
  flex-wrap: wrap;
}

/* ── Brand column ── */
.app-footer-brand {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  max-width: 240px;
}

.app-footer-logo-link {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  text-decoration: none;
}

.app-footer-logo {
  width: 1.6rem;
  height: 1.6rem;
}

.app-footer-wordmark {
  font-family: var(--font-heading);
  font-size: 1.05rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--ink-1);
}

.app-footer-tagline {
  margin: 0;
  font-size: 0.8rem;
  color: var(--ink-muted);
  line-height: 1.5;
}

.app-footer-social {
  display: flex;
  gap: 0.55rem;
  margin-top: 0.2rem;
}

.app-footer-social-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--line);
  color: var(--ink-muted);
  background: transparent;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
}

.app-footer-social-link:hover {
  color: var(--ink-1);
  border-color: var(--line-strong);
  background: color-mix(in srgb, var(--card) 60%, transparent 40%);
}

.app-footer-social-icon {
  width: 1rem;
  height: 1rem;
}

/* ── Link groups ── */
.app-footer-links-groups {
  display: flex;
  gap: 2.5rem;
  flex-wrap: wrap;
}

.app-footer-group {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}

.app-footer-group-label {
  font-size: 0.68rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-muted);
  margin-bottom: 0.1rem;
}

.app-footer-group a {
  font-size: 0.83rem;
  color: var(--ink-2);
  text-decoration: none;
  transition: color 0.14s;
}

.app-footer-group a:hover {
  color: var(--ink-1);
}

/* ── Copyright bar ── */
.app-footer-bottom {
  max-width: 1020px;
  margin: 0 auto;
  padding: 0.9rem 1.5rem;
  border-top: 1px solid var(--line);
  font-size: 0.75rem;
  color: var(--ink-muted);
}

/* On EventPage mobile the bottom tab bar is fixed — add padding so the
   copyright bar clears it rather than being hidden behind it. */
@media (max-width: 900px) {
  .app-root:has(.event-bottom-nav) .app-footer-bottom {
    padding-bottom: calc(4.5rem + env(safe-area-inset-bottom, 0px));
  }
}

@media (max-width: 640px) {
  .app-footer-inner {
    padding: 2rem 1.1rem 1.5rem;
    flex-direction: column;
    gap: 2rem;
  }

  .app-footer-brand {
    max-width: none;
  }

  .app-footer-links-groups {
    gap: 2rem;
  }

  .app-footer-bottom {
    padding: 0.9rem 1.1rem;
  }
}
</style>
