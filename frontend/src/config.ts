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
}

declare const __ARCADIA_CONFIG__: ArcadiaFrontendConfig

export const config: ArcadiaFrontendConfig = __ARCADIA_CONFIG__
