import { defineStore } from 'pinia'
import { apiCall } from '../lib/api'

export const useEventStore = defineStore('event', {
  actions: {
    fetchEvent(eventId) {
      return apiCall(`/api/events/${eventId}`)
    },
    updateEvent(eventId, payload) {
      return apiCall(`/api/events/${eventId}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      })
    },
    deleteEvent(eventId) {
      return apiCall(`/api/events/${eventId}`, {
        method: 'DELETE',
      })
    },
    createTeam(eventId, name) {
      return apiCall(`/api/events/${eventId}/teams`, {
        method: 'POST',
        body: JSON.stringify({ name }),
      })
    },
    updateTeam(eventId, teamId, name) {
      return apiCall(`/api/events/${eventId}/teams/${teamId}`, {
        method: 'PUT',
        body: JSON.stringify({ name }),
      })
    },
    deleteTeam(eventId, teamId) {
      return apiCall(`/api/events/${eventId}/teams/${teamId}`, {
        method: 'DELETE',
      })
    },
    addPlayer(eventId, payload) {
      return apiCall(`/api/events/${eventId}/players`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    updatePlayer(eventId, playerId, payload) {
      return apiCall(`/api/events/${eventId}/players/${playerId}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      })
    },
    deletePlayer(eventId, playerId) {
      return apiCall(`/api/events/${eventId}/players/${playerId}`, {
        method: 'DELETE',
      })
    },
    assignPlayerTeam(eventId, playerId, teamId) {
      return apiCall(`/api/events/${eventId}/team-members`, {
        method: 'POST',
        body: JSON.stringify({
          player_id: playerId,
          team_id: teamId,
        }),
      })
    },
    fetchSignupLink(eventId) {
      return apiCall(`/api/events/${eventId}/signup-link`)
    },
    rotateSignupLink(eventId) {
      return apiCall(`/api/events/${eventId}/signup-link/rotate`, {
        method: 'POST',
      })
    },
    listSignupRequests(eventId) {
      return apiCall(`/api/events/${eventId}/signup-requests`)
    },
    acceptSignupRequest(eventId, requestId) {
      return apiCall(`/api/events/${eventId}/signup-requests/${requestId}/accept`, {
        method: 'POST',
      })
    },
    declineSignupRequest(eventId, requestId) {
      return apiCall(`/api/events/${eventId}/signup-requests/${requestId}/decline`, {
        method: 'POST',
      })
    },
    fetchPublicSignupInfo(signupToken) {
      return apiCall(`/api/public/event-signups/${signupToken}`)
    },
    submitPublicSignupRequest(signupToken, payload) {
      return apiCall(`/api/public/event-signups/${signupToken}/requests`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
  },
})
