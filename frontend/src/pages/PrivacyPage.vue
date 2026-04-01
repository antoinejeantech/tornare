<template>
  <main class="legal-shell">
    <header class="legal-header">
      <h1 class="legal-title">Privacy Policy</h1>
      <p class="legal-meta">Last updated: April 1, 2026 &mdash; Effective immediately</p>
    </header>

    <div class="legal-body card">

      <p class="legal-intro">
        Tornare (&ldquo;we&rdquo;, &ldquo;us&rdquo;, or &ldquo;our&rdquo;) operates the Tornare platform, a community tournament
        management tool. This Privacy Policy explains what information we collect, how we use it,
        and what your rights are. By using Tornare, you agree to the practices described here.
      </p>

      <section class="legal-section">
        <h2>1. Information We Collect</h2>

        <h3>1.1 Account Information</h3>
        <p>When you register, we collect a username, an email address, and a hashed password. If
        you link a third-party identity (Discord or Battle.net), we additionally store your
        platform username and a stable provider identifier. We never store your third-party
        passwords or access tokens beyond the duration of a session.</p>

        <h3>1.2 Profile Data</h3>
        <p>You may optionally provide a display name and a profile avatar. Avatars are restricted
        to preset images hosted on our domain and are never uploaded from your device.</p>

        <h3>1.3 Event &amp; Participation Data</h3>
        <p>We record the events you create, your signup requests (name, selected roles, rank
        self-assessment, and any voluntarily supplied contact handles), team assignments, and
        match results. This information is necessary for the service to function.</p>

        <h3>1.4 Usage Data</h3>
        <p>We may collect standard server logs including IP addresses, HTTP request paths, and
        timestamps for the purposes of security monitoring, abuse prevention, and performance
        diagnosis. These logs are retained for a maximum of 90 days.</p>
      </section>

      <section class="legal-section">
        <h2>2. How We Use Your Information</h2>
        <ul>
          <li><strong>Providing the Service</strong> &mdash; Powering event creation, signup management, team balancing, and match scheduling.</li>
          <li><strong>Account Security</strong> &mdash; Verifying your identity, detecting suspicious activity, and sending security-critical notifications.</li>
          <li><strong>Communication</strong> &mdash; Sending transactional messages related to your account or events you are participating in. We do not send marketing emails without explicit consent.</li>
          <li><strong>Improvement</strong> &mdash; Aggregated, anonymised usage patterns may be analysed to improve product quality. This analysis does not identify individuals.</li>
        </ul>
      </section>

      <section class="legal-section">
        <h2>3. Information Sharing</h2>
        <p>We do not sell, rent, or trade your personal information to third parties.</p>
        <p>We may disclose information in the following limited circumstances:</p>
        <ul>
          <li><strong>Hosting infrastructure</strong> &mdash; Our cloud infrastructure provider processes data on our behalf under a data processing agreement.</li>
          <li><strong>Legal obligation</strong> &mdash; If required by applicable law, court order, or governmental authority.</li>
          <li><strong>Protection of rights</strong> &mdash; To investigate, prevent, or act against fraud, abuse, or violations of these policies.</li>
        </ul>
        <p>Event data that you mark as public (event name, date, format, and results) is visible to
        other users of the platform. Signup contact handles (Discord tag, Battle.net tag) are
        only visible to event owners and administrators.</p>
      </section>

      <section class="legal-section">
        <h2>4. Data Retention</h2>
        <p>We retain your account data for as long as your account remains active. Event and
        participation records are retained indefinitely to preserve community history. You may
        request deletion of your account and associated personal data at any time; event
        records will be anonymised rather than deleted to preserve tournament history integrity.</p>
      </section>

      <section class="legal-section">
        <h2>5. Cookies &amp; Local Storage</h2>
        <p>Tornare uses a single authentication token stored in a secure, HTTP-only cookie to
        keep you logged in. No advertising or analytics cookies are set. We do not use any
        third-party tracking pixels or fingerprinting scripts.</p>
      </section>

      <section class="legal-section">
        <h2>6. Your Rights</h2>
        <p>Depending on your jurisdiction, you may have the following rights regarding your
        personal information:</p>
        <ul>
          <li><strong>Access</strong> &mdash; Request a copy of the personal data we hold about you.</li>
          <li><strong>Rectification</strong> &mdash; Correct inaccurate or incomplete data via your profile settings.</li>
          <li><strong>Deletion</strong> &mdash; Request erasure of your account and personal data.</li>
          <li><strong>Portability</strong> &mdash; Receive your data in a structured, machine-readable format.</li>
          <li><strong>Objection</strong> &mdash; Object to processing based on legitimate interests.</li>
        </ul>
        <p>To exercise any of these rights, contact us through the Support page. We will respond
        within 30 days.</p>
      </section>

      <section class="legal-section">
        <h2>7. Children&rsquo;s Privacy</h2>
        <p>Tornare is not directed at children under 13. We do not knowingly collect personal
        information from children under 13. If you believe we have inadvertently collected
        such information, please contact us immediately and we will delete it promptly.</p>
      </section>

      <section class="legal-section">
        <h2>8. Changes to This Policy</h2>
        <p>We may update this Privacy Policy from time to time. When we do, we will revise the
        &ldquo;Last updated&rdquo; date at the top of this page and, for material changes, post a notice
        on the platform. Continued use after changes constitutes acceptance of the updated policy.</p>
      </section>

      <section class="legal-section">
        <h2>9. Contact</h2>
        <p>For privacy questions, data requests, or concerns, please reach out through the
        <a href="/support" class="legal-link">Support page</a>.</p>
      </section>

    </div>
  </main>
</template>

<style scoped>
.legal-shell {
  max-width: 780px;
  margin: 0 auto;
  padding: 2.5rem 1rem 4rem;
  display: grid;
  gap: 1.8rem;
  animation: rise-in 300ms ease-out;
}

@keyframes rise-in {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}

.legal-header {
  display: grid;
  gap: 0.35rem;
}

.legal-title {
  font-family: var(--font-heading);
  font-size: 2rem;
  font-weight: 900;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--ink-1);
  margin: 0;
}

.legal-meta {
  font-size: 0.82rem;
  color: var(--ink-muted);
  margin: 0;
}

.legal-body {
  display: grid;
  gap: 0;
  padding: 2rem 2rem;
}

.legal-intro {
  font-size: 0.97rem;
  line-height: 1.7;
  color: var(--ink-2);
  margin: 0 0 1.8rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid var(--line);
}

.legal-section {
  padding: 1.5rem 0;
  border-bottom: 1px solid var(--line);
}

.legal-section:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.legal-section h2 {
  font-family: var(--font-heading);
  font-size: 1.05rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--ink-1);
  margin: 0 0 0.9rem;
}

.legal-section h3 {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink-1);
  margin: 1.1rem 0 0.35rem;
}

.legal-section p {
  font-size: 0.92rem;
  line-height: 1.7;
  color: var(--ink-2);
  margin: 0 0 0.65rem;
}

.legal-section p:last-child {
  margin-bottom: 0;
}

.legal-section ul {
  margin: 0.4rem 0 0.65rem 1.1rem;
  padding: 0;
  display: grid;
  gap: 0.45rem;
}

.legal-section li {
  font-size: 0.92rem;
  line-height: 1.65;
  color: var(--ink-2);
}

.legal-link {
  color: var(--brand-1);
  text-decoration: none;
}

.legal-link:hover {
  text-decoration: underline;
}
</style>
