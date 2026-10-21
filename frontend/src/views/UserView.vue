<template>
  <div id="user-view" v-if="user">
    <div class="main">
      <div class="top-bar">
        <UsernameEnriched :user displayAllInfo noLink />
        <div class="actions">
          <RouterLink :to="`/conversation/new?receiverId=${user.id}&username=${user.username}`" class="no-color" v-if="userStore.id !== user.id">
            <i v-tooltip.top="t('user.message_user', [user.username])" class="pi pi-envelope" />
          </RouterLink>
          <i
            v-if="userStore.id !== user.id"
            v-tooltip.top="t('user.gift.send_gift', [user.username])"
            class="cursor-pointer pi pi-gift"
            @click="sendGiftDialogVisible = true"
          />
          <template v-if="userStore.permissions.includes('set_and_view_user_permissions')">
            <i v-tooltip.top="t('user.manage_permissions')" class="cursor-pointer pi pi-key" @click="editPermissionsDialogVisible = true" />
          </template>
          <template v-if="userStore.permissions.includes('change_user_class')">
            <i v-tooltip.top="t('user.change_user_class')" class="cursor-pointer pi pi-crown" @click="changeUserClassDialogVisible = true" />
          </template>
          <template v-if="userStore.permissions.includes('lock_user_class')">
            <i
              v-if="user.class_locked"
              v-tooltip.top="t('user.unlock_user_class')"
              class="cursor-pointer pi pi-unlock"
              @click="lockUnlockClassDialogVisible = true"
            />
            <i v-else v-tooltip.top="t('user.lock_user_class')" class="cursor-pointer pi pi-lock" @click="lockUnlockClassDialogVisible = true" />
          </template>
          <template v-if="userStore.permissions.includes('warn_user') && userStore.id !== user.id">
            <i v-tooltip.top="t('user.warn')" class="cursor-pointer pi pi-exclamation-triangle" @click="warnUserDialogVisible = true" />
          </template>
          <template v-if="userStore.permissions.includes('remove_user_warning') && userStore.id !== user.id && (user.warned || user.banned)">
            <i
              v-tooltip.top="user.banned ? t('user.remove_ban') : t('user.remove_warning')"
              class="cursor-pointer pi pi-eraser"
              @click="removeWarningDialogVisible = true"
            />
          </template>
          <template v-if="userStore.permissions.includes('set_user_custom_title')">
            <i v-tooltip.top="t('user.set_custom_title')" class="cursor-pointer pi pi-id-card" @click="setCustomTitleDialogVisible = true" />
          </template>
          <template v-if="userStore.id === user.id">
            <i v-tooltip.top="t('general.edit')" class="cursor-pointer pi pi-pen-to-square" @click="editUserDialogVisible = true" />
          </template>
          <template v-if="userStore.id !== user.id && userStore.permissions.includes('generate_reset_password_token')">
            <i v-tooltip.top="t('user.generate_password_reset_link')" class="cursor-pointer pi pi-lock" @click="passwordResetLinkDialogVisible = true" />
          </template>
        </div>
      </div>
      <ContentContainer :containerTitle="t('general.description')" class="section">
        <BBCodeRenderer :content="user.description" />
      </ContentContainer>
      <ContentContainer v-if="showTorrentClients" :containerTitle="t('torrent.clients_and_ips')" class="section">
        <TorrentClientTable :clients="torrentClients" />
      </ContentContainer>
      <EarnedBadges
        :badges="earnedBadges"
        :userId="user.id"
        :username="user.username"
        class="section"
        @awarded="fetchUser"
        @revoked="(id) => (earnedBadges = earnedBadges.filter((b) => b.id !== id))"
      />
      <LatestTorrents
        v-if="uploadedTorrents.length > 0 || userStore.id === user.id"
        :titleGroups="uploadedTorrents"
        class="section"
        :containerTitle="t('user.uploads')"
        :containerTitleLink="`/torrents?torrent_created_by_id=${user.id}`"
        type="uploads"
      >
        <template #top-right>
          <i
            v-if="canDownloadOwnTorrents && uploadedTorrents.length > 0"
            v-tooltip.top="t('user.download_all_uploaded_torrents')"
            :class="['cursor-pointer', isDownloadingUploadedArchive ? 'pi pi-spin pi-spinner' : 'pi pi-download']"
            @click="downloadTorrentsArchive('uploaded')"
          />
        </template>
      </LatestTorrents>
      <LatestTorrents
        v-if="snatchedTorrents.length > 0 || userStore.id === user.id"
        :titleGroups="snatchedTorrents"
        class="section"
        :containerTitle="t('user.snatches')"
        :containerTitleLink="`/torrents?torrent_snatched_by_id=${user.id}&order_by_column=torrent_snatched_at`"
        type="snatches"
      >
        <template #top-right>
          <i
            v-if="canDownloadOwnTorrents && snatchedTorrents.length > 0"
            v-tooltip.top="t('user.download_all_snatched_torrents')"
            :class="['cursor-pointer', isDownloadingSnatchedArchive ? 'pi pi-spin pi-spinner' : 'pi pi-download']"
            @click="downloadTorrentsArchive('snatched')"
          />
        </template>
      </LatestTorrents>
      <UserRegistrationIp
        v-if="userStore.permissions.includes('view_registration_ips')"
        :key="`registration-ip-${user.id}`"
        :userId="user.id"
        class="section"
      />
      <UserStaffNotes v-if="userStore.permissions.includes('write_user_staff_note')" :key="user.id" :userId="user.id" class="section" />
    </div>
    <UserSidebar :user class="sidebar" />
  </div>
  <Dialog closeOnEscape modal :header="t('user.warn_user')" v-model:visible="warnUserDialogVisible">
    <WarnUserDialog @warned="warningCreated" />
  </Dialog>
  <Dialog closeOnEscape modal :header="user?.banned ? t('user.remove_ban') : t('user.remove_warning')" v-model:visible="removeWarningDialogVisible">
    <DeleteDialog
      v-if="removeWarningDialogVisible && user"
      :message="user.banned ? t('user.confirm_remove_ban') : t('user.confirm_remove_warning')"
      :action="() => removeUserWarningsAndBans(user!.id)"
      :successMessage="user.banned ? t('user.ban_removed_success') : t('user.warning_removed_success')"
      :label="user.banned ? t('user.remove_ban') : t('user.remove_warning')"
      @deleted="warningRemoved"
    />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.edit_profile')" v-model:visible="editUserDialogVisible">
    <EditUserDialog @done="userEdited" :initialUser="user as EditedUser" v-if="user" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.manage_permissions')" v-model:visible="editPermissionsDialogVisible">
    <EditPermissionsDialog @saved="permissionsSaved" :userId="user!.id" v-if="editPermissionsDialogVisible && user" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.change_user_class')" v-model:visible="changeUserClassDialogVisible">
    <ChangeUserClassDialog @saved="userClassChanged" :userId="user!.id" :currentClassName="user!.class_name" v-if="changeUserClassDialogVisible && user" />
  </Dialog>
  <Dialog
    closeOnEscape
    modal
    :header="user?.class_locked ? t('user.unlock_user_class') : t('user.lock_user_class')"
    v-model:visible="lockUnlockClassDialogVisible"
  >
    <LockUnlockUserClassDialog @saved="classLockChanged" :userId="user!.id" :classLocked="user!.class_locked" v-if="lockUnlockClassDialogVisible && user" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.set_custom_title')" v-model:visible="setCustomTitleDialogVisible">
    <SetCustomTitleDialog @saved="customTitleChanged" :userId="user!.id" :currentCustomTitle="user!.custom_title" v-if="setCustomTitleDialogVisible && user" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.gift.send_gift', [user?.username])" v-model:visible="sendGiftDialogVisible">
    <SendGiftDialog :receiverId="user!.id" @sent="sendGiftDialogVisible = false" v-if="sendGiftDialogVisible && user" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.generate_password_reset_link')" v-model:visible="passwordResetLinkDialogVisible">
    <PasswordResetLinkDialog :userId="user!.id" v-if="passwordResetLinkDialogVisible && user" />
  </Dialog>
