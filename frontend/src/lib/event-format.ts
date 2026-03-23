import type { EventFormat, EventType } from '../types'

export const formatOptionsByType: Record<EventType, EventFormat[]> = {
  PUG: ['5v5', '6v6'],
  TOURNEY: ['5v5', '6v6', '1v1'],
}

export function formatOptionsForType(eventType: string | undefined | null): EventFormat[] {
  const type = String(eventType || '').toUpperCase()
  return (formatOptionsByType as Record<string, EventFormat[]>)[type] ?? formatOptionsByType.PUG
}
