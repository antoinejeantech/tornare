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

<style scoped>
.section-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.section-title {
  margin: 0 0 0.72rem;
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
  font-size: 1.55rem;
  font-weight: 800;
}

.section-title-divider {
  display: none;
}

.section-title-icon {
  font-size: 1.16rem;
  line-height: 1;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
}
</style>
