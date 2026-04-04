<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  ApiHttpError,
  addGuildMember,
  deleteDiscordGuild,
  getDiscordBotInviteUrl,
  getDiscordGuilds,
  listGuildMembers,
  removeGuildMember,
  searchUsers,
  toggleDiscordAnnouncements,
} from '../lib/api'
import type { DiscordGuild, GuildMember, UserSearchResult } from '../lib/api'
import DiscordIcon from '../components/ui/DiscordIcon.vue'

const { t } = useI18n()
const alert = useAlert()

import { useAlert } from '../composables/alerts'

const guilds = ref<DiscordGuild[]>([])
const inviteUrl = ref('')
const loading = ref(true)

// Per-guild state, keyed by guild.guild_id
const membersMap = ref<Record<string, GuildMember[]>>({})
const memberSearch = ref<Record<string, string>>({})
const memberSearchResults = ref<Record<string, UserSearchResult[]>>({})
const memberSearchLoading = ref<Record<string, boolean>>({})
const addingMember = ref<Record<string, boolean>>({})
const removingMember = ref<Record<string, string | null>>({})
const toggling = ref<Record<string, boolean>>({})
const disconnecting = ref<Record<string, boolean>>({})

async function load() {
  loading.value = true
  try {
    const [gs, inv] = await Promise.allSettled([
      getDiscordGuilds(),
      getDiscordBotInviteUrl(),
    ])
    if (gs.status === 'fulfilled') {
      guilds.value = gs.value
      // Load members for each guild
      await Promise.all(guilds.value.map(g => loadMembers(g.guild_id)))
    }
    if (inv.status === 'fulfilled') inviteUrl.value = inv.value.url
  } finally {
    loading.value = false
  }
}

async function loadMembers(guildId: string) {
  try {
    membersMap.value[guildId] = await listGuildMembers(guildId)
  } catch {
    membersMap.value[guildId] = []
  }
}

async function toggleAnnouncements(guild: DiscordGuild) {
  toggling.value[guild.guild_id] = true
  try {
    const updated = await toggleDiscordAnnouncements(guild.guild_id, !guild.announcements_enabled)
    const idx = guilds.value.findIndex(g => g.guild_id === guild.guild_id)
    if (idx !== -1) guilds.value[idx] = updated
  } catch (e) {
    alert.error(e instanceof ApiHttpError ? e.message : t('discord.announcements.error'))
  } finally {
    toggling.value[guild.guild_id] = false
  }
}

async function disconnect(guild: DiscordGuild) {
  disconnecting.value[guild.guild_id] = true
  try {
    await deleteDiscordGuild(guild.guild_id)
    guilds.value = guilds.value.filter(g => g.guild_id !== guild.guild_id)
    alert.success(t('discord.disconnected'))
  } catch (e) {
    alert.error(e instanceof ApiHttpError ? e.message : t('discord.disconnectError'))
  } finally {
    disconnecting.value[guild.guild_id] = false
  }
}

const searchTimeouts: Record<string, ReturnType<typeof setTimeout> | null> = {}
async function onSearchInput(guildId: string) {
  const q = (memberSearch.value[guildId] ?? '').trim()
  memberSearchResults.value[guildId] = []
  if (!q) return
  if (searchTimeouts[guildId]) clearTimeout(searchTimeouts[guildId]!)
  searchTimeouts[guildId] = setTimeout(async () => {
    memberSearchLoading.value[guildId] = true
    try {
      memberSearchResults.value[guildId] = await searchUsers(q)
    } finally {
      memberSearchLoading.value[guildId] = false
    }
  }, 250)
}

async function addMember(guildId: string, user: UserSearchResult) {
  addingMember.value[guildId] = true
  try {
    membersMap.value[guildId] = await addGuildMember(guildId, user.id)
    memberSearch.value[guildId] = ''
    memberSearchResults.value[guildId] = []
  } catch (e) {
    alert.error(e instanceof ApiHttpError ? e.message : t('discord.members.addError'))
  } finally {
    addingMember.value[guildId] = false
  }
}

