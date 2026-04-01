<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'

const openIds = ref(new Set<string>())

function toggle(id: string) {
  if (openIds.value.has(id)) {
    openIds.value.delete(id)
  } else {
    openIds.value.add(id)
  }
  // trigger reactivity
  openIds.value = new Set(openIds.value)
}

const sections = [
  {
    id: 'getting-started',
    title: 'Getting Started',
    icon: 'rocket_launch',
    questions: [
      {
        id: 'gs-1',
        q: 'How do I create an account?',
        a: 'Click "Sign in" in the top navigation. You can register with a username and password, or link an existing Discord or Battle.net account via OAuth on the Auth page.'
      },
      {
        id: 'gs-2',
        q: 'How do I create an event?',
        a: 'Open the Events page and use the "Create Event" button in the header. You will be prompted to name the event, configure its format, and set signup options. Newly created events start in Draft status.'
      },
      {
        id: 'gs-3',
        q: 'How do I invite players to my event?',
        a: "From the event's Requests tab, copy the signup link and share it in your community Discord or chat. Players do not need a Tornare account to submit a signup — they just open the link and fill in the form."
      },
      {
        id: 'gs-4',
        q: 'What does "public registration" mean?',
        a: "When public registration is enabled, anyone with the signup link can request to join. When disabled, only manually approved players can access the event. Toggle this in the event's Settings tab."
      }
    ]
  },
  {
    id: 'events',
    title: 'Events & Rosters',
    icon: 'event',
    questions: [
      {
        id: 'ev-1',
        q: 'How does auto-balance work?',
        a: 'The balancing engine distributes players across teams by optimising role coverage (Tank, Support, DPS) and minimising total rank ELO difference between teams. Ensure all players have a rank set before running the balance for best results.'
      },
      {
        id: 'ev-2',
        q: 'Can I manually adjust teams after auto-balance?',
        a: 'Yes. After running auto-balance, any team slot can be edited individually from the Teams section of the event.'
      },
      {
        id: 'ev-3',
        q: 'Can a player update their signup after submitting?',
        a: "Players cannot edit a submitted request directly. As an organiser you can reject and re-invite them, which lets them submit a fresh request with updated information."
      },
      {
        id: 'ev-4',
        q: 'What are the different event statuses?',
        a: 'Draft — being set up; not visible to the public, signups not accepted. Active — live and visible; players can submit signup requests if Public Registration is enabled. Ended — event is over; still visible in Past Events but locked for new signups.'
      },
      {
        id: 'ev-5',
        q: 'Can I archive or soft-delete an event?',
        a: "Yes. From the event Settings tab you can mark an event as deleted. It remains in the database for record-keeping but is hidden from the public events list. Admins can still access it."
      }
    ]
  },
  {
    id: 'players',
    title: 'Players & Roles',
    icon: 'group',
    questions: [
      {
        id: 'pl-1',
        q: 'What roles are supported?',
        a: 'Tornare currently supports Tank, Support, and DPS — the standard Overwatch 2 role categories. Players can set primary and secondary role preferences when submitting a signup request.'
      },
      {
        id: 'pl-2',
        q: 'What are ranks and how is ELO calculated?',
        a: 'Players self-report their competitive rank (e.g. Gold 3, Platinum 1). Tornare maps these to numeric ELO values used for team balancing. The tiers follow the standard Overwatch competitive ladder.'
      },
      {
        id: 'pl-3',
        q: 'What is a "flex" role?',
        a: 'Flex indicates the player has no strong role preference and is willing to fill any position. The balance engine uses flex players as wildcards when optimising team compositions.'
      },
      {
        id: 'pl-4',
        q: 'Can a player be assigned a different role than their preference?',
        a: "Yes. Organisers can override the assigned role for any player in the Teams section. The player's preference is shown as a hint, but the final assignment is always under organiser control."
      }
    ]
  },
  {
    id: 'accounts',
    title: 'Accounts & Identity',
    icon: 'manage_accounts',
    questions: [
      {
        id: 'ac-1',
        q: 'How do I link my Discord account?',
        a: "Go to your Profile page and click \"Link Discord\". You will be redirected to Discord's OAuth screen. After authorising, your Discord username will appear on your profile and on your player cards in events."
      },
      {
        id: 'ac-2',
        q: 'How do I link my Battle.net account?',
        a: "The same flow applies: Profile → \"Link Battle.net\" → authorise on Blizzard's site. Your BattleTag will then be linked to your Tornare identity and shown on player cards."
      },
      {
        id: 'ac-3',
        q: 'Can I change my username?',
        a: 'Username changes are not currently self-served. Contact support if you need a correction. Display names can be updated freely from your Profile settings at any time.'
      },
      {
        id: 'ac-4',
        q: 'How do I delete my account?',
        a: 'Send a deletion request through the Support page. We will anonymise your personal data within 30 days. Event participation records are retained in anonymised form to preserve tournament history.'
      }
    ]
  }
]
</script>

