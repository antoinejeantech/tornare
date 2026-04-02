<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { apiCall } from '../lib/api'
import { useAuthStore } from '../stores/auth'
import { useDebounce } from '../composables/useDebounce'
import { useRequestSequence } from '../composables/useRequestSequence'
import EventListItem from '../components/events/EventListItem.vue'
import SpotlightEventCard from '../components/events/SpotlightEventCard.vue'
import CreateEventModal from '../components/events/CreateEventModal.vue'
import ActionCtaButton from '../components/ui/ActionCtaButton.vue'
import type { Event } from '../types'

const { t } = useI18n()

interface PaginatedEventsResponse {
  items: Event[]
  total: number
}

const authStore = useAuthStore()
const router = useRouter()
const route = useRoute()

const events = ref<Event[]>([])
const featuredEvent = ref<Event | null>(null)
const kpis = ref({
  total_events: 0,
  total_signups: 0,
  upcoming_events_this_week: 0,
  upcoming_tourneys_this_week: 0,
})
const error = ref('')
const loadingEvents = ref(false)
const activeOwnerFilter = ref('all')
const activeTypeFilter = ref('all')
const pastEventsOnly = ref(false)
const showEventsKpis = false
const eventSearchQuery = ref('')
const activeSort = ref('soonest')
const showCreateModal = ref(false)
const SEARCH_DEBOUNCE_MS = 350
const { debounced: debouncedLoad } = useDebounce(SEARCH_DEBOUNCE_MS)
const { next: nextLoadId, isCurrent: isCurrentLoad } = useRequestSequence()
let eventsRequestController: AbortController | null = null
const pageSize = ref(12)
const pageSizeOptions = [12, 24, 48]
const currentPage = ref(1)
const totalEventsAvailable = ref(0)

const normalizedSearchQuery = computed(() => eventSearchQuery.value.trim().toLowerCase())

const sortedEvents = computed(() => events.value)

const totalEventsCount = computed(() => Number(kpis.value.total_events) || 0)

const totalPages = computed(() => {
  const total = Number(totalEventsAvailable.value) || 0
  return Math.max(1, Math.ceil(total / pageSize.value))
})

const visibleEventsCount = computed(() => events.value.length)

const totalPlayersSignedUp = computed(() => Number(kpis.value.total_signups) || 0)
const weeklyTourneyCount = computed(() => Number(kpis.value.upcoming_tourneys_this_week) || 0)

const hasActiveFilters = computed(() => {
  return (
    activeOwnerFilter.value !== 'all' ||
    activeTypeFilter.value !== 'all' ||
    normalizedSearchQuery.value.length > 0 ||
    activeSort.value !== 'soonest' ||
    pastEventsOnly.value
  )
})

function setError(message: string) {
  error.value = message
}

function clearError() {
  error.value = ''
}

function setTypeFilter(filter: string) {
  activeTypeFilter.value = filter
}

function setOwnerFilter(filter: string) {
  activeOwnerFilter.value = filter
}

function clearFilters() {
  activeOwnerFilter.value = 'all'
  activeTypeFilter.value = 'all'
  eventSearchQuery.value = ''
  activeSort.value = 'soonest'
  pastEventsOnly.value = false
}

function openCreateModal() {
  if (!authStore.isAuthenticated) {
    setError(t('events.signInToCreate'))
    return
  }

  clearError()
  showCreateModal.value = true
}

async function loadEvents() {
  if (eventsRequestController) {
    eventsRequestController.abort()
  }
  eventsRequestController = new AbortController()

  const requestId = nextLoadId()
  loadingEvents.value = true
  try {
    clearError()
    const params = new URLSearchParams()
    if (activeOwnerFilter.value !== 'all') {
      params.set('owner', activeOwnerFilter.value)
    }
    if (activeTypeFilter.value !== 'all') {
      params.set('type', activeTypeFilter.value)
    }
    if (normalizedSearchQuery.value) {
      params.set('search', normalizedSearchQuery.value)
    }
    if (activeSort.value !== 'soonest') {
      params.set('sort', activeSort.value)
    }
    if (pastEventsOnly.value) {
      params.set('status', 'ended')
    }
    params.set('page', String(currentPage.value))
    params.set('per_page', String(pageSize.value))

    const query = params.toString()
    const path = query ? `/api/events?${query}` : '/api/events'
    const response = await apiCall<PaginatedEventsResponse>(path, { signal: eventsRequestController.signal })

    if (!isCurrentLoad(requestId)) {
      return
    }

    events.value = response?.items ?? []
    totalEventsAvailable.value = response?.total ?? 0

    if (currentPage.value > totalPages.value) {
      currentPage.value = totalPages.value
      return
    }
  } catch (err) {
    if (err instanceof Error && err.name === 'AbortError') {
      return
    }
    if (!isCurrentLoad(requestId)) {
      return
    }
    setError(err instanceof Error ? err.message : 'Failed to load events')
  } finally {
    if (isCurrentLoad(requestId)) {
      loadingEvents.value = false
    }
  }
}

