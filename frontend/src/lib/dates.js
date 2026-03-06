const eventStartDateFormatter = new Intl.DateTimeFormat('en-GB', {
  day: '2-digit',
  month: '2-digit',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
})

export function formatEventStartDate(value) {
  const raw = String(value || '').trim()
  if (!raw) {
    return ''
  }

  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) {
    return raw
  }

  return eventStartDateFormatter.format(parsed)
}
