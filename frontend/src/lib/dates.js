const eventStartDateFormatter = new Intl.DateTimeFormat('en-GB', {
  day: '2-digit',
  month: '2-digit',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
})

const shortMonthDayFormatter = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: '2-digit',
})

const mediumDateFormatter = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: '2-digit',
  year: 'numeric',
})

const time24Formatter = new Intl.DateTimeFormat(undefined, {
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
})

function padDatePart(value) {
  return String(value).padStart(2, '0')
}

export function parseDateValue(value) {
  const raw = String(value || '').trim()
  if (!raw) {
    return null
  }

  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) {
    return null
  }

  return parsed
}

export function getDateTimestamp(value) {
  const parsed = parseDateValue(value)
  return parsed ? parsed.getTime() : null
}

export function datetimeLocalToIsoString(value) {
  const raw = String(value || '').trim()
  if (!raw) {
    return null
  }

  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) {
    return null
  }

  return parsed.toISOString()
}

export function normalizeDatetimeLocalInput(value, errorLabel = 'date') {
  const raw = String(value || '').trim()
  if (!raw) {
    return null
  }

  const normalized = datetimeLocalToIsoString(raw)
  if (!normalized) {
    throw new Error(`Invalid ${errorLabel}`)
  }

  return normalized
}

export function isoToDatetimeLocalValue(value) {
  const raw = String(value || '').trim()
  if (!raw) {
    return ''
  }

  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) {
    return ''
  }

  return `${parsed.getFullYear()}-${padDatePart(parsed.getMonth() + 1)}-${padDatePart(parsed.getDate())}T${padDatePart(parsed.getHours())}:${padDatePart(parsed.getMinutes())}`
}

export function formatEventStartDate(value) {
  const parsed = parseDateValue(value)
  if (!parsed) {
    return String(value || '').trim()
  }

  return eventStartDateFormatter.format(parsed)
}

export function formatShortMonthDay(value, fallback = '') {
  const parsed = parseDateValue(value)
  return parsed ? shortMonthDayFormatter.format(parsed) : fallback
}

export function formatMediumDate(value, fallback = '') {
  const parsed = parseDateValue(value)
  return parsed ? mediumDateFormatter.format(parsed) : fallback
}

export function formatTime24(value, fallback = '') {
  const parsed = parseDateValue(value)
  return parsed ? time24Formatter.format(parsed) : fallback
}

export function formatDayMonthYear(value, fallback = '') {
  const parsed = parseDateValue(value)
  if (!parsed) {
    return fallback
  }

  const day = padDatePart(parsed.getDate())
  const month = padDatePart(parsed.getMonth() + 1)
  const year = String(parsed.getFullYear())
  return `${day}/${month}/${year}`
}