async function loadFeaturedEvent() {
  try {
    const featured = await apiCall('/api/events/featured')
    featuredEvent.value = (featured as Event) || null
  } catch {
    featuredEvent.value = null
  }
}

function formatKpiValue(value: number): string {
  const numericValue = Number(value)
  if (!Number.isFinite(numericValue)) {
    return '00'
  }

  if (numericValue >= 0 && numericValue < 10) {
    return `0${Math.floor(numericValue)}`
  }

  return String(Math.floor(numericValue))
}

function goToPrevPage() {
  if (currentPage.value > 1) {
    currentPage.value -= 1
  }
}

function goToNextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value += 1
  }
}

function onEventCreated(event: Event) {
  router.push({ name: 'event', params: { id: event.id } })
}

onMounted(() => {
  const q = route.query
  if (q.owner)  activeOwnerFilter.value  = String(q.owner)
  if (q.type)   activeTypeFilter.value   = String(q.type)
  if (q.sort)   activeSort.value         = String(q.sort)
  if (q.ended)  pastEventsOnly.value = q.ended === 'true'
  if (q.search) eventSearchQuery.value   = String(q.search)
  if (q.page)   currentPage.value        = Math.max(1, Number(q.page) || 1)
  if (q.per_page) pageSize.value         = Number(q.per_page) || 12
  loadFeaturedEvent()
  loadEvents()
})

function syncUrl() {
  const query: Record<string, string> = {}
  if (activeOwnerFilter.value !== 'all')   query.owner    = activeOwnerFilter.value
  if (activeTypeFilter.value !== 'all')    query.type     = activeTypeFilter.value
  if (activeSort.value !== 'soonest')      query.sort     = activeSort.value
  if (pastEventsOnly.value)               query.ended    = 'true'
  if (eventSearchQuery.value.trim())       query.search   = eventSearchQuery.value.trim()
  if (currentPage.value > 1)              query.page     = String(currentPage.value)
  if (pageSize.value !== 12)              query.per_page = String(pageSize.value)
  router.replace({ name: 'events', query })
}

watch([activeOwnerFilter, activeTypeFilter, activeSort, pastEventsOnly], () => {
  currentPage.value = 1
  syncUrl()
  loadEvents()
})

watch(pageSize, () => {
  currentPage.value = 1
  syncUrl()
  loadEvents()
})

watch(eventSearchQuery, () => {
  debouncedLoad(() => {
    currentPage.value = 1
    syncUrl()
    loadEvents()
  })
})

watch(currentPage, () => {
  syncUrl()
  loadEvents()
})

onBeforeUnmount(() => {
  if (eventsRequestController) {
    eventsRequestController.abort()
  }
})
</script>

