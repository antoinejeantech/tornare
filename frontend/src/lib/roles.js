export const rolePriority = {
  Tank: 0,
  DPS: 1,
  Support: 2,
}

export function getRoleIcon(role) {
  if (role === 'Tank') {
    return 'shield'
  }
  if (role === 'Support') {
    return 'medical_services'
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
