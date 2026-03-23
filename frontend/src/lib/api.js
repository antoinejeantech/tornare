const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:8000'
const ACCESS_TOKEN_STORAGE_KEY = 'tornare_access_token'
const REFRESH_TOKEN_STORAGE_KEY = 'tornare_refresh_token'

let accessToken = ''
if (typeof window !== 'undefined') {
  accessToken = window.localStorage.getItem(ACCESS_TOKEN_STORAGE_KEY) || ''
}

export function setAccessToken(token) {
  accessToken = token || ''
  if (typeof window !== 'undefined') {
    if (accessToken) {
      window.localStorage.setItem(ACCESS_TOKEN_STORAGE_KEY, accessToken)
    } else {
      window.localStorage.removeItem(ACCESS_TOKEN_STORAGE_KEY)
    }
  }
}

export function clearAccessToken() {
  setAccessToken('')
}

export function getAccessToken() {
  return accessToken
}

export function getStoredAccessToken() {
  if (typeof window === 'undefined') {
    return ''
  }

  return window.localStorage.getItem(ACCESS_TOKEN_STORAGE_KEY) || ''
}

export function syncAccessTokenFromStorage() {
  accessToken = getStoredAccessToken()
  return accessToken
}

function getRefreshToken() {
  if (typeof window === 'undefined') {
    return ''
  }
  return window.localStorage.getItem(REFRESH_TOKEN_STORAGE_KEY) || ''
}

function setRefreshToken(token) {
  if (typeof window === 'undefined') {
    return
  }

  if (token) {
    window.localStorage.setItem(REFRESH_TOKEN_STORAGE_KEY, token)
  } else {
    window.localStorage.removeItem(REFRESH_TOKEN_STORAGE_KEY)
  }
}

async function tryRefreshSession() {
  const refreshToken = getRefreshToken()
  if (!refreshToken) {
    return false
  }

  let response
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

  let body
  try {
    body = await response.json()
  } catch {
    return false
  }

  setAccessToken(body.access_token || '')
  setRefreshToken(body.refresh_token || '')
  return Boolean(body.access_token)
}

export async function apiCall(path, options = {}) {
  const headers = {
    'Content-Type': 'application/json',
    ...(options.headers || {})
  }

  if (accessToken && !headers.Authorization) {
    headers.Authorization = `Bearer ${accessToken}`
  }

  let response = await fetch(`${apiBase}${path}`, {
    headers,
    ...options
  })

  // If access token expired, refresh once and retry the original request.
  // Token clearing is handled inside tryRefreshSession on definitive rejection.
  if (response.status === 401 && path !== '/api/auth/refresh') {
    const refreshed = await tryRefreshSession()
    if (refreshed) {
      const retryHeaders = {
        ...headers,
        Authorization: `Bearer ${accessToken}`,
      }

      response = await fetch(`${apiBase}${path}`, {
        headers: retryHeaders,
        ...options,
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
    throw new Error(message)
  }

  return response.json()
}