<template>
  <main class="app-shell app-shell--wide events-shell">
    <SpotlightEventCard
      v-if="featuredEvent"
      class="reveal-block reveal-1"
      :event="featuredEvent"
      :badge-label="t('home.spotlightBadge')"
    />

    <section v-if="showEventsKpis" class="events-stats-grid reveal-block reveal-2" aria-label="Event highlights">
      <article class="events-stat-card">
        <span class="material-symbols-rounded events-stat-icon" aria-hidden="true">space_dashboard</span>
        <div class="events-stat-copy">
          <span class="events-stat-label">Live board</span>
          <strong class="events-stat-value">{{ formatKpiValue(totalEventsCount) }}</strong>
          <span class="muted">Active listings</span>
        </div>
      </article>
      <article class="events-stat-card">
        <span class="material-symbols-rounded events-stat-icon" aria-hidden="true">groups</span>
        <div class="events-stat-copy">
          <span class="events-stat-label">Signups</span>
          <strong class="events-stat-value">{{ formatKpiValue(totalPlayersSignedUp) }}</strong>
          <span class="muted">Players currently registered</span>
        </div>
      </article>
      <article class="events-stat-card">
        <span class="material-symbols-rounded events-stat-icon" aria-hidden="true">event_upcoming</span>
        <div class="events-stat-copy">
          <span class="events-stat-label">This week</span>
          <strong class="events-stat-value">{{ formatKpiValue(weeklyTourneyCount) }}</strong>
          <span class="muted">Upcoming tourneys</span>
        </div>
      </article>
    </section>

    <section class="events-header reveal-block reveal-2">
      <div class="events-toolbar-title-wrap">
        <h2>{{ pastEventsOnly ? t('events.titlePast') : t('events.title') }}</h2>
        <p class="muted">{{ t('events.subtitle') }}</p>
      </div>
      <ActionCtaButton
        class="events-create-btn"
        :disabled="!authStore.isAuthenticated"
        :title="authStore.isAuthenticated ? t('events.createBtnTitle') : t('events.createBtnTitleGuest')"
        @click="openCreateModal"
      >
        <span class="material-symbols-rounded" aria-hidden="true">add</span>
        <span>{{ t('events.createBtn') }}</span>
      </ActionCtaButton>
    </section>

    <section class="card events-toolbar reveal-block reveal-2">
      <div class="events-filter-row">
        <label class="events-search">
          <span class="sr-only">{{ t('events.searchLabel') }}</span>
          <span class="material-symbols-rounded" aria-hidden="true">search</span>
          <input v-model="eventSearchQuery" type="search" :placeholder="t('events.searchPlaceholder')" />
        </label>

        <div v-if="authStore.isAuthenticated" class="events-subnav" :aria-label="t('events.ownershipFilter')">
          <button
            class="events-subnav-btn"
            :class="{ active: activeOwnerFilter === 'all' }"
            :aria-pressed="activeOwnerFilter === 'all'"
            @click="setOwnerFilter('all')"
          >
            {{ t('events.filterAll') }}
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeOwnerFilter === 'mine' }"
            :aria-pressed="activeOwnerFilter === 'mine'"
            @click="setOwnerFilter('mine')"
          >
            {{ t('events.filterMine') }}
          </button>
        </div>

        <div class="events-subnav" :aria-label="t('events.typeFilter')">
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'all' }"
            :aria-pressed="activeTypeFilter === 'all'"
            @click="setTypeFilter('all')"
          >
            {{ t('events.typeAll') }}
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'PUG' }"
            :aria-pressed="activeTypeFilter === 'PUG'"
            @click="setTypeFilter('PUG')"
          >
            {{ t('events.typePug') }}
          </button>
          <button
            class="events-subnav-btn"
            :class="{ active: activeTypeFilter === 'TOURNEY' }"
            :aria-pressed="activeTypeFilter === 'TOURNEY'"
            @click="setTypeFilter('TOURNEY')"
          >
            {{ t('events.typeTourney') }}
          </button>
        </div>

        <label class="events-sort">
          <span class="events-sort-copy">
            <span class="events-sort-label">{{ t('events.sortLabel') }}</span>
          </span>
          <span class="events-sort-field">
            <select v-model="activeSort" :aria-label="t('events.sortLabel')">
              <option value="soonest">{{ t('events.sortSoonest') }}</option>
              <option value="newest">{{ t('events.sortNewest') }}</option>
              <option value="players">{{ t('events.sortPlayers') }}</option>
              <option value="name">{{ t('events.sortName') }}</option>
            </select>
            <span class="material-symbols-rounded events-sort-caret" aria-hidden="true">expand_more</span>
          </span>
        </label>

        <button
          type="button"
          class="events-ended-toggle"
          :class="{ active: pastEventsOnly }"
          role="switch"
          :aria-checked="pastEventsOnly"
          @click="pastEventsOnly = !pastEventsOnly"
        >
          <span class="events-ended-toggle-copy">
            <span class="events-ended-toggle-label">{{ t('events.pastToggleLabel') }}</span>
            <span class="events-ended-toggle-state">{{ pastEventsOnly ? t('events.pastToggleOn') : t('events.pastToggleOff') }}</span>
          </span>
          <span class="events-ended-toggle-switch" aria-hidden="true">
            <span class="events-ended-toggle-thumb" />
          </span>
        </button>

        <button
          type="button"
          class="events-clear-link"
          :disabled="!hasActiveFilters"
          @click="clearFilters"
        >
          <span class="material-symbols-rounded" aria-hidden="true">refresh</span>
          <span>{{ t('events.clearFilters') }}</span>
        </button>
      </div>
    </section>

    <p v-if="error" class="status status-error">{{ error }}</p>

    <section class="card events-list-shell reveal-block reveal-3">
      <p v-if="loadingEvents">{{ t('events.loading') }}</p>
      <div v-else-if="sortedEvents.length === 0" class="events-empty-state">
        <h2>{{ t('events.emptyTitle') }}</h2>
        <p class="muted">{{ t('events.emptySubtitle') }}</p>
        <div class="events-empty-actions">
          <button type="button" class="btn-secondary" :disabled="!hasActiveFilters" @click="clearFilters">{{ t('events.clearFilters') }}</button>
          <ActionCtaButton :disabled="!authStore.isAuthenticated" @click="openCreateModal">{{ t('events.createBtn') }}</ActionCtaButton>
        </div>
      </div>
      <ul v-else class="home-events-list">
        <EventListItem
          v-for="(event, index) in sortedEvents"
          :key="event.id"
          :event="event"
          :to="{ name: 'event', params: { id: event.id } }"
          class="events-list-row"
          :style="{ animationDelay: `${index * 45}ms` }"
        />
      </ul>

      <div class="events-pagination" role="navigation" :aria-label="t('events.pagination')">
        <p class="events-pagination-meta muted">
          {{ t('events.paginationPage', { current: currentPage, total: totalPages }) }}
          <span class="events-pagination-divider" aria-hidden="true">•</span>
          {{ t('events.paginationShown', { count: visibleEventsCount }) }}
          <span class="events-pagination-divider" aria-hidden="true">•</span>
          {{ t('events.paginationTotal', { total: totalEventsAvailable }) }}
        </p>

        <label class="events-pagination-size">
          <span class="events-pagination-size-label">{{ t('events.paginationPerPage') }}</span>
          <span class="events-pagination-size-field">
            <select v-model.number="pageSize" :aria-label="t('events.paginationPerPage')">
              <option v-for="option in pageSizeOptions" :key="`page-size-${option}`" :value="option">
                {{ option }}
              </option>
            </select>
            <span class="material-symbols-rounded events-pagination-size-caret" aria-hidden="true">expand_more</span>
          </span>
        </label>

        <div class="events-pagination-actions">
          <button
            type="button"
            class="events-pagination-nav events-pagination-nav--prev"
            :disabled="currentPage <= 1"
            @click="goToPrevPage"
          >
            <span class="material-symbols-rounded" aria-hidden="true">arrow_back</span>
            <span>{{ t('events.paginationPrev') }}</span>
          </button>
          <button
            type="button"
            class="events-pagination-nav events-pagination-nav--next"
            :disabled="currentPage >= totalPages"
            @click="goToNextPage"
          >
            <span>{{ t('events.paginationNext') }}</span>
            <span class="material-symbols-rounded" aria-hidden="true">arrow_forward</span>
          </button>
        </div>
      </div>
    </section>

    <CreateEventModal v-model:open="showCreateModal" @created="onEventCreated" />
  </main>
