import { refreshToken as callRefreshToken } from '@/services/api-schema'

export function isTokenExpired(token: string): boolean {
  try {
    const payload = JSON.parse(atob(token.split('.')[1].replace(/-/g, '+').replace(/_/g, '/')))
    if (typeof payload.exp !== 'number') return false
    return payload.exp * 1000 <= Date.now()
  } catch {
    return true
  }
}

// a refresh token can only be redeemed once, so concurrent callers share the same refresh
let pendingRefresh: Promise<string> | null = null

export async function refreshAccessToken(): Promise<string> {
  const refresh = localStorage.getItem('refreshToken')
  if (!refresh) return Promise.reject(new Error('no refresh token'))
  pendingRefresh ??= callRefreshToken({ refresh_token: refresh })
    .then((tokens) => {
      localStorage.setItem('token', tokens.token)
      localStorage.setItem('refreshToken', tokens.refresh_token)
      return tokens.token
    })
    .finally(() => {
      pendingRefresh = null
    })
  return pendingRefresh
}

export async function getValidToken(): Promise<string | null> {
  const token = localStorage.getItem('token')
  if (!token) return Promise.resolve(null)
  if (!isTokenExpired(token)) return Promise.resolve(token)
  return refreshAccessToken().catch(() => null)
}
