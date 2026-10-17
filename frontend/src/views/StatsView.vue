<template>
  <div id="stats">
    <Tabs :value="currentTab" @update:value="tabChanged">
      <TabList>
        <Tab value="torrents">{{ t('stats.torrents') }}</Tab>
        <Tab value="forum">{{ t('stats.forum') }}</Tab>
        <Tab value="users">{{ t('stats.users') }}</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="torrents" v-if="currentTab === 'torrents'">
          <TorrentStats />
        </TabPanel>
        <TabPanel value="forum" v-if="currentTab === 'forum'">
          <ForumStats />
        </TabPanel>
        <TabPanel value="users" v-if="currentTab === 'users'">
          <UserStats />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref, watch } from 'vue'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
const TorrentStats = defineAsyncComponent(() => import('@/components/stats/TorrentStats.vue'))
const ForumStats = defineAsyncComponent(() => import('@/components/stats/ForumStats.vue'))
const UserStats = defineAsyncComponent(() => import('@/components/stats/UserStats.vue'))
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const router = useRouter()

const currentTab = ref('torrents')

const tabChanged = (tab: string | number) => {
  router.push({ query: { tab } })
}

onMounted(() => {
  if (router.currentRoute.value.query.tab) {
    currentTab.value = router.currentRoute.value.query.tab as string
  }
})

watch(
  () => router.currentRoute.value.query.tab,
  (newTab) => {
    if (newTab) {
      currentTab.value = newTab as string
    }
  },
)
</script>
