import { computed, nextTick, ref } from 'vue'
import type { EventMatchesCtx } from './event-ctx'
import { normalizeDatetimeLocalInput } from '../../lib/dates'

export function useEventMatches({
  event, eventId, isTourneyEvent, ensureOwnerAction, setError, setNotice, hydrateSelections, matchupSelections, eventStore, matchStore, confirm,
}: EventMatchesCtx) {
  const creatingMatch = ref(false)
  const clearingBracket = ref(false)
  const deletingMatchId = ref<string | null>(null)
  const savingMatchups = ref<Record<string, boolean>>({})
  const reportingWinners = ref<Record<string, boolean>>({})
  const cancellingWinners = ref<Record<string, boolean>>({})
  const newMatchTitle = ref('')
  const newMatchMap = ref('')
  const newMatchTeamAId = ref('')
  const newMatchTeamBId = ref('')
  const newMatchStartDate = ref('')

  const canCreateMatch = computed(
    () => Boolean(event.value) && newMatchTitle.value.trim().length > 0 && newMatchMap.value.trim().length > 0
  )

  async function saveMatchup(matchId: string) {
    if (!ensureOwnerAction() || !eventId.value || savingMatchups.value[matchId]) return false
    const selection = matchupSelections.value[matchId] || { teamAId: '', teamBId: '' }
    const teamAId = selection.teamAId || null
    const teamBId = selection.teamBId || null
    savingMatchups.value = { ...savingMatchups.value, [matchId]: true }
    try {
      const updatedMatch = await matchStore.setMatchupForEvent(eventId.value, matchId, { team_a_id: teamAId, team_b_id: teamBId })
      if (event.value) {
        event.value = { ...event.value, matches: event.value.matches.map((match) => (match.id === matchId ? updatedMatch : match)) }
        hydrateSelections()
      }
      setNotice('Matchup saved')
      return true
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save matchup')
      return false
    } finally {
      savingMatchups.value = { ...savingMatchups.value, [matchId]: false }
    }
  }

  async function createMatch() {
    if (!ensureOwnerAction() || !eventId.value || !canCreateMatch.value || creatingMatch.value) return

    let normalizedStartDate = null
    try {
      normalizedStartDate = normalizeDatetimeLocalInput(newMatchStartDate.value, 'match start date')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid match start date')
      return
    }

    creatingMatch.value = true
    try {
      const created = await matchStore.createMatchForEvent(eventId.value, {
        title: newMatchTitle.value.trim(),
        map: newMatchMap.value.trim(),
        start_date: normalizedStartDate,
      })
      if (event.value) {
        event.value = { ...event.value, matches: [created, ...event.value.matches] }
        const teamAId = newMatchTeamAId.value || null
        const teamBId = newMatchTeamBId.value || null
        matchupSelections.value = {
          ...matchupSelections.value,
          [created.id]: { teamAId: teamAId ? String(teamAId) : '', teamBId: teamBId ? String(teamBId) : '' },
        }
        if (teamAId && teamBId && teamAId !== teamBId) {
          try {
            const updatedMatch = await matchStore.setMatchupForEvent(eventId.value, created.id, { team_a_id: teamAId, team_b_id: teamBId })
            event.value = { ...event.value, matches: event.value.matches.map((m) => (m.id === created.id ? updatedMatch : m)) }
            hydrateSelections()
          } catch (err) {
            setError(err instanceof Error ? err.message : 'Match created but failed to set teams')
          }
        }
      }
      newMatchTitle.value = ''
      newMatchMap.value = ''
      newMatchTeamAId.value = ''
      newMatchTeamBId.value = ''
      newMatchStartDate.value = ''
      setNotice('Match created in event')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create match')
    } finally {
      creatingMatch.value = false
    }
  }

  async function updateMatchStartDate(matchId: string, startDate: string) {
    if (!ensureOwnerAction() || !eventId.value) return

    let normalizedStartDate = null
    try {
      normalizedStartDate = normalizeDatetimeLocalInput(startDate, 'match start date')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid match start date')
      return
    }

    try {
      const updated = await matchStore.updateMatchStartDate(eventId.value, matchId, normalizedStartDate)
      if (event.value) {
        event.value = { ...event.value, matches: event.value.matches.map((m) => (m.id === matchId ? updated : m)) }
      }
      setNotice('Start date updated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update start date')
    }
  }

  async function generateTourneyBracket(mode = 'random') {
    if (!ensureOwnerAction() || !eventId.value || !isTourneyEvent.value || creatingMatch.value) return
    const hasPlayedMatches = Boolean(event.value?.matches?.some((match) => Boolean(match.winner_team_id)))
    if (hasPlayedMatches) {
      setError('Cannot regenerate bracket after matches have been played')
      return
    }
    creatingMatch.value = true
    try {
      const updatedEvent = await matchStore.generateTourneyBracket(eventId.value, mode)
      event.value = updatedEvent
      hydrateSelections()
      setNotice(mode === 'empty' ? 'Empty tournament bracket generated' : 'Random tournament bracket generated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate bracket')
    } finally {
      creatingMatch.value = false
    }
  }

  async function clearTourneyBracket() {
    if (!ensureOwnerAction() || !eventId.value || !isTourneyEvent.value || clearingBracket.value) return
    const hasPlayedMatches = Boolean(event.value?.matches?.some((match) => Boolean(match.winner_team_id)))
    if (hasPlayedMatches) {
      setError('Cannot clear bracket after matches have been played')
      return
    }
    if (!event.value?.matches?.length) {
      setNotice('No generated bracket to clear')
      return
    }
    const confirmed = await confirm.ask({
      title: 'Clear bracket?',
      message: 'Delete generated bracket matches? This cannot be undone.',
      confirmText: 'Delete bracket',
      tone: 'danger',
    })
    if (!confirmed) return
    clearingBracket.value = true
    try {
      const updatedEvent = await matchStore.clearTourneyBracket(eventId.value)
      event.value = updatedEvent
      hydrateSelections()
      setNotice('Bracket cleared')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear bracket')
    } finally {
      clearingBracket.value = false
    }
  }

  async function reportMatchWinner(matchId: string, winnerTeamId: string) {
    if (!ensureOwnerAction() || !eventId.value || !winnerTeamId || reportingWinners.value[matchId]) return
    reportingWinners.value = { ...reportingWinners.value, [matchId]: true }
    const savedWindowY = window.scrollY
    const savedWindowX = window.scrollX
    const savedBracketScrollLeft = document.querySelector('.tourney-bracket-wrap')?.scrollLeft ?? 0
    try {
      await matchStore.reportMatchWinner(eventId.value, matchId, winnerTeamId)
      const updatedEvent = await eventStore.fetchEvent(eventId.value)
      event.value = updatedEvent
      hydrateSelections()
      await nextTick()
      window.scrollTo({ top: savedWindowY, left: savedWindowX })
      const bracketWrap = document.querySelector('.tourney-bracket-wrap')
      if (bracketWrap) bracketWrap.scrollLeft = savedBracketScrollLeft
      setNotice('Winner reported')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to report winner')
    } finally {
      reportingWinners.value = { ...reportingWinners.value, [matchId]: false }
    }
  }

  async function cancelMatchWinner(matchId: string) {
    if (!ensureOwnerAction() || !eventId.value || cancellingWinners.value[matchId]) return
    const confirmed = await confirm.ask({
      title: 'Cancel match result?',
      message: isTourneyEvent.value
        ? 'Downstream bracket progression will be reset where needed.'
        : 'The recorded result for this match will be cleared.',
      confirmText: 'Cancel result',
      tone: 'warning',
    })
    if (!confirmed) return
    cancellingWinners.value = { ...cancellingWinners.value, [matchId]: true }
    const savedWindowY = window.scrollY
    const savedWindowX = window.scrollX
    const savedBracketScrollLeft = document.querySelector('.tourney-bracket-wrap')?.scrollLeft ?? 0
    try {
      await matchStore.cancelMatchWinner(eventId.value, matchId)
      const updatedEvent = await eventStore.fetchEvent(eventId.value)
      event.value = updatedEvent
      hydrateSelections()
      await nextTick()
      window.scrollTo({ top: savedWindowY, left: savedWindowX })
      const bracketWrap = document.querySelector('.tourney-bracket-wrap')
      if (bracketWrap) bracketWrap.scrollLeft = savedBracketScrollLeft
      setNotice('Match result cancelled')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel match result')
    } finally {
      cancellingWinners.value = { ...cancellingWinners.value, [matchId]: false }
    }
  }

  async function deleteMatch(matchId: string) {
    if (!ensureOwnerAction() || deletingMatchId.value) return
    const target = event.value?.matches.find((match) => match.id === matchId)
    const confirmed = await confirm.ask({
      title: 'Delete match?',
      message: `Delete match "${target?.title || matchId}"?`,
      confirmText: 'Delete match',
      tone: 'danger',
    })
    if (!confirmed) return
    deletingMatchId.value = matchId
    try {
      await matchStore.deleteMatch(matchId)
      if (event.value) {
        event.value = { ...event.value, matches: event.value.matches.filter((match) => match.id !== matchId) }
      }
      setNotice('Match deleted')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete match')
    } finally {
      deletingMatchId.value = null
    }
  }

  return {
    creatingMatch, clearingBracket, deletingMatchId, savingMatchups, reportingWinners, cancellingWinners,
    newMatchTitle, newMatchMap, newMatchTeamAId, newMatchTeamBId, newMatchStartDate, canCreateMatch,
    saveMatchup, createMatch, updateMatchStartDate, generateTourneyBracket, clearTourneyBracket,
    reportMatchWinner, cancelMatchWinner, deleteMatch,
  }
}
