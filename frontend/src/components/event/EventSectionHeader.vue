<script setup>
defineProps({
  icon: { type: String, required: true },
  title: { type: String, required: true },
})
</script>

<template>
  <div class="section-header-row">
    <h3 class="section-title">
      <span class="material-symbols-rounded section-title-icon" aria-hidden="true">{{ icon }}</span>
      <span>{{ title }}</span>
    </h3>
    <slot />
  </div>
  <div class="section-title-divider" aria-hidden="true"></div>
</template>
