<template>
  <Drawer v-model:visible="visible" :header="config.site_name" position="right">
    <PanelMenu
      :model="[
        { label: 'Home', icon: 'pi pi-home', url: '/' },
        { label: 'Upload', icon: 'pi pi-upload', url: '/upload' },
        { label: 'Toggle dark mode', icon: 'pi pi-moon', command: toggleDarkMode },
        {
          label: 'Profile',
          items: [
            { label: 'View profile', icon: 'pi pi-user', url: `/user/${user.id}` },
            { label: t('conversation.conversation', 2), icon: 'pi pi-envelope', url: '/conversations' },
            { label: t('general.bookmarks'), icon: 'pi pi-bookmark', url: `/torrents?user_id_bookmarks=${user.id}` },
            { label: t('user.logout'), icon: 'pi pi-sign-out', command: handleLogout },
          ],
        },
      ]"
    />
    <Divider />
    <PanelMenu
      :model="[
        { label: 'Torrents', url: '/torrents' },
        { label: 'Artists', url: '/artists' },
        ...(user.permissions.includes('search_users') ? [{ label: 'Users', url: '/users' }] : []),
        { label: 'Collages', url: '' },
        { label: 'Requests', url: '/torrent-requests' },
        { label: 'Forum', url: '/forum' },
        { label: 'IRC', url: '' },
        { label: 'Top', url: '' },
        { label: 'Rules', url: '/wiki/article/1' },
        { label: 'Wiki', url: '/wiki/article/1' },
      ]"
    >
      <template #item="{ item }">
        <RouterLink :to="item.url ?? ''">
          <i :class="item.icon"></i>
          <span>{{ item.label }}</span>
        </RouterLink>
      </template>
    </PanelMenu>
  </Drawer>
  <Button id="mobileMenuTrigger" icon="pi pi-bars" severity="contrast" size="small" @click="onOpenMenu" />
</template>

<script setup lang="ts">
import Drawer from 'primevue/drawer'
import { Button, Divider, PanelMenu } from 'primevue'
import router from '@/router'
import { setLoggedInCookie } from '@/services/helpers'
import { ref } from 'vue'
import { useUserStore } from '@/stores/user'
import { logout } from '@/services/api-schema'
import { useI18n } from 'vue-i18n'
import { config } from '@/config'

const { t } = useI18n()
const user = useUserStore()
const visible = ref(false)

const onOpenMenu = () => {
  visible.value = true
}

const handleLogout = async () => {
  try {
    await logout()
  } catch {
    // Ignore errors (token may already be expired)
  }
  localStorage.removeItem('token')
  localStorage.removeItem('refreshToken')
  localStorage.removeItem('user')
  await setLoggedInCookie(false)
  user.removeUser()
  router.push('/login')
}

const toggleDarkMode = () => {
  document.documentElement.classList.toggle('dark-theme')
}
</script>
