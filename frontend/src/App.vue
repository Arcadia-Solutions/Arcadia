<template>
  <PageProgressBar />
  <div id="app-container" v-if="isAppReady">
    <Toast position="top-right" group="tr" />
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
import { Toast } from 'primevue'
import { useUserStore } from './stores/user'
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import FooterBar from './components/FooterBar.vue'
import PageProgressBar from './components/PageProgressBar.vue'
import { isRouteProtected } from './services/helpers'
import { getMe, getPublicArcadiaSettings } from './services/api-schema'
import { usePublicArcadiaSettingsStore } from './stores/publicArcadiaSettings'
import { connectNotificationStream, disconnectNotificationStream } from './services/notificationStream'

// enable dark mode by default
document.documentElement.classList.add('dark-theme')

const isAppReady = ref(false)
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
        const [profile, publicArcadiaSettings] = await Promise.all([getMe(), getPublicArcadiaSettings()])
        localStorage.setItem('user', JSON.stringify(profile.user))
        const userStore = useUserStore()
        userStore.setUser(profile.user)
        // refresh public arcadia settings
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

        isAppReady.value = true
        connectNotificationStream()
      } catch {
        // token is invalid, redirect to login
        disconnectNotificationStream()
        localStorage.removeItem('token')
        localStorage.removeItem('user')
        router.push('/login')
      }
    } else {
      // no token is present
      if (config.enable_custom_front_page) {
        window.location.href = '/home/index.html'
      } else {
        router.push('/login')
      }
    }
  }
}

router.isReady().then(async () => {
  getAppReady()
})
</script>

<style>
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
