import { defineStore } from 'pinia'
import { apiCall, clearAccessToken, getAccessToken, setAccessToken } from '../lib/api'

const REFRESH_TOKEN_STORAGE_KEY = 'tornare_refresh_token'

function getStoredRefreshToken() {
  if (typeof window === 'undefined') {
    return ''
  }

  return window.localStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) || ''
}

function setStoredRefreshToken(token) {
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
    user: null,
    accessToken: getAccessToken(),
    refreshToken: getStoredRefreshToken(),
    initialized: false,
  }),
  getters: {
    isAuthenticated: (state) => Boolean(state.accessToken),
  },
  actions: {
    setSession(payload) {
      this.user = payload.user
      this.accessToken = payload.access_token
      this.refreshToken = payload.refresh_token
      setAccessToken(payload.access_token)
      setStoredRefreshToken(payload.refresh_token)
    },
    clearSession() {
      this.user = null
      this.accessToken = ''
      this.refreshToken = ''
      clearAccessToken()
      setStoredRefreshToken('')
    },
    async register(payload) {
      const response = await apiCall('/api/auth/register', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      this.setSession(response)
      return response
    },
    async login(payload) {
      const response = await apiCall('/api/auth/login', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      this.setSession(response)
      return response
    },
    async fetchMe() {
      const me = await apiCall('/api/auth/me')
      this.user = me
      return me
    },
    async refreshAccessToken() {
      if (!this.refreshToken) {
        throw new Error('No refresh token')
      }

      const response = await apiCall('/api/auth/refresh', {
        method: 'POST',
        body: JSON.stringify({ refresh_token: this.refreshToken }),
      })
      this.setSession(response)
      return response
    },
    async logout() {
      try {
        if (this.refreshToken) {
          await apiCall('/api/auth/logout', {
            method: 'POST',
            body: JSON.stringify({ refresh_token: this.refreshToken }),
          })
        }
      } finally {
        this.clearSession()
      }
    },
    async initialize() {
      if (this.initialized) {
        return
      }

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
        } catch {
          this.clearSession()
        }
      }

      this.initialized = true
    },
  },
})