async function removeMember(guildId: string, userId: string, guild: DiscordGuild) {
  if (userId === guild.owner_user_id) {
    alert.error(t('discord.members.removeOwner'))
    return
  }
  removingMember.value[guildId] = userId
  try {
    membersMap.value[guildId] = await removeGuildMember(guildId, userId)
  } catch (e) {
    alert.error(e instanceof ApiHttpError ? e.message : t('discord.members.removeError'))
  } finally {
    removingMember.value[guildId] = null
  }
}

onMounted(load)
</script>

<template>
  <div class="discord-page">
    <div class="page-header">
      <DiscordIcon class="header-icon" />
      <div>
        <h1>{{ t('discord.title') }}</h1>
        <p class="subtitle">{{ t('discord.subtitle') }}</p>
      </div>
    </div>

    <div v-if="loading" class="loading">{{ t('common.loading') }}</div>

    <template v-else>
      <!-- Setup instructions -->
      <section class="card">
        <h2>{{ t('discord.step1.title') }}</h2>
        <p>{{ t('discord.step1.description') }}</p>
        <a
          v-if="inviteUrl"
          :href="inviteUrl"
          target="_blank"
          rel="noopener noreferrer"
          class="btn btn-discord"
        >
          <DiscordIcon class="btn-icon" />
          {{ t('discord.step1.addBot') }}
        </a>
      </section>

      <section class="card">
        <h2>{{ t('discord.step2.title') }}</h2>
        <p>{{ t('discord.step2.description') }}</p>
        <div class="command-block">/setup</div>
        <p class="hint">{{ t('discord.step2.hint') }}</p>
      </section>

      <!-- One card per guild -->
      <section
        v-for="guild in guilds"
        :key="guild.guild_id"
        class="card guild-card"
      >
        <!-- Header row -->
        <div class="guild-header">
          <div class="status-row">
            <span class="dot connected" />
            <span class="guild-name">
              {{ guild.guild_name || guild.guild_id }}
            </span>
          </div>
          <span class="channel-badge"># {{ guild.channel_id }}</span>
        </div>

        <!-- Post error warning -->
        <div v-if="guild.last_post_error" class="post-error-banner">
          ⚠️ {{ t('discord.postError.label') }}: {{ guild.last_post_error }}
        </div>

        <!-- Announcements toggle -->
        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">{{ t('discord.announcements.label') }}</span>
            <span class="hint">{{ t('discord.announcements.hint') }}</span>
          </div>
          <button
            class="toggle-btn"
            :class="{ active: guild.announcements_enabled }"
            :disabled="toggling[guild.guild_id]"
            :aria-label="t('discord.announcements.label')"
            @click="toggleAnnouncements(guild)"
          >
            <span class="toggle-track">
              <span class="toggle-thumb" />
            </span>
            <span class="toggle-text">
              {{ guild.announcements_enabled
                ? t('common.enabled')
                : t('common.disabled') }}
            </span>
          </button>
        </div>

        <!-- Members section -->
        <div class="members-section">
          <div class="members-header">
            <span class="section-label">{{ t('discord.members.title') }}</span>
            <span class="hint">{{ t('discord.members.hint') }}</span>
          </div>

          <ul class="member-list">
            <li
              v-for="member in membersMap[guild.guild_id] ?? []"
              :key="member.user_id"
              class="member-item"
            >
              <span class="member-name">
                {{ member.display_name }}
                <span v-if="member.username" class="member-username">@{{ member.username }}</span>
              </span>
              <span v-if="member.user_id === guild.owner_user_id" class="owner-badge">
                {{ t('discord.members.owner') }}
              </span>
              <button
                v-else
                class="remove-btn"
                :disabled="removingMember[guild.guild_id] === member.user_id"
                @click="removeMember(guild.guild_id, member.user_id, guild)"
              >
                <span class="material-symbols-rounded">close</span>
              </button>
            </li>
          </ul>

          <!-- Add member -->
          <div class="add-member">
            <div class="search-wrapper">
              <input
                v-model="memberSearch[guild.guild_id]"
                type="text"
                class="search-input"
                :placeholder="t('discord.members.addPlaceholder')"
                @input="onSearchInput(guild.guild_id)"
              />
              <ul
                v-if="(memberSearch[guild.guild_id] ?? '').trim()"
                class="search-dropdown"
              >
                <li v-if="memberSearchLoading[guild.guild_id]" class="no-results">
                  {{ t('common.loading') }}
                </li>
                <template v-else>
                  <li
                    v-for="user in memberSearchResults[guild.guild_id]"
                    :key="user.id"
                    class="search-result"
                    @click="addMember(guild.guild_id, user)"
                  >
                    <span class="member-name">{{ user.display_name }}</span>
                    <span v-if="user.username" class="member-username">@{{ user.username }}</span>
                  </li>
                  <li
                    v-if="!(memberSearchResults[guild.guild_id] ?? []).length"
                    class="no-results"
                  >
                    {{ t('discord.members.noResults') }}
                  </li>
                </template>
              </ul>
            </div>
          </div>
        </div>

        <!-- Disconnect -->
        <div class="form-actions">
          <button
            class="btn btn-danger"
            :disabled="disconnecting[guild.guild_id]"
            @click="disconnect(guild)"
          >
            {{ disconnecting[guild.guild_id] ? t('common.loading') : t('discord.form.disconnect') }}
          </button>
        </div>
      </section>

      <div v-if="!guilds.length" class="card status-card">
        <div class="status-row">
          <span class="dot disconnected" />
          <span class="status-label">{{ t('discord.notConnected') }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.discord-page {
  max-width: 620px;
  margin: 0 auto;
  padding: 2rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.page-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-wrap: nowrap;
}

.header-icon {
  width: 2.5rem;
  height: 2.5rem;
  color: #5865f2;
  flex-shrink: 0;
}

.page-header h1 {
  font-size: 1.75rem;
  font-weight: 700;
  margin: 0 0 0.2rem;
  white-space: nowrap;
}

.page-header .subtitle {
  color: var(--color-text-muted, #888);
  margin: 0;
  font-size: 0.9rem;
}

.card {
  background: var(--color-surface, #1a1a2e);
  border: 1px solid var(--color-border, #2a2a3e);
  border-radius: 12px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.card h2 {
  font-size: 1.05rem;
  font-weight: 600;
  margin: 0;
}

.card p {
  color: var(--color-text-muted, #888);
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.5;
}

.hint {
  color: var(--color-text-muted, #888);
  font-size: 0.8rem;
}

.command-block {
  display: inline-flex;
  align-items: center;
  background: var(--color-input-bg, #0d0d1a);
  border: 1px solid var(--color-border, #2a2a3e);
  border-radius: 6px;
  padding: 0.4rem 0.9rem;
  font-family: monospace;
  font-size: 1rem;
  color: var(--color-accent, #5865f2);
  letter-spacing: 0.03em;
  align-self: flex-start;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1.25rem;
  border-radius: 8px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  text-decoration: none;
  transition: opacity 0.15s;
  white-space: nowrap;
  align-self: flex-start;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-discord {
  background: #5865f2;
  color: #fff;
  align-self: center;
}

.btn-discord:hover:not(:disabled) {
  opacity: 0.85;
}

.btn-icon {
  width: 1.1rem;
  height: 1.1rem;
  flex-shrink: 0;
}

.btn-danger {
  background: transparent;
  border: 1px solid #ed4245;
  color: #ed4245;
}

.btn-danger:hover:not(:disabled) {
  background: #ed424520;
}

/* Guild card */
.guild-card {
  gap: 1rem;
}

.guild-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.guild-name {
  font-weight: 600;
  font-size: 1rem;
}

.channel-badge {
  font-family: monospace;
  font-size: 0.8rem;
  color: var(--color-text-muted, #888);
  background: var(--color-input-bg, #0d0d1a);
  border: 1px solid var(--color-border, #2a2a3e);
  border-radius: 4px;
  padding: 0.15rem 0.5rem;
}

/* Status */
.status-card { gap: 1rem; }

.status-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot.connected { background: #57f287; }
.dot.disconnected { background: #888; }

.status-label {
  font-size: 0.9rem;
  font-weight: 500;
}

/* Toggle */
.post-error-banner {
  padding: 0.6rem 0.85rem;
  border-radius: 6px;
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.35);
  color: #f87171;
  font-size: 0.82rem;
  line-height: 1.4;
  word-break: break-word;
  margin-bottom: 0.25rem;
}

.toggle-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 0;
  border-top: 1px solid var(--color-border, #2a2a3e);
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.toggle-label {
  font-size: 0.9rem;
  font-weight: 500;
}

.toggle-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  color: var(--color-text, #fff);
}

.toggle-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-track {
  position: relative;
  width: 40px;
  height: 22px;
  background: #444;
  border-radius: 11px;
  transition: background 0.2s;
  display: flex;
  align-items: center;
}

.toggle-btn.active .toggle-track { background: #57f287; }

.toggle-thumb {
  position: absolute;
  left: 3px;
  width: 16px;
  height: 16px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s;
}

.toggle-btn.active .toggle-thumb { transform: translateX(18px); }

.toggle-text {
  font-size: 0.85rem;
  color: var(--color-text-muted, #888);
  min-width: 3.5rem;
}

/* Members */
.members-section {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border, #2a2a3e);
}

.members-header {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.section-label {
  font-size: 0.9rem;
  font-weight: 600;
}

.member-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.member-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  background: var(--color-input-bg, #0d0d1a);
}

.member-name {
  font-size: 0.9rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.member-username {
  font-size: 0.78rem;
  color: var(--color-text-muted, #888);
}

.owner-badge {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #5865f2;
  background: #5865f210;
  border: 1px solid #5865f230;
  border-radius: 4px;
  padding: 0.1rem 0.4rem;
  flex-shrink: 0;
}

.remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted, #888);
  padding: 0.1rem;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  transition: color 0.15s;
  font-size: 1rem;
}

.remove-btn:hover:not(:disabled) { color: #ed4245; }

.remove-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Add member / search */
.add-member { position: relative; }

.search-wrapper { position: relative; }

.search-input {
  width: 100%;
  padding: 0.45rem 0.8rem;
  border-radius: 7px;
  border: 1px solid var(--color-border, #2a2a3e);
  background: var(--color-input-bg, #0d0d1a);
  color: var(--color-text, #fff);
  font-size: 0.88rem;
  box-sizing: border-box;
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus { border-color: #5865f2; }

.search-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--color-surface, #1a1a2e);
  border: 1px solid var(--color-border, #2a2a3e);
  border-radius: 8px;
  list-style: none;
  padding: 0.3rem;
  margin: 0;
  z-index: 10;
  max-height: 200px;
  overflow-y: auto;
}

.search-result {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.45rem 0.6rem;
  border-radius: 5px;
  cursor: pointer;
  transition: background 0.1s;
}

.search-result:hover { background: var(--color-input-bg, #0d0d1a); }

.no-results {
  padding: 0.5rem 0.6rem;
  font-size: 0.85rem;
  color: var(--color-text-muted, #888);
}

/* Footer actions */
.form-actions {
  display: flex;
  gap: 0.75rem;
  padding-top: 0.25rem;
  border-top: 1px solid var(--color-border, #2a2a3e);
}

.loading {
  text-align: center;
  padding: 3rem;
  color: var(--color-text-muted, #888);
}
</style>
