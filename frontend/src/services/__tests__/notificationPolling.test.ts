import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { NotificationCounts } from '@/services/api-schema'
import { getNotificationCounts } from '@/services/api-schema'
import { useNotificationsStore } from '@/stores/notifications'

vi.mock('@/main', () => ({ i18n: { global: { t: (key: string, [count]: number[]) => `${key}:${count}` } } }))
vi.mock('@/config', () => ({ config: { site_name: 'Arcadia' } }))
vi.mock('@/services/api-schema', () => ({ getNotificationCounts: vi.fn() }))

const noCounts: NotificationCounts = {
  announcements: 0,
  conversations: 0,
  forum_sub_category_threads: 0,
  forum_thread_posts: 0,
  title_group_comments: 0,
  title_group_torrents: 0,
  artist_title_groups: 0,
  collages: 0,
  staff_pm_messages: 0,
  torrent_request_comments: 0,
  torrent_deletions: 0,
  announce_errors: 0,
  reseed_requests: 0,
}

const shownNotifications: { body?: string; tag?: string }[] = []
let isHidden = false
let isLockGranted = false
const addedListeners: [string, EventListenerOrEventListenerObject][] = []
const addEventListener = document.addEventListener.bind(document)

function respondWith(...responses: Partial<NotificationCounts>[]) {
  for (const response of responses) vi.mocked(getNotificationCounts).mockResolvedValueOnce({ ...noCounts, ...response })
}

// a fresh module per test, as the polling state lives at module level
async function start() {
  const { startNotificationPolling, POLL_INTERVAL } = await import('@/services/notificationPolling')
  startNotificationPolling()
  await vi.advanceTimersByTimeAsync(0)
  return POLL_INTERVAL
}

beforeEach(() => {
  vi.resetModules()
  vi.useFakeTimers()
  vi.mocked(getNotificationCounts).mockReset()
  setActivePinia(createPinia())
  shownNotifications.length = 0
  isHidden = false
  isLockGranted = false
  localStorage.setItem('arcadia_token', 'token')
  vi.stubGlobal(
    'Notification',
    class {
      static permission = 'granted'
      constructor(_title: string, options: { body?: string; tag?: string }) {
        shownNotifications.push(options)
      }
    },
  )
  document.addEventListener = (type: string, listener: EventListenerOrEventListenerObject) => {
    addedListeners.push([type, listener])
    addEventListener(type, listener)
  }
  Object.defineProperty(document, 'hidden', { configurable: true, get: () => isHidden })
  Object.defineProperty(navigator, 'locks', {
    configurable: true,
    value: { request: (_name: string, callback: () => Promise<void>) => (isLockGranted ? callback() : new Promise(() => {})) },
  })
})

// every test imports a fresh module, whose listeners must not react in the next tests
afterEach(() => {
  for (const [type, listener] of addedListeners.splice(0)) document.removeEventListener(type, listener)
  vi.clearAllTimers()
  vi.useRealTimers()
  vi.unstubAllGlobals()
  localStorage.clear()
})

describe('notification polling', () => {
  it('only notifies the counts that went up since the first fetch', async () => {
    respondWith({ conversations: 2, collages: 1 }, { conversations: 3, collages: 1, announce_errors: 1 })
    const pollInterval = await start()
    expect(shownNotifications).toEqual([])
    expect(useNotificationsStore().conversations).toBe(2)

    await vi.advanceTimersByTimeAsync(pollInterval)
    expect(shownNotifications).toEqual([
      { body: 'user.conversations:3', tag: 'conversations', icon: '/favicon.ico' },
      { body: 'user.announce_errors:1', tag: 'announce_errors', icon: '/favicon.ico' },
    ])
    expect(useNotificationsStore().$state).toEqual({ ...noCounts, conversations: 3, collages: 1, announce_errors: 1 })
  })

  it.each([
    [false, 1],
    [true, 2],
  ])('polls a hidden tab in the background only when it is the leader (leader: %s)', async (isLeader, expectedCalls) => {
    isHidden = true
    isLockGranted = isLeader
    respondWith({}, {})
    const pollInterval = await start()
    await vi.advanceTimersByTimeAsync(pollInterval)
    expect(getNotificationCounts).toHaveBeenCalledTimes(expectedCalls)
  })

  it('polls as soon as the tab becomes visible again', async () => {
    isHidden = true
    respondWith({}, { forum_thread_posts: 1 })
    await start()

    isHidden = false
    document.dispatchEvent(new Event('visibilitychange'))
    await vi.advanceTimersByTimeAsync(0)
    expect(useNotificationsStore().forum_thread_posts).toBe(1)
    expect(shownNotifications).toHaveLength(1)
  })

  it('does not poll once logged out', async () => {
    localStorage.removeItem('arcadia_token')
    await start()
    expect(getNotificationCounts).not.toHaveBeenCalled()
  })
})
