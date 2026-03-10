<template>
  <main class="app-shell static-page-shell">
    <header class="page-header">
      <h1 class="page-title">Terms Of Service</h1>
      <p class="muted page-subtitle">Basic rules for using Tornare.</p>
    </header>

    <section class="card static-page-card">
      <h2>Usage</h2>
      <p class="muted">By using Tornare, you agree to use the platform for lawful community event coordination.</p>

      <h2>Accounts</h2>
      <p class="muted">You are responsible for activity tied to your account and for keeping your credentials secure.</p>

      <h2>Service Changes</h2>
      <p class="muted">Features may evolve over time to improve reliability and event operations.</p>
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
