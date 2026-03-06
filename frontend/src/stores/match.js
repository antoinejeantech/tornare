import { defineStore } from 'pinia'
import { apiCall } from '../lib/api'

export const useMatchStore = defineStore('match', {
  actions: {
    createMatchForEvent(eventId, payload) {
      return apiCall(`/api/events/${eventId}/matches`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    setMatchupForEvent(eventId, matchId, payload) {
      return apiCall(`/api/events/${eventId}/matches/${matchId}/matchup`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    fetchMatch(matchId) {
      return apiCall(`/api/matches/${matchId}`)
    },
    deleteMatch(matchId) {
      return apiCall(`/api/matches/${matchId}`, {
        method: 'DELETE',
      })
    },
  },
})
