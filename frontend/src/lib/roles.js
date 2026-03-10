export const rolePriority = {
  Tank: 0,
  DPS: 1,
  Support: 2,
}

export function getRoleIcon(role) {
  const normalizedRole = String(role || '').trim().toLowerCase()

  if (normalizedRole === 'tank') {
    return 'shield'
  }
  if (normalizedRole === 'dps') {
    return 'swords'
  }
  if (normalizedRole === 'flex') {
    return 'sync'
  }
  if (normalizedRole === 'support') {
    return 'health_cross'
  }
  return 'swords'
}

export function sortPlayersByRoleThenName(players) {
  return [...players].sort((a, b) => {
    const aPriority = rolePriority[a?.role] ?? 99
    const bPriority = rolePriority[b?.role] ?? 99
    if (aPriority !== bPriority) {
      return aPriority - bPriority
    }

    return String(a?.name || '').localeCompare(String(b?.name || ''))
  })
}
