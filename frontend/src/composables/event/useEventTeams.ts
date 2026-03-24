import { computed, ref } from 'vue'
import type { SharedEventCtx } from './event-ctx'
import type { Event, EventTeam } from '../../types'

export function useEventTeams({
  event, eventId, ensureOwnerAction, setError, setNotice, hydrateSelections, eventStore, confirm,
}: SharedEventCtx) {
  const creatingTeam = ref(false)
  const creatingSoloTeams = ref(false)
  const balancingTeams = ref(false)
  const deletingTeams = ref<Record<string, boolean>>({})
  const savingTeamEdits = ref<Record<string, boolean>>({})
  const newTeamName = ref('')
  const editingTeamId = ref<string | null>(null)
  const editTeamName = ref('')
  const lastBalanceSummary = ref('')
  const lastBalancedFingerprint = ref<string | null>(null)

  const canCreateTeam = computed(() => Boolean(event.value) && newTeamName.value.trim().length > 0)

  function teamsFingerprint(ev: Event | null | undefined): string | null {
    if (!Array.isArray(ev?.players)) return null
    return JSON.stringify({
      format: String(ev?.format || ''),
      teams: Array.isArray(ev?.teams) ? ev.teams.map((t) => String(t.id || '')).sort() : [],
      players: ev.players
        .map((player) => ({
          id: String(player?.id || ''),
          team_id: player?.team_id || null,
          role: player?.role || null,
          rank: player?.rank || null,
          assigned_role: player?.assigned_role || null,
          assigned_rank: player?.assigned_rank || null,
          roles: Array.isArray(player?.roles)
            ? player.roles.map((entry) => ({ role: entry?.role || null, rank: entry?.rank || null }))
            : [],
        }))
        .sort((a, b) => (a.id < b.id ? -1 : 1)),
    })
  }

  const teamsAreAlreadyBalanced = computed(() => {
    if (!lastBalancedFingerprint.value) return false
    return teamsFingerprint(event.value) === lastBalancedFingerprint.value
  })

  function clearLastBalancedFingerprint() {
    lastBalancedFingerprint.value = null
  }

  async function createTeam() {
    if (!ensureOwnerAction() || !eventId.value || !canCreateTeam.value || creatingTeam.value) return
    creatingTeam.value = true
    try {
      const updatedEvent = await eventStore.createTeam(eventId.value, newTeamName.value.trim())
      event.value = updatedEvent
      hydrateSelections()
      newTeamName.value = ''
      setNotice('Team created')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create team')
    } finally {
      creatingTeam.value = false
    }
  }

  async function autoCreateSoloTeams() {
    if (!ensureOwnerAction() || !eventId.value || creatingSoloTeams.value) return
    creatingSoloTeams.value = true
    try {
      const updatedEvent = await eventStore.autoCreateSoloTeams(eventId.value)
      event.value = updatedEvent
      hydrateSelections()
      setNotice('Created solo teams for unassigned players')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to auto-create solo teams')
    } finally {
      creatingSoloTeams.value = false
    }
  }

  async function autoBalanceTeams() {
    if (!ensureOwnerAction() || !eventId.value || balancingTeams.value) return
    balancingTeams.value = true
    try {
      const response = await eventStore.autoBalanceTeams(eventId.value)
      event.value = response.event
      hydrateSelections()
      lastBalanceSummary.value = response.summary || 'Teams auto-balanced by rank ELO'
      lastBalancedFingerprint.value = teamsFingerprint(response.event)
      setNotice(lastBalanceSummary.value)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to auto-balance teams')
    } finally {
      balancingTeams.value = false
    }
  }

  async function saveTeamEdit(teamId: string) {
    if (!ensureOwnerAction() || !eventId.value || !editTeamName.value.trim() || savingTeamEdits.value[teamId]) return
    savingTeamEdits.value = { ...savingTeamEdits.value, [teamId]: true }
    try {
      const updatedEvent = await eventStore.updateTeam(eventId.value, teamId, editTeamName.value.trim())
      event.value = updatedEvent
      hydrateSelections()
      editingTeamId.value = null
      editTeamName.value = ''
      setNotice('Team updated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update team')
    } finally {
      savingTeamEdits.value = { ...savingTeamEdits.value, [teamId]: false }
    }
  }

  async function deleteTeam(team: EventTeam) {
    if (!ensureOwnerAction() || !eventId.value || deletingTeams.value[team.id]) return

    const confirmed = await confirm.ask({
      title: 'Delete team?',
      message: `Delete team "${team.name}"?`,
      confirmText: 'Delete team',
      tone: 'danger',
    })
    if (!confirmed) return

    deletingTeams.value = { ...deletingTeams.value, [team.id]: true }
    try {
      await eventStore.deleteTeam(eventId.value, team.id)

      if (event.value) {
        const deletedTeamId = String(team.id)
        event.value = {
          ...event.value,
          teams: event.value.teams.filter((entry) => String(entry.id) !== deletedTeamId),
          players: event.value.players.map((player) => {
            if (String(player.team_id || '') !== deletedTeamId) return player
            return { ...player, team_id: null, team: null }
          }),
          matches: event.value.matches.map((match) => {
            const clearsTeamA = String(match.team_a_id || '') === deletedTeamId
            const clearsTeamB = String(match.team_b_id || '') === deletedTeamId
            const clearsWinner = String(match.winner_team_id || '') === deletedTeamId
            if (!clearsTeamA && !clearsTeamB && !clearsWinner) return match
            return {
              ...match,
              team_a_id: clearsTeamA ? null : match.team_a_id,
              team_a_name: clearsTeamA ? null : match.team_a_name,
              team_b_id: clearsTeamB ? null : match.team_b_id,
              team_b_name: clearsTeamB ? null : match.team_b_name,
              winner_team_id: clearsWinner ? null : match.winner_team_id,
              winner_team_name: clearsWinner ? null : match.winner_team_name,
            }
          }),
        }
      }

      if (editingTeamId.value === team.id) {
        editingTeamId.value = null
        editTeamName.value = ''
      }
      hydrateSelections()
      setNotice('Team deleted')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete team')
    } finally {
      deletingTeams.value = { ...deletingTeams.value, [team.id]: false }
    }
  }

  return {
    creatingTeam, creatingSoloTeams, balancingTeams, deletingTeams, savingTeamEdits,
    newTeamName, editingTeamId, editTeamName, canCreateTeam,
    lastBalanceSummary, lastBalancedFingerprint, teamsAreAlreadyBalanced,
    clearLastBalancedFingerprint,
    createTeam, autoCreateSoloTeams, autoBalanceTeams, saveTeamEdit, deleteTeam,
  }
}
