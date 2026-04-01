<script setup lang="ts">
const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>()

// Official Overwatch 2 maps (5v5 competitive pool as of 2026)
const OW_MAPS = [
  // Control
  'Antarctic Peninsula',
  'Busan',
  'Ilios',
  'Lijiang Tower',
  'Nepal',
  'Oasis',
  'Samoa',
  // Escort
  'Circuit Royal',
  'Dorado',
  'Havana',
  'Junkertown',
  'Rialto',
  'Route 66',
  'Shambali Monastery',
  'Watchpoint: Gibraltar',
  // Hybrid
  'Blizzard World',
  'Eichenwalde',
  'Hollywood',
  "King's Row",
  'Midtown',
  'Numbani',
  'Paraíso',
  // Push
  'Colosseo',
  'Esperança',
  'New Queen Street',
  'Runasapi',
  // Clash
  'Hanaoka',
  'Throne of Anubis',
  // Flashpoint
  'New Junk City',
  'Suravasa',
  // Deathmatch / others sometimes in custom PUGs
//   'Malevento',
//   'Kanezaka',
//   'Château Guillard',
//   'Black Forest',
//   'Necropolis',
//   'Petra',
].sort()

const listId = `map-picker-${Math.random().toString(36).slice(2, 8)}`

function pickRandom() {
  const map = OW_MAPS[Math.floor(Math.random() * OW_MAPS.length)]
  emit('update:modelValue', map)
}
</script>

<template>
  <div class="map-picker">
    <input
      class="map-picker-input"
      :list="listId"
      :value="modelValue"
      placeholder="King's Row"
      autocomplete="off"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <datalist :id="listId">
      <option v-for="map in OW_MAPS" :key="map" :value="map" />
    </datalist>
    <button
      type="button"
      class="map-picker-random"
      title="Pick a random map"
      @click="pickRandom"
    >
      <span class="material-symbols-rounded map-picker-random-icon" aria-hidden="true">shuffle</span>
    </button>
  </div>
</template>

<style scoped>
.map-picker {
  display: flex;
  align-items: center;
  gap: 0;
}

.map-picker-input {
  flex: 1;
  min-width: 0;
  border-radius: var(--radius-sm) 0 0 var(--radius-sm);
}

.map-picker-random {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 100%;
  min-height: 2.25rem;
  border: 1px solid var(--line);
  border-left: none;
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  background: color-mix(in srgb, var(--brand-1) 10%, var(--card) 90%);
  color: var(--brand-1);
  cursor: pointer;
  transition: background 120ms, color 120ms, border-color 120ms;
  flex-shrink: 0;
}

.map-picker-random:hover {
  background: color-mix(in srgb, var(--brand-1) 22%, var(--card) 78%);
  border-color: color-mix(in srgb, var(--brand-1) 50%, var(--line) 50%);
}

.map-picker-random-icon {
  font-size: 1.1rem;
  font-variation-settings: 'FILL' 1;
}
</style>
