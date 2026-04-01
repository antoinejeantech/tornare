<template>
  <main class="legal-shell">
    <header class="legal-header">
      <h1 class="legal-title">Terms of Service</h1>
      <p class="legal-meta">Last updated: April 1, 2026 &mdash; Effective immediately</p>
    </header>

    <div class="legal-body card">

      <p class="legal-intro">
        These Terms of Service (&ldquo;Terms&rdquo;) govern your access to and use of the Tornare platform.
        Please read them carefully. By creating an account or using any part of the service,
        you agree to be bound by these Terms. If you do not agree, do not use Tornare.
      </p>

      <section class="legal-section">
        <h2>1. Eligibility</h2>
        <p>You must be at least 13 years old to use Tornare. If you are under 18, you represent
        that you have your parent or guardian&rsquo;s permission. By using the service you warrant
        that you meet these requirements and that you are not prohibited from using the service
        under applicable laws.</p>
      </section>

      <section class="legal-section">
        <h2>2. Accounts</h2>
        <p>You are responsible for all activity that occurs under your account. You must provide
        accurate information when registering and must keep your credentials secure and
        confidential. You may not share your account with others or create accounts for the
        purpose of abuse, impersonation, or circumventing suspensions.</p>
        <p>We reserve the right to suspend or terminate accounts that violate these Terms,
        without prior notice, at our sole discretion.</p>
      </section>

      <section class="legal-section">
        <h2>3. Acceptable Use</h2>
        <p>You agree to use Tornare only for lawful purposes. You must not:</p>
        <ul>
          <li>Post, transmit, or distribute content that is unlawful, harassing, defamatory, obscene, or otherwise objectionable.</li>
          <li>Impersonate any person or entity, or misrepresent your affiliation with any person or entity.</li>
          <li>Attempt to gain unauthorised access to any part of the service, other accounts, or connected systems.</li>
          <li>Use automated scripts, bots, or other tools to scrape, crawl, or abuse the platform.</li>
          <li>Interfere with the integrity or performance of the service or the data it contains.</li>
          <li>Use the service to facilitate cheating, match-fixing, or fraudulent competitive outcomes.</li>
        </ul>
      </section>

      <section class="legal-section">
        <h2>4. Content You Submit</h2>
        <p>By submitting content to Tornare (including event names, player names, match results,
        and profile information), you grant us a non-exclusive, worldwide, royalty-free licence
        to store, display, and distribute that content solely for the purpose of operating
        and improving the platform. You retain ownership of your content.</p>
        <p>You are solely responsible for the accuracy of the content you submit. We do not
        verify the accuracy of event data or competitive results.</p>
      </section>

      <section class="legal-section">
        <h2>5. Third-Party Integrations</h2>
        <p>Tornare allows you to link third-party accounts (Discord, Battle.net). These
        integrations are provided for convenience. Your use of those third-party services is
        governed by their respective terms and privacy policies, not ours. We are not
        responsible for any data practices of those services.</p>
      </section>

      <section class="legal-section">
        <h2>6. Intellectual Property</h2>
        <p>The Tornare name, logo, interface design, and all platform software are the exclusive
        property of Tornare and are protected by applicable intellectual property laws. You may
        not copy, modify, or distribute any part of the platform without our prior written consent.</p>
        <p>Game titles, character names, and related assets referenced on this platform
        (including Overwatch) are the property of their respective owners. Tornare is an
        independent community tool and is not affiliated with, endorsed by, or sponsored by
        Blizzard Entertainment.</p>
      </section>

      <section class="legal-section">
        <h2>7. Disclaimer of Warranties</h2>
        <p>Tornare is provided &ldquo;as is&rdquo; and &ldquo;as available&rdquo; without warranties of any kind, either
        express or implied, including but not limited to implied warranties of merchantability,
        fitness for a particular purpose, or non-infringement. We do not guarantee that the
        service will be uninterrupted, error-free, or free of harmful components.</p>
      </section>

      <section class="legal-section">
        <h2>8. Limitation of Liability</h2>
        <p>To the maximum extent permitted by applicable law, Tornare and its operators shall
        not be liable for any indirect, incidental, special, consequential, or punitive damages
        arising from your use of or inability to use the service, even if advised of the
        possibility of such damages. Our total liability for any claim arising under these
        Terms shall not exceed the amount you paid us in the twelve months preceding the claim
        (which, for a free service, is zero).</p>
      </section>

      <section class="legal-section">
        <h2>9. Modifications to the Service</h2>
        <p>We reserve the right to modify, suspend, or discontinue the service (or any part
        thereof) at any time with or without notice. We will not be liable to you or any third
        party for any modification, suspension, or discontinuation of the service.</p>
      </section>

      <section class="legal-section">
        <h2>10. Changes to These Terms</h2>
        <p>We may revise these Terms from time to time. When we do, we will update the
        &ldquo;Last updated&rdquo; date and, for material changes, post a notice on the platform.
        Your continued use of Tornare after changes take effect constitutes your acceptance
        of the new Terms.</p>
      </section>

      <section class="legal-section">
        <h2>11. Governing Law</h2>
        <p>These Terms are governed by and construed in accordance with the laws of the
        jurisdiction in which the platform is operated, without regard to conflict of law
        provisions. Any disputes arising under these Terms shall be subject to the exclusive
        jurisdiction of the courts of that jurisdiction.</p>
      </section>

      <section class="legal-section">
        <h2>12. Contact</h2>
        <p>If you have questions about these Terms, please reach out through the
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