<template>
  <main class="faq-shell">

    <header class="faq-header">
      <span class="material-symbols-rounded faq-header-icon" aria-hidden="true">help</span>
      <div>
        <h1 class="faq-title">Frequently Asked Questions</h1>
        <p class="faq-subtitle">Quick answers across events, players, teams, and accounts.</p>
      </div>
    </header>

    <section
      v-for="section in sections"
      :key="section.id"
      class="card faq-section"
    >
      <div class="faq-section-head">
        <span class="material-symbols-rounded faq-section-icon" aria-hidden="true">{{ section.icon }}</span>
        <h2 class="faq-section-title">{{ section.title }}</h2>
      </div>

      <ul class="faq-list">
        <li
          v-for="item in section.questions"
          :key="item.id"
          class="faq-item"
          :class="{ 'faq-item--open': openIds.has(item.id) }"
        >
          <button class="faq-question" @click="toggle(item.id)" :aria-expanded="openIds.has(item.id)">
            <span>{{ item.q }}</span>
            <span class="material-symbols-rounded faq-chevron" aria-hidden="true">expand_more</span>
          </button>
          <div v-if="openIds.has(item.id)" class="faq-answer">
            <p>{{ item.a }}</p>
          </div>
        </li>
      </ul>
    </section>

    <div class="faq-footer card">
      <span class="material-symbols-rounded faq-footer-icon" aria-hidden="true">live_help</span>
      <p>Didn't find what you were looking for? <RouterLink to="/support" class="faq-footer-link">Visit the Support page</RouterLink> to reach the team directly.</p>
    </div>

  </main>
</template>

<style scoped>
.faq-shell {
  width: 100%;
  max-width: 760px;
  margin-inline: auto;
  padding: 2rem 1.25rem 4rem;
  display: grid;
  gap: 1.1rem;
}

.faq-section,
.faq-footer {
  min-width: 0;
}

.faq-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 0 0.25rem;
}

.faq-header-icon {
  font-size: 2.6rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
  flex-shrink: 0;
}

.faq-title {
  margin: 0;
  font-size: clamp(1.5rem, 3vw, 2.1rem);
  letter-spacing: -0.02em;
}

.faq-subtitle {
  margin: 0.15rem 0 0;
  color: var(--ink-2);
  font-size: 1rem;
}

/* ── Section ───────────────────────────────────────── */
.faq-section {
  display: grid;
  gap: 0.6rem;
}

.faq-section-head {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.faq-section-icon {
  font-size: 1.25rem;
  font-variation-settings: 'FILL' 1;
  color: var(--brand-1);
}

.faq-section-title {
  margin: 0;
  font-size: 1.05rem;
  letter-spacing: -0.01em;
}

/* ── List ──────────────────────────────────────────── */
.faq-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.35rem;
}

.faq-item {
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: border-color 160ms;
}

.faq-item--open {
  border-color: color-mix(in srgb, var(--brand-1) 28%, var(--line) 72%);
}

.faq-question {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.65rem 0.8rem;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  color: var(--ink-1);
  font-size: 0.93rem;
  font-weight: 600;
  line-height: 1.45;
  transition: background 120ms;
}

.faq-question:hover {
  background: color-mix(in srgb, var(--brand-1) 5%, transparent 95%);
}

.faq-chevron {
  font-size: 1.15rem;
  flex-shrink: 0;
  color: var(--ink-3);
  transition: transform 200ms;
}

.faq-item--open .faq-chevron {
  transform: rotate(180deg);
}

.faq-answer {
  padding: 0 0.8rem 0.7rem;
  border-top: 1px solid var(--line);
}

.faq-answer p {
  margin: 0.6rem 0 0;
  font-size: 0.88rem;
  color: var(--ink-2);
  line-height: 1.65;
}

/* ── Footer prompt ─────────────────────────────────── */
.faq-footer {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.faq-footer-icon {
  font-size: 1.5rem;
  font-variation-settings: 'FILL' 1;
  color: var(--ink-3);
  flex-shrink: 0;
}

.faq-footer p {
  margin: 0;
  font-size: 0.9rem;
  color: var(--ink-2);
}

.faq-footer-link {
  color: var(--brand-1);
  font-weight: 700;
  text-decoration: none;
}

.faq-footer-link:hover {
  text-decoration: underline;
}
</style>
