/**
 * `frontend` section of the config.yml file at the root of the repository, inlined at build
 * time by vite.config.ts. Changing it requires rebuilding the frontend.
 */
export interface ArcadiaFrontendConfig {
  /** Base URL of the API. No trailing slash. */
  api_base_url: string
  /** Name of the site, displayed in the interface. */
  site_name: string
  /** When true, unauthenticated users are served public/home/index.html at /. */
  enable_custom_front_page: boolean
  /** Label displayed on the big input of /apply. */
  apply_input_message: string
  /** Message shown once the user sent their application. */
  application_sent_message: string
  /** URL where the Kiwi IRC webchat is served. */
  irc_webchat_url?: string
  /** WebSocket URL of the IRC server (Ergo). */
  irc_websocket_url?: string
  /** Channels guests auto join from the webchat link of the login page. Empty hides the link. */
  irc_webchat_guest_channels?: string[]
  /** Domains (`domain.com` or `domain.com/path`) sent the site's origin as referrer, every other one gets none. */
  referrer_allowed_domains?: string[]
  /** Public URLs of Elysia instances: their images are displayed as their medium, the original only loads when enlarged. */
  elysia_public_urls?: string[]
}

declare const __ARCADIA_CONFIG__: ArcadiaFrontendConfig

export const config: ArcadiaFrontendConfig = __ARCADIA_CONFIG__

export interface PublicRatingWebsite {
  name: string
  /** Displayed in the lists of title groups. */
  abbreviation: string
  /** The ratings of the links on this domain or one of its subdomains belong to the website. */
  domain: string
  icon: string
}

declare const __PUBLIC_RATING_WEBSITES__: PublicRatingWebsite[]

/** TMDB and the websites the scraper plugins accept links from, from the `scrapers` section of config.yml. */
export const publicRatingWebsites: PublicRatingWebsite[] = __PUBLIC_RATING_WEBSITES__

/** The website of a rated link. A link matching none (e.g. its domain was removed from config.yml) is displayed with its host name. */
export const publicRatingWebsite = (link: string): Pick<PublicRatingWebsite, 'name' | 'abbreviation'> & Partial<PublicRatingWebsite> => {
  const host = new URL(link).hostname
  return publicRatingWebsites.find(({ domain }) => host === domain || host.endsWith(`.${domain}`)) ?? { name: host, abbreviation: host }
}

/** Whether the link is on a known rating website, so a provider can be asked for its rating. */
export const isRatableLink = (link: string) => publicRatingWebsite(link).domain !== undefined
