<template>
  <main class="app-shell static-page-shell">
    <header class="page-header">
      <h1 class="page-title">FAQ</h1>
      <p class="muted page-subtitle">Quick answers for common Tornare questions.</p>
    </header>

    <section class="card static-page-card">
      <h2>How do I create an event?</h2>
      <p class="muted">Open the Events page and use the Create Event action in the header.</p>

      <h2>Can I invite players with a link?</h2>
      <p class="muted">Yes. Event owners can copy and share signup links in the Requests and Settings sections.</p>

      <h2>How are teams balanced?</h2>
      <p class="muted">Owners can auto-balance teams based on role targets and player rank ELO.</p>
    </section>
  </main>
</template>

<style scoped>
.static-page-shell {
  max-width: 1120px;
}

.page-subtitle {
  margin: 0;
  text-align: right;
}

.static-page-card {
  display: grid;
  gap: 0.7rem;
}

.static-page-card h2,
.static-page-card p {
  margin: 0;
}
</style>
