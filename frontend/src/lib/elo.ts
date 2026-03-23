import type { EventPlayer } from '../types'
import { getRankElo } from './ranks'

export function averagePlayersElo(players: EventPlayer[]): number | null {
  const eloValues = players
    .map((player) => getRankElo(player?.rank))
    .filter((value): value is number => typeof value === 'number')

  if (eloValues.length === 0) {
    return null
  }

  const total = eloValues.reduce((sum, value) => sum + value, 0)
  return Math.round(total / eloValues.length)
}

export function formatAverageElo(value: number | null): string {
  if (value === null) {
    return 'Avg ELO: N/A'
  }

  return `Avg ELO: ${value.toLocaleString()}`
}
