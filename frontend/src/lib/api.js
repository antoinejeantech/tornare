const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:8000'
const ACCESS_TOKEN_STORAGE_KEY = 'tornare_access_token'

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

export async function apiCall(path, options = {}) {
  const headers = {
    'Content-Type': 'application/json',
    ...(options.headers || {})
  }

  if (accessToken && !headers.Authorization) {
    headers.Authorization = `Bearer ${accessToken}`
  }

  const response = await fetch(`${apiBase}${path}`, {
    headers,
    ...options
  })

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
