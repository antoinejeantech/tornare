import { computed, ref } from 'vue'

export function useEventSignup({ event, eventId, canManageEvent, hasEventAdminAccess, ensureOwnerAction, setError, setNotice, hydrateSelections, eventStore, confirm }) {
  const signupToken = ref('')
  const signupRequests = ref([])
  const loadingSignupRequests = ref(false)
  const reviewingSignupRequests = ref({})
  const rotatingSignupLink = ref(false)
  const updatingSignupVisibility = ref(false)
  const updatingFeaturedEvent = ref(false)
  const endingEvent = ref(false)

  const signupShareUrl = computed(() => {
    if (!signupToken.value) return ''
    if (typeof window === 'undefined') return `/join/${signupToken.value}`
    return `${window.location.origin}/join/${signupToken.value}`
  })

  const pendingSignupRequestCount = computed(() => {
    if (!Array.isArray(signupRequests.value)) return 0
    return signupRequests.value.filter((r) => String(r?.status || '').toLowerCase() === 'pending').length
  })

  function clearSignupData() {
    signupToken.value = ''
    signupRequests.value = []
  }

  async function loadOwnerSignupData() {
    if (!eventId.value || !canManageEvent.value) return

    loadingSignupRequests.value = true
    try {
      const [linkResult, requestsResult] = await Promise.allSettled([
        eventStore.fetchSignupLink(eventId.value),
        eventStore.listSignupRequests(eventId.value),
      ])

      signupToken.value = linkResult.status === 'fulfilled' ? (linkResult.value?.signup_token || '') : ''

      if (requestsResult.status === 'fulfilled') {
        signupRequests.value = Array.isArray(requestsResult.value) ? requestsResult.value : []
      } else {
        signupRequests.value = []
        reviewingSignupRequests.value = {}
      }

      if (linkResult.status === 'rejected' && requestsResult.status === 'rejected') {
        throw new Error('Failed to load signup link and requests')
      }
      if (linkResult.status === 'rejected') {
        setError('Failed to refresh signup link. Please retry before sharing.')
      } else if (requestsResult.status === 'rejected') {
        setError('Failed to load signup requests')
      }
    } catch (err) {
      signupRequests.value = []
      reviewingSignupRequests.value = {}
      setError(err instanceof Error ? err.message : 'Failed to load signup requests')
    } finally {
      loadingSignupRequests.value = false
    }
  }

  async function copySignupLink() {
    if (!signupShareUrl.value) return
    try {
      await navigator.clipboard.writeText(signupShareUrl.value)
      setNotice('Signup link copied')
    } catch {
      setError('Could not copy signup link')
    }
  }

  async function rotateSignupLink() {
    if (!ensureOwnerAction() || !eventId.value || rotatingSignupLink.value) return

    const confirmed = await confirm.ask({
      title: 'Rotate signup link?',
      message: 'The current shared link will stop working immediately.',
      confirmText: 'Rotate link',
      tone: 'warning',
    })
    if (!confirmed) return

    rotatingSignupLink.value = true
    try {
      const response = await eventStore.rotateSignupLink(eventId.value)
      signupToken.value = response.signup_token || ''
      setNotice('Signup link rotated')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to rotate signup link')
    } finally {
      rotatingSignupLink.value = false
    }
  }

  async function setSignupVisibility(enabled) {
    if (!ensureOwnerAction() || !eventId.value || updatingSignupVisibility.value) return

    if (!enabled && Boolean(event.value?.public_signup_enabled)) {
      const confirmed = await confirm.ask({
        title: 'Make registration private?',
        message: 'This hides the public Join button and rotates the signup link token. Existing shared links will stop working.',
        confirmText: 'Make private',
        tone: 'warning',
      })
      if (!confirmed) return
    }

    updatingSignupVisibility.value = true
    try {
      const updatedEvent = await eventStore.setSignupVisibility(eventId.value, enabled)
      event.value = updatedEvent
      hydrateSelections()
      await loadOwnerSignupData()
      setNotice(enabled
        ? 'Public event registration enabled'
        : 'Event registration is now private. Public Join button is hidden and the signup link was rotated.')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update signup visibility')
    } finally {
      updatingSignupVisibility.value = false
    }
  }

  async function setFeaturedEvent(featured) {
    if (!hasEventAdminAccess.value) {
      setError('Only app admins and moderators can change the featured event.')
      return
    }
    if (!eventId.value || updatingFeaturedEvent.value) return

    updatingFeaturedEvent.value = true
    try {
      const updatedEvent = await eventStore.setFeaturedEvent(eventId.value, featured)
      event.value = updatedEvent
      hydrateSelections()
      setNotice(featured ? 'Event is now featured in the spotlight card' : 'Event removed from the spotlight card')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update featured event')
    } finally {
      updatingFeaturedEvent.value = false
    }
  }

  async function setEventEnded(ended) {
    if (!ensureOwnerAction() || !eventId.value || endingEvent.value) return

    if (ended) {
      const confirmed = await confirm.ask({
        title: 'End this event?',
        message: 'The event will be marked as ended and hidden from the public event listings. You can reopen it at any time from the settings.',
        confirmText: 'End event',
        tone: 'warning',
      })
      if (!confirmed) return
    }

    endingEvent.value = true
    try {
      const updatedEvent = await eventStore.setEventEnded(eventId.value, ended)
      event.value = updatedEvent
      hydrateSelections()
      setNotice(ended ? 'Event ended. It is now hidden from public listings.' : 'Event reopened and visible in listings again.')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update event status')
    } finally {
      endingEvent.value = false
    }
  }

  async function acceptSignupRequest(requestId) {
    if (!ensureOwnerAction() || !eventId.value || reviewingSignupRequests.value[requestId]) return
    reviewingSignupRequests.value = { ...reviewingSignupRequests.value, [requestId]: true }
    try {
      const updatedEvent = await eventStore.acceptSignupRequest(eventId.value, requestId)
      event.value = updatedEvent
      hydrateSelections()
      await loadOwnerSignupData()
      setNotice('Signup request accepted')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to accept signup request')
    } finally {
      reviewingSignupRequests.value = { ...reviewingSignupRequests.value, [requestId]: false }
    }
  }

  async function declineSignupRequest(requestId) {
    if (!ensureOwnerAction() || !eventId.value || reviewingSignupRequests.value[requestId]) return
    reviewingSignupRequests.value = { ...reviewingSignupRequests.value, [requestId]: true }
    try {
      await eventStore.declineSignupRequest(eventId.value, requestId)
      await loadOwnerSignupData()
      setNotice('Signup request declined')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to decline signup request')
    } finally {
      reviewingSignupRequests.value = { ...reviewingSignupRequests.value, [requestId]: false }
    }
  }

  return {
    signupToken, signupRequests, loadingSignupRequests, reviewingSignupRequests,
    rotatingSignupLink, updatingSignupVisibility, updatingFeaturedEvent, endingEvent,
    signupShareUrl, pendingSignupRequestCount,
    clearSignupData, loadOwnerSignupData, copySignupLink, rotateSignupLink,
    setSignupVisibility, setFeaturedEvent, setEventEnded, acceptSignupRequest, declineSignupRequest,
  }
}
