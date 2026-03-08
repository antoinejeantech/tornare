<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

const placeholderNews = [
  {
    id: 'placeholder-1',
    title: 'Upcoming Feature Drop',
    date: 'March 2026',
    category: 'Product',
    summary: 'Quality-of-life updates for event setup, roster handling, and faster match-room preparation.',
    impact: 'Faster organizer workflow'
  },
  {
    id: 'placeholder-2',
    title: 'Community Spotlight',
    date: 'March 2026',
    category: 'Community',
    summary: 'Tournament recap format and best-practice highlights from active Tornare communities.',
    impact: 'Share repeatable playbooks'
  },
  {
    id: 'placeholder-3',
    title: 'Patch Notes Format',
    date: 'March 2026',
    category: 'Engineering',
    summary: 'A transparent release template for backend/frontend changes, migrations, and known issues.',
    impact: 'Clearer release communication'
  },
  {
    id: 'placeholder-4',
    title: 'Organizer Toolkit Preview',
    date: 'March 2026',
    category: 'Product',
    summary: 'Preview of upcoming organizer-centric controls to streamline pre-match and check-in operations.',
    impact: 'Less pre-game friction'
  }
]

const featuredPost = computed(() => placeholderNews[0] || null)
const remainingPosts = computed(() => placeholderNews.slice(1))
const productNewsCount = computed(() => placeholderNews.filter((item) => item.category === 'Product').length)
</script>

<template>
  <main class="app-shell news-shell">
    <header class="page-header">
      <h1 class="page-title">Tornare Newsroom</h1>
    </header>

    <section class="card news-hero reveal-block reveal-1">
      <p class="news-eyebrow">Release Intelligence</p>
      <h2>Product updates, community recaps, and shipping notes in one feed.</h2>
      <p class="muted">
        This page is currently curated with structured placeholder content, ready to be replaced by real posts as launch cadence grows.
      </p>
      <div class="news-meta-strip">
        <span class="news-meta-pill">{{ placeholderNews.length }} total stories</span>
        <span class="news-meta-pill">{{ productNewsCount }} product updates</span>
        <span class="news-meta-pill">Updated March 2026</span>
      </div>
    </section>

    <section v-if="featuredPost" class="card news-featured reveal-block reveal-2">
      <div class="news-featured-head">
        <span class="news-featured-badge">Featured Story</span>
        <span class="news-category-chip">{{ featuredPost.category }}</span>
      </div>
      <h2>{{ featuredPost.title }}</h2>
      <p class="muted">{{ featuredPost.summary }}</p>
      <div class="news-featured-foot">
        <span class="news-impact">{{ featuredPost.impact }}</span>
        <span class="muted">{{ featuredPost.date }}</span>
      </div>
    </section>

    <section class="card reveal-block reveal-3">
      <div class="news-list-head">
        <h2>Latest Updates</h2>
        <RouterLink class="news-link" to="/events">See platform in action</RouterLink>
      </div>
      <ul class="news-list">
        <li v-for="item in remainingPosts" :key="item.id" class="news-item">
          <div class="news-item-header">
            <h3>{{ item.title }}</h3>
            <span class="news-category-chip">{{ item.category }}</span>
          </div>
          <p class="muted">{{ item.summary }}</p>
          <div class="news-item-foot">
            <span class="news-impact">{{ item.impact }}</span>
            <span class="muted">{{ item.date }}</span>
          </div>
        </li>
      </ul>
    </section>
  </main>
</template>

<style scoped>
.news-shell {
  max-width: 1820px;
  width: min(96vw, 1820px);
  display: grid;
  gap: 0.88rem;
}

.news-hero {
  display: grid;
  gap: 0.55rem;
  border-color: color-mix(in srgb, var(--brand-2) 34%, var(--line) 66%);
  background:
    radial-gradient(1000px 130px at 0% 0%, rgba(66, 133, 244, 0.16), transparent 60%),
    color-mix(in srgb, var(--card) 92%, #edf5ff 8%);
}

.news-hero h2,
.news-hero p {
  margin: 0;
}

.news-eyebrow {
  margin: 0;
  color: var(--accent);
  font-family: "Space Mono", ui-monospace, monospace;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-size: 0.78rem;
  font-weight: 700;
}

.news-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.news-meta-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.news-meta-pill {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--brand-1) 34%, var(--line) 66%);
  background: color-mix(in srgb, var(--accent) 18%, var(--meta-bg) 82%);
  color: var(--meta-ink);
  padding: 0.18rem 0.58rem;
  font-size: 0.74rem;
  font-family: "Space Mono", ui-monospace, monospace;
  font-weight: 700;
}

.news-featured {
  display: grid;
  gap: 0.45rem;
}

.news-featured h2,
.news-featured p {
  margin: 0;
}

.news-featured-head,
.news-featured-foot,
.news-item-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.news-featured-badge {
  font-size: 0.72rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--brand-1);
}

.news-category-chip {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--line) 84%, var(--brand-1) 16%);
  background: color-mix(in srgb, var(--card) 90%, #edf4ff 10%);
  color: var(--ink-2);
  padding: 0.14rem 0.5rem;
  font-size: 0.72rem;
  font-weight: 800;
}

.news-impact {
  font-size: 0.76rem;
  color: var(--brand-1);
  font-weight: 700;
}

.news-list-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.7rem;
}

.news-list-head h2 {
  margin: 0;
}

.news-link {
  color: var(--brand-1);
  font-weight: 800;
  text-decoration: none;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-size: 0.82rem;
}

.news-link:hover {
  text-decoration: underline;
}

.news-list {
  list-style: none;
  margin: 0.55rem 0 0;
  padding: 0;
  display: grid;
  gap: 0.6rem;
}

.news-item {
  border: 1px solid color-mix(in srgb, var(--line) 90%, var(--brand-2) 10%);
  border-radius: 10px;
  background: color-mix(in srgb, var(--card) 92%, #eef5ff 8%);
  padding: 0.62rem 0.7rem;
  display: grid;
  gap: 0.35rem;
}

.news-item h3 {
  margin: 0;
  font-size: 1.03rem;
}

.news-item p {
  margin: 0.35rem 0 0;
}

.news-item-header {
  display: flex;
  justify-content: space-between;
  gap: 0.7rem;
  align-items: baseline;
}

.reveal-block {
  opacity: 0;
  transform: translateY(10px);
  animation: reveal-rise 380ms ease-out forwards;
}

.reveal-1 { animation-delay: 60ms; }
.reveal-2 { animation-delay: 120ms; }
.reveal-3 { animation-delay: 180ms; }

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 980px) {
  .news-list-head,
  .news-item-header,
  .news-featured-head,
  .news-featured-foot,
  .news-item-foot {
    flex-wrap: wrap;
  }
}
</style>
