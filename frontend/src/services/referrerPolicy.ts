// The site wide policy is no-referrer (index.html), the urls under one of `allowedDomains` get the
// origin instead. Also imported by vite.config.ts at build time, so it must not depend on the app.

// An allowed entry is either a bare host (`domain.com`) or a host followed by a path prefix
// (`domain.com/elysia`), in which case the url path must be that prefix or lie under it.
const matchesAllowedDomain = (url: URL, allowed: string): boolean => {
  const [host, ...path] = allowed.replace(/\/+$/, '').split('/')
  const prefix = path.length > 0 ? `/${path.join('/')}` : ''
  return url.hostname === host && (url.pathname === prefix || url.pathname.startsWith(`${prefix}/`))
}

// the elements accepting a referrerpolicy attribute, with the attribute holding their url
export const URL_ATTRIBUTES: Record<string, string> = { img: 'src', iframe: 'src', a: 'href', area: 'href' }

export const referrerPolicyFor = (url: string | null | undefined, allowedDomains: string[]): ReferrerPolicy | undefined => {
  if (!url || allowedDomains.length === 0) return undefined
  let parsed: URL
  try {
    // relative urls resolve to a host no entry can match, so do the schemes without a host (data:, javascript:...)
    parsed = new URL(url, 'https://relative.invalid')
  } catch {
    return undefined
  }
  return allowedDomains.some((allowed) => matchesAllowedDomain(parsed, allowed)) ? 'strict-origin' : undefined
}
