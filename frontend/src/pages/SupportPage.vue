<script setup lang="ts">
const channels = [
  {
    id: 'discord',
    icon: 'forum',
    title: 'Community Discord',
    description: 'Fastest path to help. Post in #support and the community or a maintainer will respond.',
    label: 'Open Discord',
    href: 'https://discord.gg/tornare',
    primary: true
  },
  {
    id: 'github',
    icon: 'bug_report',
    title: 'Bug Reports',
    description: 'Found a reproducible issue? Open a GitHub issue with steps and any visible error message.',
    label: 'Open Issue',
    href: 'https://github.com/tornare/tornare/issues',
    primary: false
  },
  {
    id: 'email',
    icon: 'mail',
    title: 'Direct Contact',
    description: 'For account, privacy, or data requests that require private handling.',
    label: 'support@tornare.gg',
    href: 'mailto:support@tornare.gg',
    primary: false
  }
]

const topics = [
  {
    icon: 'event',
    title: 'Event Setup',
    tip: 'Make sure the event is published (status ACTIVE) and "Public Registration" is enabled in the Settings tab before sharing the signup link. Players cannot submit requests while the event is a Draft or after it has Ended.'
  },
  {
    icon: 'group_add',
    title: 'Signup Issues',
    tip: 'If a player cannot submit a request, verify the event is public and the link is current. Reject + re-invite resets their request.'
  },
  {
    icon: 'shuffle',
    title: 'Team Balancing',
    tip: 'Balancing uses rank ELO and role targets. Ensure all players have a rank set before running auto-balance for best results.'
  },
  {
    icon: 'manage_accounts',
    title: 'Account & Identity',
    tip: 'Linking Discord or Battle.net requires completing the OAuth flow from your Profile settings page. Check browser pop-up blockers if it fails.'
  }
]
</script>

<template>
  <main class="support-shell">

    <header class="support-header">
      <span class="material-symbols-rounded support-header-icon" aria-hidden="true">support_agent</span>
      <div>
        <h1 class="support-title">Support</h1>
        <p class="support-subtitle">Get help with events, accounts, and platform issues.</p>
      </div>
    </header>

    <!-- Contact channels -->
    <section class="support-channels">
      <article
        v-for="ch in channels"
        :key="ch.id"
        class="card support-channel"
        :class="{ 'support-channel--primary': ch.primary }"
      >
        <span class="material-symbols-rounded support-channel-icon" aria-hidden="true">{{ ch.icon }}</span>
        <div class="support-channel-body">
          <h2>{{ ch.title }}</h2>
          <p>{{ ch.description }}</p>
        </div>
        <a :href="ch.href" target="_blank" rel="noopener noreferrer" class="support-channel-link">
          {{ ch.label }}
          <span class="material-symbols-rounded" aria-hidden="true" style="font-size:0.9rem">arrow_outward</span>
        </a>
      </article>
    </section>

    <!-- Common topics -->
    <section class="card support-topics">
      <h2 class="support-section-title">Common Topics</h2>
      <div class="support-topics-grid">
        <div v-for="t in topics" :key="t.icon" class="support-topic">
          <span class="material-symbols-rounded support-topic-icon" aria-hidden="true">{{ t.icon }}</span>
          <div>
            <strong>{{ t.title }}</strong>
            <p>{{ t.tip }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Bug report guide -->
    <section class="card support-bug">
      <div class="support-bug-head">
        <span class="material-symbols-rounded support-bug-icon" aria-hidden="true">checklist</span>
        <h2>Ideal Bug Report</h2>
      </div>
      <p class="muted support-bug-intro">Include all of the following to get the fastest resolution:</p>
      <ol class="support-bug-list">
        <li><strong>What you were trying to do</strong> &mdash; a one-sentence description of the intended action.</li>
        <li><strong>Steps to reproduce</strong> &mdash; a numbered, repeatable sequence from a clean state.</li>
        <li><strong>What actually happened</strong> &mdash; the result, including any error text shown on screen.</li>
        <li><strong>Event ID or URL</strong> &mdash; copy-paste from the browser address bar.</li>
        <li><strong>Browser &amp; device</strong> &mdash; e.g. Chrome 124 on macOS 14.</li>
      </ol>
    </section>

    <!-- Response expectations -->
    <section class="support-response-grid">
      <div class="card support-response-item">
        <span class="material-symbols-rounded support-response-icon" aria-hidden="true">schedule</span>
        <strong>Community Discord</strong>
        <p class="muted">Usually a few hours during active community times. Best for general questions.</p>
      </div>
      <div class="card support-response-item">
        <span class="material-symbols-rounded support-response-icon" aria-hidden="true">policy</span>
        <strong>Privacy &amp; Data Requests</strong>
        <p class="muted">We respond within 30 days as required by applicable law. Email preferred for privacy matters.</p>
      </div>
      <div class="card support-response-item">
        <span class="material-symbols-rounded support-response-icon" aria-hidden="true">construction</span>
        <strong>Bug &amp; Feature Backlog</strong>
        <p class="muted">Tracked on GitHub. High-impact bugs are prioritised in the next release cycle.</p>
      </div>
    </section>

  </main>
</template>

<style scoped>
.support-shell {
  max-width: 820px;
  margin-inline: auto;
  padding: 2rem 1.25rem 4rem;
  display: grid;
  gap: 1.1rem;
}

.support-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 0 0.25rem;
}

