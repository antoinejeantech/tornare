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
    generateTourneyBracket(eventId, mode = 'random') {
      return apiCall(`/api/events/${eventId}/tourney/generate`, {
        method: 'POST',
        body: JSON.stringify({ mode }),
      })
    },
    clearTourneyBracket(eventId) {
      return apiCall(`/api/events/${eventId}/tourney/clear`, {
        method: 'POST',
      })
    },
    reportMatchWinner(eventId, matchId, winnerTeamId) {
      return apiCall(`/api/events/${eventId}/matches/${matchId}/winner`, {
        method: 'POST',
        body: JSON.stringify({ winner_team_id: winnerTeamId }),
      })
    },
    cancelMatchWinner(eventId, matchId) {
      return apiCall(`/api/events/${eventId}/matches/${matchId}/winner/cancel`, {
        method: 'POST',
      })
    },
    deleteMatch(matchId) {
      return apiCall(`/api/matches/${matchId}`, {
        method: 'DELETE',
      })
    },
    updateMatchStartDate(eventId, matchId, startDate) {
      return apiCall(`/api/events/${eventId}/matches/${matchId}/start-date`, {
        method: 'POST',
        body: JSON.stringify({ start_date: startDate }),
      })
    },
  },
})