</template>

<script setup lang="ts">
import { config } from '@/config'
import { ref, onMounted, computed, watch } from 'vue'
import { useUserStore } from '@/stores/user'
import { useRoute } from 'vue-router'
import UserSidebar from '@/components/user/UserSidebar.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import WarnUserDialog from '@/components/user/WarnUserDialog.vue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import TorrentClientTable from '@/components/user/TorrentClientTable.vue'
import { Dialog } from 'primevue'
import LatestTorrents from '@/components/torrent/LatestTorrents.vue'
import EarnedBadges from '@/components/user/EarnedBadges.vue'
import EditUserDialog from '@/components/user/EditUserDialog.vue'
import EditPermissionsDialog from '@/components/user/EditPermissionsDialog.vue'
import ChangeUserClassDialog from '@/components/user/ChangeUserClassDialog.vue'
import LockUnlockUserClassDialog from '@/components/user/LockUnlockUserClassDialog.vue'
import SetCustomTitleDialog from '@/components/user/SetCustomTitleDialog.vue'
import SendGiftDialog from '@/components/user/SendGiftDialog.vue'
import PasswordResetLinkDialog from '@/components/user/PasswordResetLinkDialog.vue'
import UserStaffNotes from '@/components/user/UserStaffNotes.vue'
import UserRegistrationIp from '@/components/user/UserRegistrationIp.vue'
import {
  getMe,
  getUser,
  removeUserWarningsAndBans,
  type EditedUser,
  type PublicUser,
  type TitleGroupHierarchyLite,
  type TorrentClient,
  type User,
  type UserEarnedBadgeWithDetails,
  type UserWarning,
} from '@/services/api-schema'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import { downloadUserTorrentsArchive } from '@/services/api/torrentService'
import { showToast } from '@/main'

