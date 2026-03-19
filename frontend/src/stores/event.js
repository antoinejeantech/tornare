import { defineStore } from 'pinia'
import { apiCall } from '../lib/api'

export const useEventStore = defineStore('event', {
  actions: {
    fetchEvent(eventId, options = {}) {
      return apiCall(`/api/events/${eventId}`, options)
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
    autoCreateSoloTeams(eventId) {
      return apiCall(`/api/events/${eventId}/teams/auto-solo`, {
        method: 'POST',
      })
    },
    autoBalanceTeams(eventId) {
      return apiCall(`/api/events/${eventId}/teams/auto-balance`, {
        method: 'POST',
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
    assignPlayerTeam(eventId, playerId, teamId, assignedRole = null, assignedRank = null) {
      return apiCall(`/api/events/${eventId}/team-members`, {
        method: 'POST',
        body: JSON.stringify({
          player_id: playerId,
          team_id: teamId,
          assigned_role: assignedRole,
          assigned_rank: assignedRank,
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
    setSignupVisibility(eventId, enabled) {
      return apiCall(`/api/events/${eventId}/signup-visibility`, {
        method: 'PUT',
        body: JSON.stringify({ enabled }),
      })
    },
    setFeaturedEvent(eventId, featured) {
      return apiCall(`/api/events/${eventId}/featured`, {
        method: 'PUT',
        body: JSON.stringify({ featured }),
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
