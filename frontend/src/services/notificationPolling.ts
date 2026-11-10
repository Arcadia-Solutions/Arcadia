import { config } from '@/config'
import { i18n } from '@/main'
import { type NotificationCounts, getNotificationCounts } from '@/services/api-schema'
import { useNotificationsStore } from '@/stores/notifications'

export const POLL_INTERVAL = 60_000

let pollInterval: ReturnType<typeof setInterval> | undefined
let isLeaderTab = false
// the first fetch only loads the counts, the notifications are for what arrives afterwards
let hasFetchedCounts = false

export function refreshNotificationCounts() {
  if (!localStorage.getItem('arcadia_token')) return
  getNotificationCounts().then((counts) => {
    const notificationsStore = useNotificationsStore()
    if (hasFetchedCounts && window.Notification?.permission === 'granted') {
      for (const key of Object.keys(counts) as (keyof NotificationCounts)[]) {
        if (counts[key] > notificationsStore[key]) {
          // the tag makes several tabs reporting the same increase replace each other's notification
          new Notification(config.site_name, { body: i18n.global.t(`user.${key}`, [counts[key]]), tag: key, icon: '/favicon.ico' })
        }
      }
    }
    notificationsStore.$patch(counts)
    hasFetchedCounts = true
  })
}

// Only the leader tab polls in the background, but every visible tab polls so that its counts stay
// fresh even when the leader tab was frozen by the browser.
export function startNotificationPolling() {
  hasFetchedCounts = false
  refreshNotificationCounts()
  if (pollInterval) return

  if (window.Notification?.permission === 'default') Notification.requestPermission()
  navigator.locks.request('arcadia-notification-polling', () => {
    isLeaderTab = true
    // never resolves, so the lock is held until the tab closes
    return new Promise(() => {})
  })
  pollInterval = setInterval(() => {
    if (isLeaderTab || !document.hidden) refreshNotificationCounts()
  }, POLL_INTERVAL)
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) refreshNotificationCounts()
  })
}
