<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import LegalPageLayout from '../components/ui/LegalPageLayout.vue'
const { t } = useI18n()
</script>

<template>
  <LegalPageLayout :title="t('legal.privacyTitle')" :meta="t('legal.privacyMeta')">

      <p class="legal-intro">{{ t('legal.privacyIntro') }}</p>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS1Title') }}</h2>

        <h3>{{ t('legal.privacyS1H1') }}</h3>
        <p>{{ t('legal.privacyS1P1') }}</p>

        <h3>{{ t('legal.privacyS1H2') }}</h3>
        <p>{{ t('legal.privacyS1P2') }}</p>

        <h3>{{ t('legal.privacyS1H3') }}</h3>
        <p>{{ t('legal.privacyS1P3') }}</p>

        <h3>{{ t('legal.privacyS1H4') }}</h3>
        <p>{{ t('legal.privacyS1P4') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS2Title') }}</h2>
        <ul>
          <li><strong>{{ t('legal.privacyS2L1T') }}</strong>{{ t('legal.privacyS2L1D') }}</li>
          <li><strong>{{ t('legal.privacyS2L2T') }}</strong>{{ t('legal.privacyS2L2D') }}</li>
          <li><strong>{{ t('legal.privacyS2L3T') }}</strong>{{ t('legal.privacyS2L3D') }}</li>
          <li><strong>{{ t('legal.privacyS2L4T') }}</strong>{{ t('legal.privacyS2L4D') }}</li>
        </ul>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS3Title') }}</h2>
        <p>{{ t('legal.privacyS3P1') }}</p>
        <p>{{ t('legal.privacyS3P2') }}</p>
        <ul>
          <li><strong>{{ t('legal.privacyS3L1T') }}</strong>{{ t('legal.privacyS3L1D') }}</li>
          <li><strong>{{ t('legal.privacyS3L2T') }}</strong>{{ t('legal.privacyS3L2D') }}</li>
          <li><strong>{{ t('legal.privacyS3L3T') }}</strong>{{ t('legal.privacyS3L3D') }}</li>
        </ul>
        <p>{{ t('legal.privacyS3P3') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS4Title') }}</h2>
        <p>{{ t('legal.privacyS4P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS5Title') }}</h2>
        <p>{{ t('legal.privacyS5P1a') }}<code>localStorage</code>{{ t('legal.privacyS5P1b') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS6Title') }}</h2>
        <p>{{ t('legal.privacyS6P1') }}</p>
        <ul>
          <li><strong>{{ t('legal.privacyS6L1T') }}</strong>{{ t('legal.privacyS6L1D') }}</li>
          <li><strong>{{ t('legal.privacyS6L2T') }}</strong>{{ t('legal.privacyS6L2D') }}</li>
          <li><strong>{{ t('legal.privacyS6L3T') }}</strong>{{ t('legal.privacyS6L3D') }}</li>
          <li><strong>{{ t('legal.privacyS6L4T') }}</strong>{{ t('legal.privacyS6L4D') }}</li>
          <li><strong>{{ t('legal.privacyS6L5T') }}</strong>{{ t('legal.privacyS6L5D') }}</li>
        </ul>
        <p>{{ t('legal.privacyS6P2') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS7Title') }}</h2>
        <p>{{ t('legal.privacyS7P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS8Title') }}</h2>
        <p>{{ t('legal.privacyS8P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.privacyS9Title') }}</h2>
        <p>{{ t('legal.privacyS9P1a') }}<a href="/support" class="legal-link">{{ t('legal.supportLinkText') }}</a>{{ t('legal.privacyS9P1b') }}</p>
      </section>

  </LegalPageLayout>
</template>

