import { config } from '@/config'
import { useNotificationsStore } from '@/stores/notifications'
import { type NotificationCounts, getNotificationCounts } from '@/services/api-schema'
import { getValidToken } from '@/services/api/tokenRefresh'
import { i18n } from '@/main'

const MINIMUM_RECONNECT_DELAY = 5000
const MAXIMUM_RECONNECT_DELAY = 60000

const eventTypeToCountKey: Record<string, keyof NotificationCounts> = {
  forum_sub_category_thread: 'forum_sub_category_threads',
  forum_thread_post: 'forum_thread_posts',
  title_group_comment: 'title_group_comments',
  title_group_torrent: 'title_group_torrents',
  reseed_request: 'reseed_requests',
  artist_title_group: 'artist_title_groups',
  torrent_request_comment: 'torrent_request_comments',
  staff_pm_message: 'staff_pm_messages',
  conversation: 'conversations',
}

let eventSource: EventSource | null = null
let channel: BroadcastChannel | null = null
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null
let reconnectDelay = MINIMUM_RECONNECT_DELAY
let lockAbort: AbortController | null = null
let releaseLock: (() => void) | null = null
let isLeaderTab = false

function applyCounts(counts: NotificationCounts) {
  const notificationsStore = useNotificationsStore()
  for (const key of Object.keys(counts) as (keyof NotificationCounts)[]) {
    notificationsStore[key] = counts[key]
  }
}

function refreshNotificationCounts() {
  getNotificationCounts().then((counts) => {
    applyCounts(counts)
    channel?.postMessage({ type: 'counts', counts })
  })
}

function handleNotificationEvent(eventType: string) {
  if (eventType === 'refresh') {
    refreshNotificationCounts()
    return
  }

  const countKey = eventTypeToCountKey[eventType]
  if (!countKey) return

  const notificationsStore = useNotificationsStore()
  const newCount = notificationsStore[countKey] + 1
  notificationsStore[countKey] = newCount

  if (Notification.permission === 'granted') {
    const body = i18n.global.t(`user.${countKey}`, [newCount])
    new Notification(config.site_name, { body, tag: countKey, icon: '/favicon.ico' })
  }
}

function scheduleReconnect() {
  if (reconnectTimeout) return
  reconnectTimeout = setTimeout(openEventSource, reconnectDelay)
  reconnectDelay = Math.min(reconnectDelay * 2, MAXIMUM_RECONNECT_DELAY)
}

function closeEventSource() {
  eventSource?.close()
  eventSource = null
}

function openEventSource() {
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout)
    reconnectTimeout = null
  }
  closeEventSource()

  getValidToken().then((token) => {
    // no token means the refresh call failed (offline most of the time), retry later
    if (!token) {
      scheduleReconnect()
      return
    }

    const source = new EventSource(`${config.api_base_url}/api/notifications/stream?token=${encodeURIComponent(token)}`)
    eventSource = source

    source.onopen = () => {
      reconnectDelay = MINIMUM_RECONNECT_DELAY
      // events emitted while the connection was down are lost, so resynchronize
      refreshNotificationCounts()
    }

    source.onmessage = (event) => {
      handleNotificationEvent(event.data)
      channel?.postMessage({ type: 'event', eventType: event.data })
    }

    source.onerror = () => {
      if (eventSource === source) closeEventSource()
      else source.close()
      scheduleReconnect()
    }
  })
}

// browsers freeze background tabs (and their timers) after a while, which kills
// the connection without ever running the reconnection logic. every time the tab
// becomes visible again the connection is therefore verified
function verifyConnection() {
  if (document.hidden) return

  if (!isLeaderTab) {
    refreshNotificationCounts()
    return
  }

  if (eventSource?.readyState === EventSource.OPEN) return
  reconnectDelay = MINIMUM_RECONNECT_DELAY
  openEventSource()
}

export function connectNotificationStream() {
  disconnectNotificationStream()

  if (!localStorage.getItem('arcadia_token')) return

  if ('Notification' in window && Notification.permission === 'default') {
    Notification.requestPermission()
  }

  channel = new BroadcastChannel('notification-stream')
  channel.onmessage = (event) => {
    if (event.data.type === 'counts') applyCounts(event.data.counts)
    else handleNotificationEvent(event.data.eventType)
  }

  document.addEventListener('visibilitychange', verifyConnection)
  window.addEventListener('online', verifyConnection)

  refreshNotificationCounts()

  // Only one tab holds the lock and maintains the SSE connection.
  // Other tabs receive events via BroadcastChannel.
  // When the leader tab closes, the lock is automatically released
  // and the next tab in the queue becomes the leader.
  lockAbort = new AbortController()
  navigator.locks
    .request('notification-stream', { signal: lockAbort.signal }, () => {
      isLeaderTab = true
      openEventSource()
      return new Promise<void>((resolve) => {
        releaseLock = resolve
      })
    })
    .catch(() => {})
}

export function disconnectNotificationStream() {
  document.removeEventListener('visibilitychange', verifyConnection)
  window.removeEventListener('online', verifyConnection)

  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout)
    reconnectTimeout = null
  }
  reconnectDelay = MINIMUM_RECONNECT_DELAY

  lockAbort?.abort()
  lockAbort = null
  releaseLock?.()
  releaseLock = null
  isLeaderTab = false

  closeEventSource()
  channel?.close()
  channel = null
}