.support-header-icon {
  font-size: 2.6rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
  flex-shrink: 0;
}

.support-title {
  margin: 0;
  font-size: clamp(1.6rem, 3vw, 2.2rem);
  letter-spacing: -0.02em;
}

.support-subtitle {
  margin: 0.15rem 0 0;
  color: var(--ink-2);
  font-size: 1rem;
}

/* ── Channels ──────────────────────────────────────── */
.support-channels {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.88rem;
}

.support-channel {
  display: grid;
  gap: 0.4rem;
  align-content: start;
}

.support-channel--primary {
  border-color: color-mix(in srgb, var(--brand-1) 30%, var(--line) 70%);
  background: color-mix(in srgb, var(--card) 88%, #1f2a1a 12%);
}

.support-channel-icon {
  font-size: 1.6rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
}

.support-channel-body h2 {
  margin: 0;
  font-size: 1rem;
}

.support-channel-body p {
  margin: 0.2rem 0 0;
  font-size: 0.87rem;
  color: var(--ink-2);
  line-height: 1.5;
}

.support-channel-link {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  margin-top: 0.35rem;
  color: var(--brand-1);
  font-weight: 700;
  font-size: 0.8rem;
  text-decoration: none;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.support-channel-link:hover {
  text-decoration: underline;
}

/* ── Topics ────────────────────────────────────────── */
.support-section-title {
  margin: 0 0 0.8rem;
}

.support-topics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.support-topic {
  display: flex;
  gap: 0.65rem;
  align-items: flex-start;
  padding: 0.65rem 0.7rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--card) 92%, #191e28 8%);
}

.support-topic-icon {
  font-size: 1.35rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
  flex-shrink: 0;
  margin-top: 0.1rem;
}

.support-topic strong {
  display: block;
  font-size: 0.9rem;
  margin-bottom: 0.18rem;
}

.support-topic p {
  margin: 0;
  font-size: 0.84rem;
  color: var(--ink-2);
  line-height: 1.5;
}

/* ── Bug report ────────────────────────────────────── */
.support-bug-head {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.35rem;
}

.support-bug-icon {
  font-size: 1.5rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
}

.support-bug-head h2 {
  margin: 0;
}

.support-bug-intro {
  margin: 0 0 0.75rem;
  font-size: 0.9rem;
}

.support-bug-list {
  margin: 0;
  padding-left: 1.3rem;
  display: grid;
  gap: 0.5rem;
}

.support-bug-list li {
  font-size: 0.89rem;
  color: var(--ink-2);
  line-height: 1.55;
}

.support-bug-list li strong {
  color: var(--ink-1);
}

/* ── Response grid ─────────────────────────────────── */
.support-response-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.88rem;
}

.support-response-item {
  display: grid;
  gap: 0.28rem;
  align-content: start;
}

.support-response-icon {
  font-size: 1.3rem;
  font-variation-settings: 'FILL' 1;
  color: var(--ink-3);
  margin-bottom: 0.05rem;
}

.support-response-item strong {
  font-size: 0.91rem;
}

.support-response-item p {
  margin: 0;
  font-size: 0.82rem;
  line-height: 1.5;
}

@media (max-width: 720px) {
  .support-channels,
  .support-response-grid {
    grid-template-columns: 1fr;
  }

  .support-topics-grid {
    grid-template-columns: 1fr;
  }
}
</style>
