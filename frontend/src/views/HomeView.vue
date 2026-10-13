<template>
  <div id="home-page">
    <div class="main">
      <LatestTorrents
        :titleGroups="latestUploads ?? []"
        :loading="!latestUploads"
        containerTitleLink="/torrents"
        :containerTitle="t('torrent.latest_uploads')"
        showUploader
        priority
      />
      <KiwiIrcChat v-if="userStore.id && publicSettings.irc_enabled && publicSettings.irc_webchat_enabled" containerTitleLink="/wiki/article/3" />
      <Tabs value="0" size="small" style="margin: 10px 0">
        <TabList>
          <Tab value="0">{{ t('forum.latest_forum_post', 2) }}</Tab>
          <Tab value="1">{{ t('title_group.latest_title_group_comment', 2) }}</Tab>
        </TabList>
        <TabPanels>
          <TabPanel value="0">
            <ForumSearchResults :search-results="latestPostsInThreads" />
          </TabPanel>
          <TabPanel value="1">
            <TitleGroupCommentSearchResults :search-results="latestTitleGroupComments" />
          </TabPanel>
        </TabPanels>
      </Tabs>
      <div class="announcements">
        <AnnouncementComponent v-for="announcement in recentAnnouncements" :key="announcement.id" :announcement class="announcement" />
      </div>
    </div>
    <div class="sidebar">
      <ContentContainer
        v-for="highlight in siteHighlights"
        :key="highlight.id"
        :containerTitle="highlight.alias"
        :containerTitleLink="`/forum/thread/${highlight.forum_thread_id}`"
        style="margin-bottom: 10px"
      >
        <RouterLink :to="siteHighlightLink(highlight.item_type, highlight.item_id)">
          <img v-if="highlight.item_image" class="site-highlight-image" :src="highlight.item_image" />
        </RouterLink>
      </ContentContainer>
      <ContentContainer :containerTitle="t('statistics.community_stats')" v-if="stats">
        <div>
          {{ t('statistics.enabled_users') }}: {{ formatNumber(stats.enabled_users) }}
          <RouterLink to="/stats?tab=users" v-if="userStore.permissions.includes('view_stats_details')"><i class="pi pi-chart-bar" /></RouterLink>
        </div>
        <div id="stat-active-today">
          {{ t('statistics.users_active_today') }}: {{ formatNumber(stats.users_active_today) }} ({{
            ((stats.users_active_today / stats.enabled_users) * 100).toFixed(2)
          }}%)
        </div>
        <div id="stat-active-week">
          {{ t('statistics.users_active_this_week') }}: {{ formatNumber(stats.users_active_this_week) }} ({{
            ((stats.users_active_this_week / stats.enabled_users) * 100).toFixed(2)
          }}%)
        </div>
        <div id="stat-active-month">
          {{ t('statistics.users_active_this_month') }}: {{ formatNumber(stats.users_active_this_month) }} ({{
            ((stats.users_active_this_month / stats.enabled_users) * 100).toFixed(2)
          }}%)
        </div>
        <div>
          {{ t('community.forum_threads') }}: {{ formatNumber(stats.forum_threads) }}
          <RouterLink to="/stats?tab=forum" v-if="userStore.permissions.includes('view_stats_details')"><i class="pi pi-chart-bar" /></RouterLink>
        </div>
        <div>{{ t('community.forum_posts') }}: {{ formatNumber(stats.forum_posts) }}</div>
        <div>{{ t('community.title_group_comments') }}: {{ formatNumber(stats.title_group_comments) }}</div>
        <div>{{ t('community.request_comments') }}: {{ formatNumber(stats.torrent_request_comments) }}</div>
      </ContentContainer>
      <ContentContainer :containerTitle="t('statistics.library_stats')" v-if="stats" style="margin-top: 10px">
        <div>
          {{ t('statistics.torrents') }}: {{ formatNumber(stats.torrents) }}
          <RouterLink to="/stats?tab=torrents" v-if="userStore.permissions.includes('view_stats_details')"><i class="pi pi-chart-bar" /></RouterLink>
        </div>
        <div id="stat-requests-filled">
          {{ t('torrent.requests') }}: {{ formatNumber(stats.torrent_requests) }} ({{
            ((stats.torrent_requests_filled / stats.torrent_requests) * 100).toFixed(2)
          }}% {{ t('torrent_request.filled') }})
        </div>
        <div>{{ t('torrent.peers') }}: {{ formatNumber(stats.peers) }}</div>
        <div>{{ t('torrent.seeders') }}: {{ formatNumber(stats.seeders) }}</div>
        <div>{{ t('torrent.leecher', 2) }}: {{ formatNumber(stats.leechers) }}</div>
        <div>{{ t('user.snatches', 2) }}: {{ formatNumber(stats.snatches) }}</div>
        <div>{{ t('statistics.titles') }}: {{ formatNumber(stats.titles) }}</div>
        <div>{{ t('statistics.artists') }}: {{ formatNumber(stats.artists) }}</div>
        <div>{{ t('series.series', 2) }}: {{ formatNumber(stats.series) }}</div>
        <div>{{ t('collage.collage', 2) }}: {{ formatNumber(stats.collages) }}</div>
      </ContentContainer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AnnouncementComponent from '@/components/forum/AnnouncementComponent.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import { Tab, TabList, TabPanel, TabPanels, Tabs } from 'primevue'
import LatestTorrents from '@/components/torrent/LatestTorrents.vue'
import ForumSearchResults from '@/components/forum/ForumSearchResults.vue'
import TitleGroupCommentSearchResults from '@/components/title_group/TitleGroupCommentSearchResults.vue'
import KiwiIrcChat from '@/components/irc/KiwiIrcChat.vue'
import { useUserStore } from '@/stores/user'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import {
  getHomeData,
  type ForumPostAndThreadName,
  type ForumSearchResult,
  type HomeStats,
  type SiteHighlightForHome,
  type TitleGroupCommentSearchResult,
  type TitleGroupLite,
} from '@/services/api-schema'
import { siteHighlightLink, formatNumber } from '@/services/helpers'

const userStore = useUserStore()
const publicSettings = usePublicArcadiaSettingsStore()

const { t } = useI18n()

const recentAnnouncements = ref<ForumPostAndThreadName[]>()
const stats = ref<HomeStats>()
const latestUploads = ref<TitleGroupLite[]>()
const latestPostsInThreads = ref<ForumSearchResult[]>([])
const latestTitleGroupComments = ref<TitleGroupCommentSearchResult[]>([])
const siteHighlights = ref<SiteHighlightForHome[]>([])

const fetchHome = async () => {
  getHomeData().then((data) => {
    recentAnnouncements.value = data.recent_announcements
    stats.value = data.stats
    latestUploads.value = data.latest_uploads
    latestPostsInThreads.value = data.latest_posts_in_threads
    latestTitleGroupComments.value = data.latest_title_group_comments
    siteHighlights.value = data.site_highlights ?? []
  })
}

onMounted(() => {
  fetchHome()
})
</script>

<style scoped>
#home-page {
  display: flex;
  justify-content: space-between;
}
.main {
  width: 77%;
}
.sidebar {
  width: 22%;
}
.site-highlight-image {
  display: block;
  width: 100%;
  height: auto;
  /* Site highlights are TMDB posters/profiles (2:3); reserve the box so the remote image
     loading does not grow the sidebar and shift the content below it. */
  aspect-ratio: 2 / 3;
  object-fit: cover;
  border-radius: 7px;
}
.announcement {
  margin-top: 10px;
  margin-bottom: 10px;
}
</style>
