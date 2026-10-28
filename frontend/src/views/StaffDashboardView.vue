<template>
  <div id="staff-dashboard">
    <Tabs :value="currentTab" scrollable @update:value="tabChanged">
      <TabList>
        <Tab value="userApplications" v-if="userStore.permissions.includes('get_user_application')">{{ t('staff.user_application.user_application', 2) }}</Tab>
        <Tab value="staffPms" v-if="userStore.permissions.includes('read_staff_pm')">{{ t('staff_pm.staff_pm', 2) }}</Tab>
        <Tab value="cssSheets" v-if="userStore.permissions.includes('edit_css_sheet') || userStore.permissions.includes('create_css_sheet')">
          {{ t('css_sheet.css_sheet', 2) }}
        </Tab>
        <Tab value="arcadiaSettings" v-if="userStore.permissions.includes('edit_arcadia_settings')">{{ t('arcadia_settings.arcadia_settings') }}</Tab>
        <Tab value="emojiManagement" v-if="userStore.permissions.includes('edit_arcadia_settings')">{{ t('emoji_management.emoji_management') }}</Tab>
        <Tab value="userClasses" v-if="userStore.permissions.includes('edit_user_class') || userStore.permissions.includes('create_user_class')">
          {{ t('user_class.user_class', 2) }}
        </Tab>
        <Tab
          value="donations"
          v-if="
            userStore.permissions.includes('search_donation') ||
            userStore.permissions.includes('create_donation') ||
            userStore.permissions.includes('edit_donation') ||
            userStore.permissions.includes('delete_donation')
          "
        >
          {{ t('donation.donation', 2) }}
        </Tab>
        <Tab value="unauthorizedAccess" v-if="userStore.permissions.includes('search_unauthorized_access')">
          {{ t('unauthorized_access.unauthorized_access', 2) }}
        </Tab>
        <Tab value="userEditLogs" v-if="userStore.permissions.includes('search_user_edit_change_logs')">
          {{ t('user_edit_log.user_edit_log', 2) }}
        </Tab>
        <Tab value="appliedTags" v-if="userStore.permissions.includes('manage_title_group_tags')">
          {{ t('applied_tag.applied_tag', 2) }}
        </Tab>
        <Tab value="allConversations" v-if="userStore.permissions.includes('read_all_conversations')">
          {{ t('conversation.all_conversations') }}
        </Tab>
        <Tab value="userBadges" v-if="canManageUserBadges">{{ t('user_badge.user_badge', 2) }}</Tab>
        <Tab value="siteHighlights" v-if="userStore.permissions.includes('manage_site_highlights')">{{ t('site_highlights.site_highlights', 2) }}</Tab>
        <Tab value="adventCalendars" v-if="userStore.permissions.includes('manage_advent_calendars')">{{ t('advent_calendar.advent_calendar', 2) }}</Tab>
        <Tab value="maintenanceTools" v-if="userStore.permissions.includes('use_maintenance_tools')">{{ t('maintenance_tools.maintenance_tools') }}</Tab>
      </TabList>
      <!-- tabs are loaded only if they are focused -->
      <TabPanels>
        <TabPanel value="userApplications" v-if="userStore.permissions.includes('get_user_application') && currentTab === 'userApplications'">
          <UserApplications />
        </TabPanel>
        <TabPanel value="staffPms" v-if="userStore.permissions.includes('read_staff_pm') && currentTab === 'staffPms'">
          <StaffPmsTable />
        </TabPanel>
        <TabPanel
          value="cssSheets"
          v-if="(userStore.permissions.includes('edit_css_sheet') || userStore.permissions.includes('create_css_sheet')) && currentTab === 'cssSheets'"
        >
          <CssSheetList showStaffActions />
        </TabPanel>
        <TabPanel value="arcadiaSettings" v-if="userStore.permissions.includes('edit_arcadia_settings') && currentTab === 'arcadiaSettings'">
          <ArcadiaSettings />
        </TabPanel>
        <TabPanel value="emojiManagement" v-if="userStore.permissions.includes('edit_arcadia_settings') && currentTab === 'emojiManagement'">
          <EmojisManager />
        </TabPanel>
        <TabPanel
          value="userClasses"
          v-if="(userStore.permissions.includes('edit_user_class') || userStore.permissions.includes('create_user_class')) && currentTab === 'userClasses'"
        >
          <UserClassesTable />
        </TabPanel>
        <TabPanel
          value="donations"
          v-if="
            (userStore.permissions.includes('search_donation') ||
              userStore.permissions.includes('create_donation') ||
              userStore.permissions.includes('edit_donation') ||
              userStore.permissions.includes('delete_donation')) &&
            currentTab === 'donations'
          "
        >
          <DonationsTable />
        </TabPanel>
        <TabPanel value="unauthorizedAccess" v-if="userStore.permissions.includes('search_unauthorized_access') && currentTab === 'unauthorizedAccess'">
          <UnauthorizedAccessTable />
        </TabPanel>
        <TabPanel value="userEditLogs" v-if="userStore.permissions.includes('search_user_edit_change_logs') && currentTab === 'userEditLogs'">
          <UserEditLogsTable />
        </TabPanel>
        <TabPanel value="appliedTags" v-if="userStore.permissions.includes('manage_title_group_tags') && currentTab === 'appliedTags'">
          <TitleGroupAppliedTagsTable />
        </TabPanel>
        <TabPanel value="allConversations" v-if="userStore.permissions.includes('read_all_conversations') && currentTab === 'allConversations'">
          <AllConversationsTable />
        </TabPanel>
        <TabPanel value="userBadges" v-if="canManageUserBadges && currentTab === 'userBadges'">
          <UserBadgesTable />
        </TabPanel>
        <TabPanel value="siteHighlights" v-if="userStore.permissions.includes('manage_site_highlights') && currentTab === 'siteHighlights'">
          <SiteHighlightsManager />
        </TabPanel>
        <TabPanel value="adventCalendars" v-if="userStore.permissions.includes('manage_advent_calendars') && currentTab === 'adventCalendars'">
          <AdventCalendarsManager />
        </TabPanel>
        <TabPanel value="maintenanceTools" v-if="userStore.permissions.includes('use_maintenance_tools') && currentTab === 'maintenanceTools'">
          <MaintenanceTools />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import UserApplications from '@/components/staff/UserApplications.vue'
