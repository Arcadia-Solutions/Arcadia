<template>
  <PageProgressBar />
  <div id="app-container" v-if="isAppReady">
    <Toast position="top-right" group="tr" :pt="toastPt" @mouse-enter="stopToastTimer" @mouse-leave="startToastTimer" />
    <div class="navbars-container" v-if="isRouteProtected(route.path)">
      <TopBar />
      <MenuBar class="menu-bar" />
      <SearchBars class="search-bars" />
    </div>
    <div id="view-container">
      <router-view></router-view>
    </div>
    <NotificationToasts />
    <FooterBar />
  </div>
</template>

<script setup lang="ts">
import { config } from '@/config'
import { useRouter } from 'vue-router'
import MenuBar from './components/MenuBar.vue'
import TopBar from './components/TopBar.vue'
import SearchBars from './components/SearchBars.vue'
import NotificationToasts from './components/NotificationToasts.vue'
import { Toast, type ToastPassThroughOptions } from 'primevue'
import { useUserStore } from './stores/user'
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import FooterBar from './components/FooterBar.vue'
import PageProgressBar from './components/PageProgressBar.vue'
import { hasLoggedInCookie, isRouteProtected, loginLocation, setLoggedInCookie } from './services/helpers'
import { getMe, getPublicArcadiaSettings } from './services/api-schema'
import { usePublicArcadiaSettingsStore } from './stores/publicArcadiaSettings'
import { connectNotificationStream, disconnectNotificationStream } from './services/notificationStream'
import { startToastTimer, stopToastTimer } from './main'

// enable dark mode by default
document.documentElement.classList.add('dark-theme')

const isAppReady = ref(false)

// Feeds each timed toast's duration to its progress bar. `instance` is the
// ToastMessage.
const toastPt: ToastPassThroughOptions = {
  message: ({ instance }) => ({
    class: { 'toast-timed': instance.message.duration },
    style: { '--toast-duration': `${instance.message.duration}ms` },
  }),
}
const route = useRoute()
const router = useRouter()
const siteName = config.site_name

router.beforeEach(async (to, from, next) => {
  if (from.path === '/login' && isRouteProtected(to.path)) {
    await getAppReady(true)
  }
  if (to.meta.dynamicDocumentTitle) {
    /*
      The View for this route handles it's own
      document title because it's dynamic.
    */
    return next()
  }

  /*
    Favour a custom document title if is defined, otherwise,
    fall back to use the route name at least.
  */
  document.title = `${to.meta.documentTitle || to.name} - ${siteName}`

  return next()
})

/*
  The pages reachable without being signed in can't use the css sheets nor the custom js of
  the public arcadia settings, since both are tied to a signed in user. They are styled by
  these two optional files instead.
*/
const UNAUTHENTICATED_CSS = '/custom_unauth.css'
const UNAUTHENTICATED_JS = '/custom_unauth.js'

const loadUnauthenticatedAssets = () => {
  const head = document.getElementsByTagName('head')[0]

  if (!document.getElementById('arcadia-unauth-css')) {
    const style = document.createElement('link')
    style.id = 'arcadia-unauth-css'
    style.href = UNAUTHENTICATED_CSS
    style.type = 'text/css'
    style.rel = 'stylesheet'
    head.append(style)
  }

  if (!document.getElementById('arcadia-unauth-js')) {
    const script = document.createElement('script')
    script.id = 'arcadia-unauth-js'
    script.src = UNAUTHENTICATED_JS
    head.append(script)
  }
}

router.afterEach(async (to) => {
  if (!isRouteProtected(to.path)) {
    loadUnauthenticatedAssets()
    isAppReady.value = true
  }
})

const getAppReady = async (forceGetUser: boolean = false) => {
  const token = localStorage.getItem('token')

  if (isRouteProtected(route.path) || forceGetUser) {
    if (token) {
      try {
        // refresh user on page reload or fetch user after registration
        const profile = await getMe()
        localStorage.setItem('user', JSON.stringify(profile.user))
        const userStore = useUserStore()
        userStore.setUser(profile.user)
        // refresh public arcadia settings
        const publicArcadiaSettings = await getPublicArcadiaSettings()
        usePublicArcadiaSettingsStore().setSettings(publicArcadiaSettings)
        // load custom css sheet
        const head = document.getElementsByTagName('head')[0]
        const style = document.createElement('link')
        style.href = `${config.api_base_url}/api/css/${profile.user.css_sheet_name}.css`
        style.type = 'text/css'
        style.rel = 'stylesheet'
        head.append(style)
        // inject custom JS configured by arcadia_settings
        if (publicArcadiaSettings.custom_js_code && !document.getElementById('arcadia-custom-js')) {
          const script = document.createElement('script')
          script.id = 'arcadia-custom-js'
          script.textContent = publicArcadiaSettings.custom_js_code
          head.append(script)
        }

        await setLoggedInCookie(true)
        isAppReady.value = true
        connectNotificationStream()
      } catch {
        // token is invalid, redirect to login
        disconnectNotificationStream()
        await setLoggedInCookie(false)
        localStorage.removeItem('token')
        localStorage.removeItem('user')
        router.push(loginLocation())
      }
    } else {
      // no token is present
      // the web server serves the custom front page at / once the cookie is gone. If it was already gone
      // and the app was still served at /, there is no front page to show
      if (config.enable_custom_front_page && (window.location.pathname !== '/' || hasLoggedInCookie())) {
        await setLoggedInCookie(false)
        window.location.href = '/'
      } else {
        router.push(loginLocation())
      }
    }
  }
}

router.isReady().then(async () => {
  getAppReady()
})
</script>

<style>
/* PrimeVue renders the summary even when empty, and the text gap then pushes
   the detail below the icon. */
.p-toast-summary:empty {
  display: none;
}
/* Time left before a timed toast closes. Hovering restarts the timer, so the
   bar resets too. */
.p-toast-message.toast-timed {
  position: relative;
  overflow: hidden;
}
.p-toast-message.toast-timed::after {
  content: '';
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  height: 3px;
  background: currentColor;
  opacity: 0.5;
  transform-origin: left;
  animation: toast-duration var(--toast-duration) linear forwards;
}
.p-toast-message.toast-timed:hover::after {
  animation: none;
}
@keyframes toast-duration {
  to {
    transform: scaleX(0);
  }
}
.navbars-container {
  width: 100%;
}
.menu-bar {
  margin-top: 10px;
}
.search-bars {
  margin-top: 10px;
  margin-bottom: 20px;
}
#footer {
  margin-top: auto;
  width: 100%;
}
</style>
