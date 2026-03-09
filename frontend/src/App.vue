<script setup>
import { RouterView } from 'vue-router'
import MainNav from './components/MainNav.vue'
import AlertHost from './components/AlertHost.vue'
</script>

<template>
  <div class="app-root">
    <MainNav />
    <RouterView />
    <AlertHost />
    <footer class="app-footer">
      <div class="app-footer-inner">© 2026 Tornare. All rights reserved.</div>
    </footer>
  </div>
</template>
