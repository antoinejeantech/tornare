import { defineStore } from 'pinia'
import type { AuthSession, AuthUser } from '../types'
import { apiCall, clearAccessToken, getAccessToken, setAccessToken, syncAccessTokenFromStorage } from '../lib/api'

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
            this.syncTokensFromStorage()
          } catch {
            this.clearSession()
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