</template>

<style scoped>
.events-shell {
  gap: 0.88rem;
}

.events-shell :is(h2, h3) {
  letter-spacing: -0.01em;
}

.events-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.events-toolbar {
  padding: 0.72rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  border-radius: var(--radius-md);
  box-shadow: none;
  margin-bottom: 0.85rem;
}

.events-list-shell {
  border: 0;
  background: transparent;
  box-shadow: none;
  padding: 0;
}

.events-toolbar-title-wrap {
  display: grid;
  gap: 0.2rem;
}

.events-toolbar-title-wrap h2,
.events-toolbar-title-wrap p {
  margin: 0;
}

.events-create-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
}

.events-create-btn .material-symbols-rounded {
  font-size: 1rem;
}

.events-filter-row {
  --events-toolbar-control-height: 2.35rem;
  display: flex;
  align-items: stretch;
  gap: 0.55rem;
  flex-wrap: wrap;
}

.events-stats-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
  margin-bottom: 0.95rem;
}

.events-stat-card {
  border: 1px solid var(--surface-card-border);
  border-radius: var(--radius-md);
  padding: 1.25rem 1.2rem;
  background: var(--surface-card-bg);
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 0.9rem;
  box-shadow: none;
}

.events-stat-copy {
  display: grid;
  gap: 0.22rem;
}

