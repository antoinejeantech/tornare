<script setup>
const placeholderNews = [
  {
    id: 'placeholder-1',
    title: 'Upcoming Feature Drop',
    date: 'March 2026',
    summary: 'Placeholder post for upcoming quality-of-life updates across event setup and match management.'
  },
  {
    id: 'placeholder-2',
    title: 'Community Spotlight',
    date: 'March 2026',
    summary: 'Placeholder post for future community highlights and tournament recaps.'
  },
  {
    id: 'placeholder-3',
    title: 'Patch Notes Format',
    date: 'March 2026',
    summary: 'Placeholder post showing where backend and frontend changelog notes will be published.'
  }
]
</script>

<template>
  <main class="app-shell">
    <header class="page-header">
      <h1 class="page-title">News</h1>
    </header>

    <section class="card">
      <h2>Latest Updates</h2>
      <p class="muted">This page currently uses placeholder articles.</p>
      <ul class="news-list">
        <li v-for="item in placeholderNews" :key="item.id" class="news-item">
          <div class="news-item-header">
            <h3>{{ item.title }}</h3>
            <span class="muted">{{ item.date }}</span>
          </div>
          <p class="muted">{{ item.summary }}</p>
        </li>
      </ul>
    </section>
  </main>
</template>

<style scoped>
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
</style>
