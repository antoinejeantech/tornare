export const formatOptionsByType = {
  PUG: ['5v5', '6v6'],
  TOURNEY: ['5v5', '6v6', '1v1'],
}

export function formatOptionsForType(eventType) {
  const type = String(eventType || '').toUpperCase()
  return formatOptionsByType[type] || formatOptionsByType.PUG
}