import { useI18n } from 'vue-i18n'
import StaffPmsTable from '@/components/staff_pm/StaffPmsTable.vue'
import CssSheetList from '@/components/CssSheetList.vue'
import ArcadiaSettings from '@/components/staff/ArcadiaSettings.vue'
import EmojisManager from '@/components/staff/EmojisManager.vue'
import UserClassesTable from '@/components/staff/UserClassesTable.vue'
import DonationsTable from '@/components/staff/DonationsTable.vue'
import UnauthorizedAccessTable from '@/components/staff/UnauthorizedAccessTable.vue'
import UserEditLogsTable from '@/components/staff/UserEditLogsTable.vue'
import AllConversationsTable from '@/components/staff/AllConversationsTable.vue'
import TitleGroupAppliedTagsTable from '@/components/staff/TitleGroupAppliedTagsTable.vue'
import UserBadgesTable from '@/components/staff/UserBadgesTable.vue'
import SiteHighlightsManager from '@/components/staff/SiteHighlightsManager.vue'
import MaintenanceTools from '@/components/staff/MaintenanceTools.vue'
import AdventCalendarsManager from '@/components/staff/AdventCalendarsManager.vue'
import { type UserPermission } from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { useRouter } from 'vue-router'
import { computed, onMounted, ref, watch } from 'vue'

const { t } = useI18n()
const userStore = useUserStore()
const router = useRouter()

const currentTab = ref('')

const userBadgeManagementPermissions: UserPermission[] = [
  'create_user_badge',
  'edit_user_badge',
  'delete_user_badge',
  'create_user_badge_category',
  'edit_user_badge_category',
  'delete_user_badge_category',
]
const canManageUserBadges = computed(() => userBadgeManagementPermissions.some((p) => userStore.permissions.includes(p)))

const tabChanged = (tab: string | number) => {
  router.push({ query: { tab } })
}

onMounted(() => {
  if (router.currentRoute.value.query.tab) {
    currentTab.value = router.currentRoute.value.query.tab as string
  } else {
    currentTab.value = 'userApplications'
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

<style scoped>
#staff-dashboard {
  /* The tabs themselves have transparent backgrounds and take their colour
     from the tablist. When scrollable, the tablist box stays at the viewport
     width while the tabs overflow past it, so the scrolled-off edges have no
     background and the page shows through. Force the tablist to span all of
     its tabs so it stays opaque end to end. Centering still applies when the
     tabs fit; once they overflow there is no free space, so they pack and the
     viewport scrolls. */
  :deep(.p-tablist-tab-list) {
    min-width: max-content;
    justify-content: center;
  }
}
</style>
