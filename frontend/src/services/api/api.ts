import { config } from '@/config'
import { showToast, i18n } from '@/main'
import axios from 'axios'
import { type SideEffect } from '../api-schema'
import { formatBp, loginLocation, setLoggedInCookie } from '../helpers'
import { refreshAccessToken } from './tokenRefresh'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { useUserStore } from '@/stores/user'
import { trackRequestEnd, trackRequestStart } from '../progressBar'

const serializeParams = (params: Record<string, unknown>): string => {
  const parts: string[] = []
  for (const [key, value] of Object.entries(params)) {
    if (value === null || value === undefined) continue
    if (Array.isArray(value)) {
      for (const item of value) {
        parts.push(`${encodeURIComponent(key)}[]=${encodeURIComponent(String(item))}`)
      }
    } else {
      parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`)
    }
  }
  return parts.join('&')
}

const api = axios.create({
  baseURL: config.api_base_url,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
  paramsSerializer: serializeParams,
})

api.interceptors.request.use(
  (config) => {
    trackRequestStart()
    const token = localStorage.getItem('token')
    if (token && !config.url?.includes('/login') && !config.url?.includes('/register')) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  },
)

api.interceptors.response.use(
  (response) => {
    trackRequestEnd()
    if (response.headers['content-type'] === 'application/json') {
      const sideEffects: [SideEffect] = response.data.side_effects
      const bonusPoints = sideEffects.find((e) => e.type === 'bonus_points')
      if (bonusPoints) {
        const { t } = i18n.global
        const { bonus_points_decimal_places, bonus_points_alias } = usePublicArcadiaSettingsStore()
        useUserStore().bonus_points += bonusPoints.amount
        showToast('', t('side_effects.bonus_points_earned', [formatBp(bonusPoints.amount, bonus_points_decimal_places), bonus_points_alias]), 'success', 4000)
      }
    }
    return response
  },
  async (error) => {
    trackRequestEnd()
    const originalRequest = error.config
    // We add a custom property `_retry` to the original request config
    // to prevent infinite loops if the refresh token also fails or if
    // a subsequent request with the refreshed token still results in a 401.
    if (error.response && error.response.data === 'jwt token expired' && !originalRequest._retry) {
      originalRequest._retry = true
      try {
        const newToken = await refreshAccessToken()
        originalRequest.headers.Authorization = `Bearer ${newToken}`
        return api(originalRequest) // Return the promise of the re-attempted request
      } catch (refreshError) {
        console.error('Failed to refresh token:', refreshError)
        localStorage.removeItem('token')
        localStorage.removeItem('refreshToken')
        await setLoggedInCookie(false)
        window.location.replace(loginLocation())
        return Promise.reject(refreshError)
      }
    }
    if (error.response && error.response.status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('refreshToken')
      await setLoggedInCookie(false)
      window.location.replace(loginLocation())
      return new Promise(() => {})
    }
    if (error.response && error.response.data && error.response.data.error) {
      showToast('error', error.response.data.error, 'error', 4000)
    } else {
      showToast('error', 'An unexpected error occurred.', 'error', 4000)
    }
    return Promise.reject(error)
  },
)

export default api