const torrentClients = ref<TorrentClient[]>([])
const user = ref<User | PublicUser | null>(null)
const uploadedTorrents = ref<TitleGroupHierarchyLite[]>([])
const snatchedTorrents = ref<TitleGroupHierarchyLite[]>([])
const earnedBadges = ref<UserEarnedBadgeWithDetails[]>([])
const siteName = config.site_name

const userStore = useUserStore()
const route = useRoute()
const { t } = useI18n()

const showTorrentClients = computed(() => {
  return userStore.id === parseInt(route.params.id.toString()) || userStore.permissions.includes('see_foreign_torrent_clients')
})

// Only the user themselves can download all of their own torrents, and only with the permission
// the backend requires to serve a .torrent file.
const canDownloadOwnTorrents = computed(() => {
  return userStore.id === user.value?.id && userStore.permissions.includes('download_torrent')
})

const isDownloadingUploadedArchive = ref(false)
const isDownloadingSnatchedArchive = ref(false)

const downloadTorrentsArchive = (kind: 'uploaded' | 'snatched') => {
  const isDownloading = kind === 'uploaded' ? isDownloadingUploadedArchive : isDownloadingSnatchedArchive
  if (isDownloading.value) {
    return
  }
  isDownloading.value = true
  downloadUserTorrentsArchive(kind)
    .catch(() => showToast('', t('user.download_torrents_archive_error'), 'error', 5000))
    .finally(() => (isDownloading.value = false))
}

const warnUserDialogVisible = ref(false)
const removeWarningDialogVisible = ref(false)
const editUserDialogVisible = ref(false)
const editPermissionsDialogVisible = ref(false)
const changeUserClassDialogVisible = ref(false)
const lockUnlockClassDialogVisible = ref(false)
const setCustomTitleDialogVisible = ref(false)
const sendGiftDialogVisible = ref(false)
const passwordResetLinkDialogVisible = ref(false)

const userEdited = (userEdited: EditedUser) => {
  user.value = { ...user.value, ...userEdited } as User
  editUserDialogVisible.value = false
}

const permissionsSaved = () => {
  editPermissionsDialogVisible.value = false
}

const userClassChanged = (className: string) => {
  if (user.value) {
    user.value.class_name = className
  }
  changeUserClassDialogVisible.value = false
}

const classLockChanged = (classLocked: boolean) => {
  if (user.value) {
    user.value.class_locked = classLocked
  }
  lockUnlockClassDialogVisible.value = false
}

const warningCreated = (warning: UserWarning) => {
  if (user.value) {
    user.value.warned = true
    if (warning.ban) {
      user.value.banned = true
    }
  }
  warnUserDialogVisible.value = false
}

const warningRemoved = () => {
  if (user.value) {
    user.value.warned = false
    user.value.banned = false
  }
  removeWarningDialogVisible.value = false
}

const customTitleChanged = (customTitle: string | null) => {
  if (user.value) {
    user.value.custom_title = customTitle
  }
  setCustomTitleDialogVisible.value = false
}

const fetchUser = async () => {
  if (parseInt(route.params.id.toString()) == userStore.id) {
    // logged in user
    ;({
      torrent_clients: torrentClients.value,
      user: user.value,
      last_five_uploaded_torrents: uploadedTorrents.value,
      last_five_snatched_torrents: snatchedTorrents.value,
      earned_badges: earnedBadges.value,
    } = await getMe())
    userStore.setUser(user.value as User)
  } else {
    // viewing another user
    ;({
      torrent_clients: torrentClients.value,
      user: user.value,
      last_five_uploaded_torrents: uploadedTorrents.value,
      last_five_snatched_torrents: snatchedTorrents.value,
      earned_badges: earnedBadges.value,
    } = await getUser(parseInt(route.params.id.toString())))
  }

  document.title = `User '${user.value.username}' - ${siteName}`
}

onMounted(async () => {
  fetchUser()
})

watch(
  () => route.params.id,
  (newId, oldId) => {
    if (oldId !== undefined) {
      fetchUser()
    }
  },
  { immediate: true },
)
</script>

<style scoped>
#user-view {
  display: flex;
}
.main {
  width: 75%;
  margin-right: 10px;
}
.top-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.username-enriched {
  font-weight: bold;
  font-size: 1.3em;
  margin-bottom: 10px;
}
.actions {
  i {
    margin-left: 7px;
  }
}
.section {
  margin-bottom: 15px;
}
.sidebar {
  width: 25%;
}
</style>
