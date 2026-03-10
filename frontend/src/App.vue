<script setup>
import { RouterView } from 'vue-router'
import MainNav from './components/MainNav.vue'
import AlertHost from './components/AlertHost.vue'
import ConfirmHost from './components/ConfirmHost.vue'
</script>

<template>
  <div class="app-root">
    <MainNav />
    <RouterView />
    <AlertHost />
    <ConfirmHost />
    <footer class="app-footer">
      <div class="app-footer-inner">
        <span>© 2026 Tornare. All rights reserved.</span>
        <nav class="app-footer-links" aria-label="Footer links">
          <a href="/privacy">Privacy Policy</a>
          <a href="/terms">Terms of Service</a>
          <a href="/faq">FAQ</a>
          <a href="/support">Support</a>
          <a href="https://github.com" target="_blank" rel="noreferrer">GitHub</a>
        </nav>
      </div>
    </footer>
  </div>
</template>