.events-stat-icon {
  font-size: 2rem;
  line-height: 1;
  color: color-mix(in srgb, var(--brand-1) 90%, #ffd869 10%);
}

.events-shell :deep(.spotlight-event-card) {
  margin-bottom: 0.5rem;
}

.events-stat-label {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--ink-2);
}

.events-stat-value {
  font-size: 1.48rem;
  line-height: 1;
}

.events-search {
  min-width: min(100%, 300px);
  flex: 1;
  display: inline-flex;
  align-items: center;
  min-height: var(--events-toolbar-control-height);
  gap: 0.35rem;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  background: var(--surface-card-bg);
  box-sizing: border-box;
  padding: 0 0.55rem;
}

.events-search input {
  flex: 1;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--ink-1);
  min-width: 0;
}

.events-search input:focus {
  outline: none;
}

.events-subnav {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
  width: fit-content;
  min-height: var(--events-toolbar-control-height);
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: var(--radius-pill);
  box-sizing: border-box;
  padding: 0.18rem;
  background: var(--surface-card-bg);
  position: relative;
  z-index: 1;
}

.events-subnav-btn {
  display: inline-flex;
  align-items: center;
  min-height: calc(var(--events-toolbar-control-height) - 0.36rem);
  border: 0;
  background: transparent;
  color: var(--ink-2);
  font-weight: 620;
  padding: 0.34rem 0.72rem;
  border-radius: var(--radius-pill);
  line-height: 1;
  cursor: pointer;
  user-select: none;
  transition: background 0.16s ease, color 0.16s ease;
}

.events-subnav-btn:hover {
  color: var(--ink-1);
  background: color-mix(in srgb, var(--brand-2) 10%, var(--card) 90%);
}

.events-subnav-btn.active {
  color: var(--primary-100);
  font-weight: 680;
  background: var(--primary-700);
}

.events-sort {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
  min-height: var(--events-toolbar-control-height);
  padding: 0 0.38rem 0 0.72rem;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: var(--radius-pill);
  background: var(--surface-card-bg);
  box-sizing: border-box;
}

.events-sort-copy {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.events-sort-label {
  font-size: 0.72rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--ink-muted);
  font-weight: 700;
}

.events-sort-field {
  position: relative;
  display: inline-flex;
  align-items: center;
  flex: 0 0 auto;
}

.events-sort select {
  appearance: none;
  -webkit-appearance: none;
  height: calc(var(--events-toolbar-control-height) - 2px);
  border: 0;
  background: transparent;
  color: var(--ink-1);
  font-family: inherit;
  font-weight: 700;
  line-height: 1;
  padding: 0 1.4rem 0 0;
  margin: 0;
  min-width: 1.25rem;
  cursor: pointer;
}

.events-sort-caret {
  position: absolute;
  right: 0;
  pointer-events: none;
  font-size: 1rem;
  color: var(--ink-muted);
}

.events-clear-link {
  border: 0;
  background: transparent;
  color: color-mix(in srgb, white 92%, var(--ink-1) 8%);
  font-weight: 700;
  letter-spacing: 0.02em;
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.2rem 0.1rem;
  cursor: pointer;
  transition: color 0.16s ease, transform 0.16s ease;
}

.events-ended-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
  min-height: var(--events-toolbar-control-height);
  padding: 0 0.38rem 0 0.72rem;
  border: 1px solid color-mix(in srgb, var(--line) 86%, var(--brand-1) 14%);
  border-radius: var(--radius-pill);
  background: var(--surface-card-bg);
  box-sizing: border-box;
  font-family: inherit;
  font-size: 0.82rem;
  font-weight: 650;
  line-height: 1;
  color: var(--ink-2);
  cursor: pointer;
  transition: background 0.14s ease, border-color 0.14s ease, color 0.14s ease, box-shadow 0.14s ease;
  user-select: none;
}

.events-ended-toggle-copy {
  display: inline-flex;
  align-items: center;
  gap: 0.42rem;
}

.events-ended-toggle-label {
  color: var(--ink-1);
}

.events-ended-toggle-state {
  display: inline-block;
  font-size: 0.72rem;
  letter-spacing: 0.05em;
  min-width: 3ch;
  text-align: center;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.events-ended-toggle-switch {
  position: relative;
  flex: 0 0 auto;
  width: 2.2rem;
  height: 1.3rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--line) 72%, var(--surface-card-bg) 28%);
  transition: background 0.16s ease;
}

