export const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:8000'

export function getApiBase(): string {
  return apiBase
}

const ACCESS_TOKEN_STORAGE_KEY = 'tornare_access_token'
const REFRESH_TOKEN_STORAGE_KEY = 'tornare_refresh_token'

let accessToken = ''
if (typeof window !== 'undefined') {
  accessToken = window.localStorage.getItem(ACCESS_TOKEN_STORAGE_KEY) || ''
}

export function setAccessToken(token: string | null | undefined): void {
  accessToken = token || ''
  if (typeof window !== 'undefined') {
    if (accessToken) {
      window.localStorage.setItem(ACCESS_TOKEN_STORAGE_KEY, accessToken)
    } else {
      window.localStorage.removeItem(ACCESS_TOKEN_STORAGE_KEY)
    }
  }
}

export function clearAccessToken(): void {
  setAccessToken('')
}

export function getAccessToken(): string {
  return accessToken
}

export function getStoredAccessToken(): string {
  if (typeof window === 'undefined') {
    return ''
  }

  return window.localStorage.getItem(ACCESS_TOKEN_STORAGE_KEY) || ''
}

export function syncAccessTokenFromStorage(): string {
  accessToken = getStoredAccessToken()
  return accessToken
}

function getRefreshToken(): string {
  if (typeof window === 'undefined') {
    return ''
  }
  return window.localStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) || ''
}

function setRefreshToken(token: string | null | undefined): void {
  if (typeof window === 'undefined') {
    return
  }

  if (token) {
    window.localStorage.setItem(REFRESH_TOKEN_STORAGE_KEY, token)
  } else {
    window.localStorage.removeItem(REFRESH_TOKEN_STORAGE_KEY)
  }
}

async function tryRefreshSession(): Promise<boolean> {
  const refreshToken = getRefreshToken()
  if (!refreshToken) {
    return false
  }

  let response: Response
  try {
    response = await fetch(`${apiBase}/api/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })
  } catch {
    // Network error — leave tokens intact so retries are possible later.
    return false
  }

  if (response.status === 401 || response.status === 403) {
    // Refresh token definitively rejected by the server — clear the session.
    clearAccessToken()
    setRefreshToken('')
    return false
  }

  if (!response.ok) {
    // Server error (5xx, etc.) — leave tokens intact.
    return false
  }

  let body: { access_token?: string; refresh_token?: string }
  try {
    body = await response.json()
  } catch {
    return false
  }

  setAccessToken(body.access_token || '')
  setRefreshToken(body.refresh_token || '')
  return Boolean(body.access_token)
}

export class ApiHttpError extends Error {
  constructor(public readonly status: number, message: string) {
    super(message)
    this.name = 'ApiHttpError'
  }
}

export interface ApiCallOptions extends Omit<RequestInit, 'headers'> {
  headers?: Record<string, string>
}

export async function apiCall<T = unknown>(path: string, options: ApiCallOptions = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers || {})
  }

  if (accessToken && !headers['Authorization']) {
    headers['Authorization'] = `Bearer ${accessToken}`
  }

  let response = await fetch(`${apiBase}${path}`, {
    ...options,
    headers,
  })

  // If access token expired, refresh once and retry the original request.
  // Token clearing is handled inside tryRefreshSession on definitive rejection.
  if (response.status === 401 && path !== '/api/auth/refresh') {
    const refreshed = await tryRefreshSession()
    if (refreshed) {
      const retryHeaders: Record<string, string> = {
        ...headers,
        Authorization: `Bearer ${accessToken}`,
      }

      response = await fetch(`${apiBase}${path}`, {
        ...options,
        headers: retryHeaders,
      })
    }
  }

  if (!response.ok) {
    let message = `API error: ${response.status}`
    try {
      const body = await response.json()
      if (body?.error) {
        message = body.error
      }
    } catch {
      // no-op
    }
    throw new ApiHttpError(response.status, message)
  }

  return response.json() as Promise<T>
}

// ---------------------------------------------------------------------------
// Discord guild API
// ---------------------------------------------------------------------------

export interface UserSearchResult {
  id: string
  username: string | null
  display_name: string
}

export function searchUsers(query: string): Promise<UserSearchResult[]> {
  return apiCall<UserSearchResult[]>(`/api/users?search=${encodeURIComponent(query)}`)
}

export interface DiscordGuild {
  id: string
  guild_id: string
  guild_name: string | null
  owner_user_id: string | null
  channel_id: string
  announcements_enabled: boolean
  mention_roles: string[]
  last_post_error?: string | null
  last_post_error_at?: string | null
}

export interface GuildMember {
  user_id: string
  username: string | null
  display_name: string
  added_at: string
}

export interface UpsertGuildInput {
  guild_id: string
  guild_name?: string
  channel_id: string
}

export function getDiscordGuilds(): Promise<DiscordGuild[]> {
  return apiCall<DiscordGuild[]>('/api/discord/guilds')
}

export function upsertDiscordGuild(input: UpsertGuildInput): Promise<DiscordGuild> {
  return apiCall<DiscordGuild>('/api/discord/guild', { method: 'PUT', body: JSON.stringify(input) })
}

export function deleteDiscordGuild(guildId: string): Promise<{ message: string }> {
  return apiCall<{ message: string }>(`/api/discord/guild/${guildId}`, { method: 'DELETE' })
}

export function getDiscordBotInviteUrl(): Promise<{ url: string }> {
  return apiCall<{ url: string }>('/api/discord/invite')
}

export function toggleDiscordAnnouncements(guildId: string, enabled: boolean): Promise<DiscordGuild> {
  return apiCall<DiscordGuild>(`/api/discord/guild/${guildId}/announcements`, {
    method: 'PATCH',
    body: JSON.stringify({ enabled }),
  })
}

export function listGuildMembers(guildId: string): Promise<GuildMember[]> {
  return apiCall<GuildMember[]>(`/api/discord/guild/${guildId}/members`)
}

export function addGuildMember(guildId: string, userId: string): Promise<GuildMember[]> {
  return apiCall<GuildMember[]>(`/api/discord/guild/${guildId}/members`, {
    method: 'POST',
    body: JSON.stringify({ user_id: userId }),
  })
}

export function removeGuildMember(guildId: string, userId: string): Promise<GuildMember[]> {
  return apiCall<GuildMember[]>(`/api/discord/guild/${guildId}/members/${userId}`, {
    method: 'DELETE',
  })
}

export function setGuildMentionRoles(guildId: string, roles: string[]): Promise<DiscordGuild> {
  return apiCall<DiscordGuild>(`/api/discord/guild/${guildId}/mention-roles`, {
    method: 'PATCH',
    body: JSON.stringify({ roles }),
  })
}
