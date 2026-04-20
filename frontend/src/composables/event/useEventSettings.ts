import { computed, ref } from 'vue'
import type { EventSettingsCtx } from './event-ctx'
import { formatOptionsForType } from '../../lib/event-format'
import { isoToDatetimeLocalValue, normalizeDatetimeLocalInput } from '../../lib/dates'

export function useEventSettings({
  event, eventId, ensureOwnerAction, setError, setNotice, hydrateSelections, eventStore, confirm, router,
}: EventSettingsCtx) {
  const editEventName = ref('')
  const editEventDescription = ref('')
  const editEventStartDate = ref('')
  const editEventFormat = ref('5v5')
  const editEventMaxPlayers = ref(10)
  const editEventRequireDiscord = ref(false)
  const editEventRequireBattletag = ref(false)
  const editEventDiscordAnnounce = ref(true)
  const updatingEvent = ref(false)
  const deletingEvent = ref(false)

  const canSaveEventMeta = computed(() => {
    const nameOk = editEventName.value.trim().length > 0
    const maxOk = Number.isInteger(editEventMaxPlayers.value) && editEventMaxPlayers.value >= 2 && editEventMaxPlayers.value <= 99
    const allowedFormats = formatOptionsForType(event.value?.event_type)
    const formatOk = allowedFormats.includes(editEventFormat.value as never)
    return nameOk && maxOk && formatOk
  })

  function syncEventEditDraftFromEvent() {
    if (!event.value) return
    editEventName.value = event.value.name || ''
    editEventDescription.value = event.value.description || ''
    editEventStartDate.value = event.value.start_date ? isoToDatetimeLocalValue(event.value.start_date) : ''
    editEventFormat.value = event.value.format || '5v5'
    editEventMaxPlayers.value = Number(event.value.max_players)
    editEventRequireDiscord.value = Boolean(event.value.require_discord)
    editEventRequireBattletag.value = Boolean(event.value.require_battletag)
    editEventDiscordAnnounce.value = event.value.discord_announce !== false
  }

  async function saveEventEdit() {
    if (!ensureOwnerAction() || !event.value || updatingEvent.value || !canSaveEventMeta.value) return

    let normalizedStartDate = null
    try {
      normalizedStartDate = normalizeDatetimeLocalInput(editEventStartDate.value, 'event start date')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid event start date')
      return
    }

    updatingEvent.value = true
    try {
      const payloadType = String(event.value.event_type).trim().toUpperCase() === 'TOURNEY' ? 'TOURNEY' : 'PUG'
      const updatedEvent = await eventStore.updateEvent(eventId.value, {
        name: editEventName.value.trim(),
        description: editEventDescription.value.trim(),
        start_date: normalizedStartDate,
        event_type: payloadType,
        format: editEventFormat.value,
        max_players: editEventMaxPlayers.value,
        require_discord: editEventRequireDiscord.value,
        require_battletag: editEventRequireBattletag.value,
        discord_announce: editEventDiscordAnnounce.value,
      })
      event.value = updatedEvent
      syncEventEditDraftFromEvent()
      hydrateSelections()
      setNotice('Event updated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update event')
    } finally {
      updatingEvent.value = false
    }
  }

  async function deleteEvent() {
    if (!ensureOwnerAction() || !event.value || deletingEvent.value) return

    const confirmed = await confirm.ask({
      title: 'Delete event?',
      message: `Delete event "${event.value.name}" and all its matches?`,
      confirmText: 'Delete event',
      tone: 'danger',
    })
    if (!confirmed) return

    deletingEvent.value = true
    try {
      await eventStore.deleteEvent(eventId.value)
      router.push({ name: 'home' })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete event')
    } finally {
      deletingEvent.value = false
    }
  }

  return {
    editEventName, editEventDescription, editEventStartDate, editEventFormat, editEventMaxPlayers,
    editEventRequireDiscord, editEventRequireBattletag, editEventDiscordAnnounce,
    updatingEvent, deletingEvent, canSaveEventMeta,
    syncEventEditDraftFromEvent, saveEventEdit, deleteEvent,
  }
}
