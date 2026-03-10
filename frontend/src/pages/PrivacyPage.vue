<template>
  <main class="app-shell static-page-shell">
    <header class="page-header">
      <h1 class="page-title">Privacy Policy</h1>
      <p class="muted page-subtitle">How Tornare handles account and event data.</p>
    </header>

    <section class="card static-page-card">
      <h2>Data We Collect</h2>
      <p class="muted">Tornare stores account profile data, event information, and participation records needed to run community events.</p>

      <h2>How We Use Data</h2>
      <p class="muted">Your data is used to power event setup, signup requests, team assignment, and match management features.</p>

      <h2>Contact</h2>
      <p class="muted">For privacy questions, contact support through the Support page.</p>
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
