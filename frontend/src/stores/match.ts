import { defineStore } from 'pinia'
import type { Event, EventMatch } from '../types'
import { apiCall } from '../lib/api'

export const useMatchStore = defineStore('match', {
  actions: {
    createMatchForEvent(eventId: string | number, payload: Record<string, unknown>): Promise<EventMatch> {
      return apiCall<EventMatch>(`/api/events/${eventId}/matches`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    setMatchupForEvent(eventId: string | number, matchId: string | number, payload: Record<string, unknown>): Promise<EventMatch> {
      return apiCall<EventMatch>(`/api/events/${eventId}/matches/${matchId}/matchup`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    generateTourneyBracket(eventId: string | number, mode = 'random'): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/tourney/generate`, {
        method: 'POST',
        body: JSON.stringify({ mode }),
      })
    },
    clearTourneyBracket(eventId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/tourney/clear`, {
        method: 'POST',
      })
    },
    reportMatchWinner(eventId: string | number, matchId: string | number, winnerTeamId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}/matches/${matchId}/winner`, {
        method: 'POST',
        body: JSON.stringify({ winner_team_id: winnerTeamId }),
      })
    },
    cancelMatchWinner(eventId: string | number, matchId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}/matches/${matchId}/winner/cancel`, {
        method: 'POST',
      })
    },
    deleteMatch(matchId: string | number): Promise<void> {
      return apiCall<void>(`/api/matches/${matchId}`, {
        method: 'DELETE',
      })
    },
    updateMatchStartDate(eventId: string | number, matchId: string | number, startDate: string | null): Promise<EventMatch> {
      return apiCall<EventMatch>(`/api/events/${eventId}/matches/${matchId}/start-date`, {
        method: 'POST',
        body: JSON.stringify({ start_date: startDate }),
      })
    },
  },
})
