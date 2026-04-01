import { defineStore } from 'pinia'
import type { AutoBalanceResponse, Event, PublicSignupInfo, SignupLink, SignupRequest } from '../types'
import type { ApiCallOptions } from '../lib/api'
import { apiCall } from '../lib/api'

export const useEventStore = defineStore('event', {
  actions: {
    fetchEvent(eventId: string | number, options: ApiCallOptions = {}): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}`, options)
    },
    updateEvent(eventId: string | number, payload: Record<string, unknown>): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      })
    },
    deleteEvent(eventId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}`, {
        method: 'DELETE',
      })
    },
    createTeam(eventId: string | number, name: string): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/teams`, {
        method: 'POST',
        body: JSON.stringify({ name }),
      })
    },
    autoCreateSoloTeams(eventId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/teams/auto-solo`, {
        method: 'POST',
      })
    },
    autoBalanceTeams(eventId: string | number): Promise<AutoBalanceResponse> {
      return apiCall<AutoBalanceResponse>(`/api/events/${eventId}/teams/auto-balance`, {
        method: 'POST',
      })
    },
    updateTeam(eventId: string | number, teamId: string | number, name: string): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/teams/${teamId}`, {
        method: 'PUT',
        body: JSON.stringify({ name }),
      })
    },
    deleteTeam(eventId: string | number, teamId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}/teams/${teamId}`, {
        method: 'DELETE',
      })
    },
    addPlayer(eventId: string | number, payload: Record<string, unknown>): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/players`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
    updatePlayer(eventId: string | number, playerId: string | number, payload: Record<string, unknown>): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/players/${playerId}`, {
        method: 'PUT',
        body: JSON.stringify(payload),
      })
    },
    deletePlayer(eventId: string | number, playerId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}/players/${playerId}`, {
        method: 'DELETE',
      })
    },
    assignPlayerTeam(
      eventId: string | number,
      playerId: string | number,
      teamId: string | number | null,
      assignedRole: string | null = null,
      assignedRank: string | null = null,
    ): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/team-members`, {
        method: 'POST',
        body: JSON.stringify({
          player_id: playerId,
          team_id: teamId,
          assigned_role: assignedRole,
          assigned_rank: assignedRank,
        }),
      })
    },
    fetchSignupLink(eventId: string | number): Promise<SignupLink> {
      return apiCall<SignupLink>(`/api/events/${eventId}/signup-link`)
    },
    rotateSignupLink(eventId: string | number): Promise<SignupLink> {
      return apiCall<SignupLink>(`/api/events/${eventId}/signup-link/rotate`, {
        method: 'POST',
      })
    },
    setSignupVisibility(eventId: string | number, enabled: boolean): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/signup-visibility`, {
        method: 'PUT',
        body: JSON.stringify({ enabled }),
      })
    },
    setFeaturedEvent(eventId: string | number, featured: boolean): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/featured`, {
        method: 'PUT',
        body: JSON.stringify({ featured }),
      })
    },
    publishEvent(eventId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/publish`, { method: 'POST' })
    },
    unpublishEvent(eventId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/unpublish`, { method: 'POST' })
    },
    endEvent(eventId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/end`, { method: 'POST' })
    },
    listSignupRequests(eventId: string | number): Promise<SignupRequest[]> {
      return apiCall<SignupRequest[]>(`/api/events/${eventId}/signup-requests`)
    },
    acceptSignupRequest(eventId: string | number, requestId: string | number): Promise<Event> {
      return apiCall<Event>(`/api/events/${eventId}/signup-requests/${requestId}/accept`, {
        method: 'POST',
      })
    },
    declineSignupRequest(eventId: string | number, requestId: string | number): Promise<void> {
      return apiCall<void>(`/api/events/${eventId}/signup-requests/${requestId}/decline`, {
        method: 'POST',
      })
    },
    fetchPublicSignupInfo(signupToken: string): Promise<PublicSignupInfo> {
      return apiCall<PublicSignupInfo>(`/api/public/event-signups/${signupToken}`)
    },
    submitPublicSignupRequest(signupToken: string, payload: Record<string, unknown>): Promise<unknown> {
      return apiCall(`/api/public/event-signups/${signupToken}/requests`, {
        method: 'POST',
        body: JSON.stringify(payload),
      })
    },
  },
})
