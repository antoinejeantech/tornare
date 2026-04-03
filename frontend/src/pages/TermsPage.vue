<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import LegalPageLayout from '../components/ui/LegalPageLayout.vue'
const { t } = useI18n()
</script>

<template>
  <LegalPageLayout :title="t('legal.termsTitle')" :meta="t('legal.termsMeta')">

      <p class="legal-intro">{{ t('legal.termsIntro') }}</p>

      <section class="legal-section">
        <h2>{{ t('legal.termsS1Title') }}</h2>
        <p>{{ t('legal.termsS1P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS2Title') }}</h2>
        <p>{{ t('legal.termsS2P1') }}</p>
        <p>{{ t('legal.termsS2P2') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS3Title') }}</h2>
        <p>{{ t('legal.termsS3P1') }}</p>
        <ul>
          <li>{{ t('legal.termsS3L1') }}</li>
          <li>{{ t('legal.termsS3L2') }}</li>
          <li>{{ t('legal.termsS3L3') }}</li>
          <li>{{ t('legal.termsS3L4') }}</li>
          <li>{{ t('legal.termsS3L5') }}</li>
          <li>{{ t('legal.termsS3L6') }}</li>
        </ul>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS4Title') }}</h2>
        <p>{{ t('legal.termsS4P1') }}</p>
        <p>{{ t('legal.termsS4P2') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS5Title') }}</h2>
        <p>{{ t('legal.termsS5P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS6Title') }}</h2>
        <p>{{ t('legal.termsS6P1') }}</p>
        <p>{{ t('legal.termsS6P2') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS7Title') }}</h2>
        <p>{{ t('legal.termsS7P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS8Title') }}</h2>
        <p>{{ t('legal.termsS8P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS9Title') }}</h2>
        <p>{{ t('legal.termsS9P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS10Title') }}</h2>
        <p>{{ t('legal.termsS10P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS11Title') }}</h2>
        <p>{{ t('legal.termsS11P1') }}</p>
      </section>

      <section class="legal-section">
        <h2>{{ t('legal.termsS12Title') }}</h2>
        <p>{{ t('legal.termsS12P1a') }}<a href="/support" class="legal-link">{{ t('legal.supportLinkText') }}</a>{{ t('legal.termsS12P1b') }}</p>
      </section>

  </LegalPageLayout>
</template>

