<script setup lang="ts">
import { RouterView } from 'vue-router'
import MainNav from './components/MainNav.vue'
import AppFooter from './components/AppFooter.vue'
import AlertHost from './components/AlertHost.vue'
import ConfirmHost from './components/ConfirmHost.vue'
import PwaUpdatePrompt from './components/PwaUpdatePrompt.vue'
</script>

<template>
  <div class="app-root">
    <MainNav />
    <RouterView />
    <AlertHost />
    <ConfirmHost />
    <PwaUpdatePrompt />
    <AppFooter />
  </div>
</template>
