import type { ComputedRef, Ref } from 'vue'
import type { Router } from 'vue-router'
import type { Event } from '../../types'
import type { useEventStore } from '../../stores/event'
import type { useMatchStore } from '../../stores/match'
import type { useConfirm } from '../confirm'

export interface SharedEventCtx {
  event: Ref<Event | null>
  eventId: ComputedRef<string>
  canManageEvent: ComputedRef<boolean>
  ensureOwnerAction: () => boolean
  setError: (message: string) => void
  setNotice: (message: string) => void
  hydrateSelections: () => void
  eventStore: ReturnType<typeof useEventStore>
  confirm: ReturnType<typeof useConfirm>
}

export interface EventSettingsCtx extends SharedEventCtx {
  router: Router
}

export interface EventSignupCtx extends SharedEventCtx {
  hasEventAdminAccess: ComputedRef<boolean>
}

export interface EventPlayersCtx extends SharedEventCtx {
  eventIsFull: ComputedRef<boolean>
  clearLastBalancedFingerprint: () => void
}

export interface EventMatchesCtx extends SharedEventCtx {
  isTourneyEvent: ComputedRef<boolean>
  matchupSelections: Ref<Record<string, { teamAId: string; teamBId: string }>>
  matchStore: ReturnType<typeof useMatchStore>
}