.events-ended-toggle-thumb {
  position: absolute;
  top: 0.15rem;
  left: 0.15rem;
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  background: color-mix(in srgb, white 88%, var(--card) 12%);
  box-shadow: 0 1px 3px rgb(0 0 0 / 0.24);
  transition: transform 0.16s ease, background 0.16s ease;
}

.events-ended-toggle:hover {
  border-color: color-mix(in srgb, var(--brand-2) 48%, var(--line) 52%);
  color: var(--ink-1);
}

.events-ended-toggle:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--brand-2) 18%, transparent 82%);
}

.events-ended-toggle.active {
  border-color: color-mix(in srgb, var(--brand-1) 52%, var(--line) 48%);
  background: var(--surface-card-bg);
  color: var(--ink-2);
}

.events-ended-toggle.active .events-ended-toggle-state {
  color: color-mix(in srgb, var(--brand-1) 72%, white 28%);
}

.events-ended-toggle.active .events-ended-toggle-switch {
  background: linear-gradient(135deg, color-mix(in srgb, var(--brand-1) 88%, white 12%), var(--brand-1));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--brand-1) 82%, black 18%);
}

.events-ended-toggle.active .events-ended-toggle-thumb {
  transform: translateX(0.9rem);
  background: white;
}

.events-clear-link .material-symbols-rounded {
  font-size: 0.92rem;
}

.events-clear-link:hover {
  color: #fff;
  transform: translateY(-1px);
}

.events-clear-link:disabled {
  color: var(--ink-muted);
  cursor: not-allowed;
  transform: none;
  opacity: 0.7;
}

.events-empty-state {
  display: grid;
  gap: 0.45rem;
}

.events-empty-state h2,
.events-empty-state p {
  margin: 0;
}

.events-empty-actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.events-pagination {
  margin-top: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
  flex-wrap: wrap;
}

.events-pagination-meta {
  margin: 0;
  min-width: 7.2rem;
}

.events-pagination-divider {
  margin: 0 0.45rem;
}

.events-pagination-size {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--ink-2);
  font-weight: 600;
}

.events-pagination-size-label {
  font-size: 0.8rem;
}

.events-pagination-size-field {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-height: 2.35rem;
  padding: 0 0.8rem;
  border: 1px solid color-mix(in srgb, var(--line) 76%, var(--brand-1) 24%);
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--surface-card-bg) 82%, var(--brand-1) 18%);
}

.events-pagination-size select {
  appearance: none;
  -webkit-appearance: none;
  min-height: 2.1rem;
  border: 0;
  background: transparent;
  color: var(--ink-1);
  font-family: inherit;
  font-weight: 700;
  padding: 0 1.1rem 0 0;
  cursor: pointer;
}

.events-pagination-size-caret {
  position: absolute;
  right: 0.75rem;
  pointer-events: none;
  font-size: 1rem;
  color: var(--ink-muted);
}

.events-pagination-actions {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
}

.events-pagination-nav {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.38rem;
  min-height: 2.35rem;
  padding: 0 0.9rem;
  border: 1px solid color-mix(in srgb, var(--line) 76%, var(--brand-1) 24%);
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--surface-card-bg) 82%, var(--brand-1) 18%);
  color: var(--ink-1);
  font-family: inherit;
  font-size: 0.84rem;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
  transition: transform 0.14s ease, border-color 0.14s ease, background 0.14s ease, color 0.14s ease;
}

.events-pagination-nav .material-symbols-rounded {
  font-size: 1rem;
}

.events-pagination-nav:hover:not(:disabled) {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--brand-2) 58%, var(--line) 42%);
  background: color-mix(in srgb, var(--surface-card-bg) 58%, var(--brand-2) 42%);
}

.events-pagination-nav:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.home-events-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.events-list-row {
  opacity: 0;
  transform: translateY(8px);
  animation: list-rise 300ms ease-out forwards;
}

.reveal-block {
  opacity: 0;
  transform: translateY(12px);
  animation: reveal-rise 380ms ease-out forwards;
}

.reveal-1 { animation-delay: 60ms; }
.reveal-2 { animation-delay: 120ms; }
.reveal-3 { animation-delay: 180ms; }

@keyframes reveal-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes list-rise {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 840px) {
  .events-header {
    flex-wrap: wrap;
  }

  .events-stats-grid {
    grid-template-columns: 1fr;
  }

  .events-empty-actions {
    flex-wrap: wrap;
  }
}

</style>
