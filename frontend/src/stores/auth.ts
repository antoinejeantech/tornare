import { defineStore } from 'pinia'
import type { AuthSession, AuthUser } from '../types'
import { ApiHttpError, apiBase, apiCall, clearAccessToken, getAccessToken, setAccessToken, syncAccessTokenFromStorage } from '../lib/api'

const REFRESH_TOKEN_STORAGE_KEY = 'tornare_refresh_token'
let initializePromise: Promise<void> | null = null

function getStoredRefreshToken(): string {
  if (typeof window === 'undefined') {
    return ''
  }

  return window.localStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) || ''
}

function setStoredRefreshToken(token: string): void {
  if (typeof window === 'undefined') {
    return
  }

  if (token) {
    window.localStorage.setItem(REFRESH_TOKEN_STORAGE_KEY, token)
  } else {
    window.localStorage.removeItem(REFRESH_TOKEN_STORAGE_KEY)
  }
}

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as AuthUser | null,
    accessToken: getAccessToken(),
    refreshToken: getStoredRefreshToken(),
    initialized: false,
  }),
  getters: {
    isAuthenticated: (state) => Boolean(state.accessToken),
  },
  actions: {
    syncTokensFromStorage(): void {
      this.accessToken = syncAccessTokenFromStorage()
      this.refreshToken = getStoredRefreshToken()
    },
    setSession(payload: AuthSession): void {
      this.user = payload.user
      this.accessToken = payload.access_token
      this.refreshToken = payload.refresh_token
      setAccessToken(payload.access_token)
      setStoredRefreshToken(payload.refresh_token)
    },
    clearSession(): void {
      this.user = null
      this.accessToken = ''
      this.refreshToken = ''
      clearAccessToken()
      setStoredRefreshToken('')
    },
    async register(payload: Record<string, unknown>): Promise<AuthSession> {
      const response = await apiCall<AuthSession>('/api/auth/register', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      this.setSession(response)
      return response
    },
    async login(payload: Record<string, unknown>): Promise<AuthSession> {
      const response = await apiCall<AuthSession>('/api/auth/login', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      this.setSession(response)
      return response
    },
    async fetchMe(): Promise<AuthUser> {
      const me = await apiCall<AuthUser>('/api/auth/me')
      this.user = me
      this.syncTokensFromStorage()
      return me
    },
    async initFromOAuth(accessToken: string, refreshToken: string): Promise<void> {
      setAccessToken(accessToken)
      setStoredRefreshToken(refreshToken)
      this.accessToken = accessToken
      this.refreshToken = refreshToken
      try {
        await this.fetchMe()
      } catch (err) {
        this.clearSession()
        throw err
      }
      this.initialized = true
    },
    async completeBnetSignup(pendingToken: string, email: string): Promise<void> {
      const response = await apiCall<AuthSession>('/api/auth/battlenet/complete', {
        method: 'POST',
        body: JSON.stringify({
          pending_token: pendingToken,
          email,
        }),
      })
      this.setSession(response)
      this.initialized = true
    },
    async connectBnetInit(): Promise<void> {
      const response = await fetch(`${apiBase}/api/auth/battlenet/connect-init`, {
        credentials: 'include',
        headers: { Authorization: `Bearer ${this.accessToken}` },
      })
      if (!response.ok) {
        const body = await response.json().catch(() => ({})) as { error?: string }
        throw new Error(body?.error || 'Failed to initiate Battle.net connection')
      }
      const { redirect_url } = await response.json() as { redirect_url: string }
      window.location.href = redirect_url
    },
    async disconnectBnet(): Promise<void> {
      await apiCall('/api/auth/battlenet/disconnect', { method: 'DELETE' })
      await this.fetchMe()
    },
    async connectDiscordInit(): Promise<void> {
      const response = await fetch(`${apiBase}/api/auth/discord/connect-init`, {
        credentials: 'include',
        headers: { Authorization: `Bearer ${this.accessToken}` },
      })
      if (!response.ok) {
        const body = await response.json().catch(() => ({})) as { error?: string }
        throw new Error(body?.error || 'Failed to initiate Discord connection')
      }
      const { redirect_url } = await response.json() as { redirect_url: string }
      window.location.href = redirect_url
    },
    async disconnectDiscord(): Promise<void> {
      await apiCall('/api/auth/discord/disconnect', { method: 'DELETE' })
      await this.fetchMe()
    },
    async refreshAccessToken(): Promise<AuthSession> {
      this.syncTokensFromStorage()

      const refreshToken = this.refreshToken
      if (!refreshToken) {
        throw new Error('No refresh token')
      }

      const response = await apiCall<AuthSession>('/api/auth/refresh', {
        method: 'POST',
        body: JSON.stringify({ refresh_token: refreshToken }),
      })
      this.setSession(response)
      return response
    },
    async logout(): Promise<void> {
      try {
        this.syncTokensFromStorage()

        const refreshToken = this.refreshToken
        if (refreshToken) {
          await apiCall('/api/auth/logout', {
            method: 'POST',
            body: JSON.stringify({ refresh_token: refreshToken }),
          })
        }
      } finally {
        this.clearSession()
      }
    },
    async initialize(): Promise<void> {
      if (this.initialized) {
        return
      }

      if (initializePromise) {
        return initializePromise
      }

      initializePromise = (async () => {
        this.syncTokensFromStorage()

        if (!this.accessToken && this.refreshToken) {
          try {
            await this.refreshAccessToken()
          } catch {
            this.clearSession()
          }
        }

        if (this.accessToken) {
          try {
            await this.fetchMe()
          } catch (err) {
            // Only clear session on definitive auth rejection (401/403).
            // Transient failures (5xx, network errors on cold backend) must
            // not delete the stored tokens — the access token is still valid
            // and subsequent API calls will carry it correctly.
            if (err instanceof ApiHttpError && (err.status === 401 || err.status === 403)) {
              this.clearSession()
            } else {
              this.syncTokensFromStorage()
            }
          }
        }

        this.initialized = true
      })()

      try {
        await initializePromise
      } finally {
        initializePromise = null
      }
    },
  },
})
