import { computed, ref } from 'vue'
import type { EventPlayersCtx } from './event-ctx'
import type { EventPlayer, RoleRank } from '../types'

export function useEventPlayers({
  event, eventId, eventIsFull, ensureOwnerAction, setError, setNotice, hydrateSelections, clearLastBalancedFingerprint, eventStore, confirm,
}: EventPlayersCtx) {
  const addingPlayer = ref(false)
  const deletingPlayers = ref<Record<string, boolean>>({})
  const savingPlayerEdits = ref<Record<string, boolean>>({})
  const savingPlayerTeams = ref<Record<string, boolean>>({})
  const newPlayerName = ref('')
  const newPlayerRole = ref('DPS')
  const newPlayerRank = ref('Unranked')
  const newPlayerRoles = ref<RoleRank[]>([{ role: 'DPS', rank: 'Unranked' }])
  const editingPlayerId = ref<string | null>(null)
  const editPlayerName = ref('')
  const editPlayerRole = ref('DPS')
  const editPlayerRank = ref('Unranked')
  const editPlayerRoles = ref<RoleRank[]>([{ role: 'DPS', rank: 'Unranked' }])

  const canAddPlayer = computed(() => {
    if (!event.value || !newPlayerName.value.trim()) return false
    return newPlayerRoles.value.length > 0 && newPlayerRoles.value.every(rp => rp.role && rp.rank)
  })

  async function addPlayer() {
    if (!ensureOwnerAction() || !eventId.value || !canAddPlayer.value || addingPlayer.value) return
    if (eventIsFull.value) {
      setError('This event roster is full. Increase max players or remove a player.')
      return
    }
    addingPlayer.value = true
    try {
      const primaryRole = newPlayerRoles.value[0]
      const updatedEvent = await eventStore.addPlayer(eventId.value, {
        name: newPlayerName.value.trim(),
        role: primaryRole.role,
        rank: primaryRole.rank,
        roles: newPlayerRoles.value.filter(rp => rp.role && rp.rank),
      })
      event.value = updatedEvent
      hydrateSelections()
      newPlayerName.value = ''
      newPlayerRole.value = 'DPS'
      newPlayerRank.value = 'Unranked'
      newPlayerRoles.value = [{ role: 'DPS', rank: 'Unranked' }]
      setNotice('Player added to event roster')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add player')
    } finally {
      addingPlayer.value = false
    }
  }

  async function savePlayerEdit(playerId: string) {
    if (!ensureOwnerAction()) return
    const validRoles = editPlayerRoles.value.filter((rp) => rp.role && rp.rank)
    if (!eventId.value || !editPlayerName.value.trim() || validRoles.length === 0 || savingPlayerEdits.value[playerId]) return
    savingPlayerEdits.value = { ...savingPlayerEdits.value, [playerId]: true }
    try {
      const primaryRole = validRoles[0]
      const updatedEvent = await eventStore.updatePlayer(eventId.value, playerId, {
        name: editPlayerName.value.trim(),
        role: primaryRole.role,
        rank: primaryRole.rank,
        roles: validRoles,
      })
      event.value = updatedEvent
      hydrateSelections()
      editingPlayerId.value = null
      editPlayerName.value = ''
      editPlayerRole.value = 'DPS'
      editPlayerRank.value = 'Unranked'
      editPlayerRoles.value = [{ role: 'DPS', rank: 'Unranked' }]
      setNotice('Player updated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update player')
    } finally {
      savingPlayerEdits.value = { ...savingPlayerEdits.value, [playerId]: false }
    }
  }

  async function setPlayerTeam(playerId: string, teamId: string | null) {
    if (!ensureOwnerAction() || !eventId.value || savingPlayerTeams.value[playerId]) return
    savingPlayerTeams.value = { ...savingPlayerTeams.value, [playerId]: true }
    try {
      const updatedEvent = await eventStore.assignPlayerTeam(eventId.value, playerId, teamId)
      event.value = updatedEvent
      hydrateSelections()
      clearLastBalancedFingerprint()
      setNotice('Team assignment saved')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to assign team')
    } finally {
      savingPlayerTeams.value = { ...savingPlayerTeams.value, [playerId]: false }
    }
  }

  async function assignPlayerToTeam(playerId: string, teamId: string | null) {
    await setPlayerTeam(playerId, teamId)
  }

  async function assignPlayerToTeamWithRole(playerId: string, teamId: string | null, role: string, rank: string) {
    if (!ensureOwnerAction() || !eventId.value || savingPlayerTeams.value[playerId]) return
    savingPlayerTeams.value = { ...savingPlayerTeams.value, [playerId]: true }
    try {
      const updatedEvent = await eventStore.assignPlayerTeam(eventId.value, playerId, teamId, role, rank)
      event.value = updatedEvent
      hydrateSelections()
      clearLastBalancedFingerprint()
      setNotice('Team assignment saved')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to assign player')
    } finally {
      savingPlayerTeams.value = { ...savingPlayerTeams.value, [playerId]: false }
    }
  }

  async function removePlayerFromTeam(playerId: string) {
    await setPlayerTeam(playerId, null)
  }

  async function removePlayer(player: EventPlayer) {
    if (!ensureOwnerAction() || !eventId.value || deletingPlayers.value[player.id]) return

    const confirmed = await confirm.ask({
      title: 'Remove player?',
      message: `Remove player "${player.name}" from this event?`,
      confirmText: 'Remove player',
      tone: 'danger',
    })
    if (!confirmed) return

    deletingPlayers.value = { ...deletingPlayers.value, [player.id]: true }
    const previousEvent = event.value
    if (event.value) {
      event.value = {
        ...event.value,
        players: event.value.players.filter((current) => current.id !== player.id),
        matches: event.value.matches.map((currentMatch) => ({
          ...currentMatch,
          players: currentMatch.players.filter((currentPlayer) => currentPlayer.id !== player.id),
        })),
      }
    }
    try {
      await eventStore.deletePlayer(eventId.value, player.id)
      setNotice('Player removed from event roster')
    } catch (err) {
      event.value = previousEvent
      setError(err instanceof Error ? err.message : 'Failed to remove player')
    } finally {
      deletingPlayers.value = { ...deletingPlayers.value, [player.id]: false }
    }
  }

  return {
    addingPlayer, deletingPlayers, savingPlayerEdits, savingPlayerTeams,
    newPlayerName, newPlayerRole, newPlayerRank, newPlayerRoles,
    editingPlayerId, editPlayerName, editPlayerRole, editPlayerRank, editPlayerRoles,
    canAddPlayer,
    addPlayer, savePlayerEdit, setPlayerTeam, assignPlayerToTeam, assignPlayerToTeamWithRole,
    removePlayerFromTeam, removePlayer,
  }
}
