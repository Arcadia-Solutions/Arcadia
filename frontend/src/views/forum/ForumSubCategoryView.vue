<template>
  <div v-if="forumSubCategory">
    <div class="top-bar">
      <div class="title">
        <RouterLink to="/forum">{{ forumSubCategory.category.name }}</RouterLink> >
        <RouterLink to="">{{ forumSubCategory.name }}</RouterLink>
      </div>
      <div class="actions">
        <RouterLink :to="`/forum/sub-category/${route.params.id}/edit`" v-if="forumSubCategory && userStore.permissions.includes('edit_forum_sub_category')">
          <i v-tooltip.top="t('forum.edit_subcategory')" class="pi pi-pen-to-square cursor-pointer" />
        </RouterLink>
        <i
          v-if="forumSubCategory.new_threads_restricted && userStore.permissions.includes('edit_forum_sub_category')"
          v-tooltip.top="t('forum.manage_allowed_posters')"
          class="pi pi-users cursor-pointer"
          @click="manageAllowedPostersDialogVisible = true"
        />
        <RouterLink
          v-if="userStore.permissions.includes('create_forum_thread') && (!forumSubCategory.new_threads_restricted || forumSubCategory.is_allowed_poster)"
          :to="`/forum/thread/new?subCategoryId=${route.params.id}`"
        >
          <i v-tooltip.top="t('forum.new_thread')" class="pi pi-plus cursor-pointer" />
        </RouterLink>
        <i v-if="togglingSubscription" class="pi pi-hourglass" />
        <i
          v-else
          v-tooltip.top="t(`general.${forumSubCategory.is_subscribed ? 'un' : ''}subscribe`)"
          @click="toggleSubscription"
          :class="`pi pi-bell${forumSubCategory.is_subscribed ? '-slash' : ''} cursor-pointer`"
        />
        <i
          v-if="userStore.permissions.includes('delete_forum_sub_category')"
          v-tooltip.top="t('forum.delete_subcategory')"
          class="pi pi-trash cursor-pointer"
          @click="deleteSubCategoryDialogVisible = true"
        />
      </div>
    </div>
    <DataTable :value="forumSubCategory.threads">
      <Column style="width: 1em">
        <template #body="slotProps">
          <div class="left-icon" v-tooltip.top="threadTooltip(slotProps.data)">
            <i :class="[slotProps.data.pinned ? 'pi pi-thumbtack' : 'pi pi-align-left', { unread: slotProps.data.has_new_posts }]" />
            <i class="pi pi-sparkles" v-if="!slotProps.data.ever_opened" />
          </div>
        </template>
      </Column>
      <Column field="name" :header="t('general.name')">
        <template #body="slotProps">
          <RouterLink :to="`/forum/thread/${slotProps.data.id}`" @click="onThreadClick(slotProps.data)">
            {{ slotProps.data.name }}
          </RouterLink>
        </template>
      </Column>
      <Column field="latest_post" :header="t('forum.latest_post')">
        <template #body="slotProps">
          <TimeAgo :date="slotProps.data.latest_post.created_at" /> {{ t('general.by') }}
          <UsernameEnriched :user="slotProps.data.latest_post.created_by" />
        </template>
      </Column>
      <Column field="posts_amount" :header="t('forum.posts')">
        <template #body="slotProps">{{ formatNumber(slotProps.data.posts_amount) }}</template>
      </Column>
      <Column field="views_count" :header="t('forum.views')">
        <template #body="slotProps">{{ formatNumber(slotProps.data.views_count) }}</template>
      </Column>
    </DataTable>
  </div>
  <Dialog closeOnEscape modal :header="t('forum.delete_subcategory')" v-model:visible="deleteSubCategoryDialogVisible">
    <DeleteDialog
      :message="t('forum.confirm_delete_subcategory')"
      :action="() => deleteForumSubCategory(parseInt(route.params.id as string))"
      :successMessage="t('forum.subcategory_deleted_success')"
      @deleted="onSubCategoryDeleted"
    />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('forum.manage_allowed_posters')" v-model:visible="manageAllowedPostersDialogVisible">
    <ManageAllowedPostersDialog :subCategoryId="parseInt(route.params.id as string)" />
  </Dialog>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { config } from '@/config'
import { useI18n } from 'vue-i18n'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { formatNumber } from '@/services/helpers'
import { RouterLink, useRouter } from 'vue-router'
import { useRoute } from 'vue-router'
import { onMounted } from 'vue'
import { ref } from 'vue'
import {
  deleteForumSubCategory,
  getForumSubCategoryThreads,
  createForumSubCategoryThreadsSubscription,
  removeForumSubCategoryThreadsSubscription,
  type ForumSubCategoryHierarchy,
  type ForumThreadHierarchy,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { useNotificationsStore } from '@/stores/notifications'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import { Dialog } from 'primevue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import ManageAllowedPostersDialog from '@/components/forum/ManageAllowedPostersDialog.vue'
import { showToast } from '@/main'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const userStore = useUserStore()
const notificationsStore = useNotificationsStore()

const forumSubCategory = ref<null | ForumSubCategoryHierarchy>(null)
const siteName = config.site_name
const deleteSubCategoryDialogVisible = ref(false)
const manageAllowedPostersDialogVisible = ref(false)
const togglingSubscription = ref(false)

onMounted(async () => {
  forumSubCategory.value = await getForumSubCategoryThreads(parseInt(route.params.id as string))

  document.title = forumSubCategory.value ? `${forumSubCategory.value.name} - ${siteName}` : `Forum category - ${siteName}`
})

const threadTooltip = (thread: ForumThreadHierarchy) =>
  [
    thread.pinned && t('forum.pinned'),
    !thread.ever_opened && t('forum.new'),
    thread.has_new_posts && t('forum.unread'),
    !thread.has_new_posts && t('forum.read'),
  ]
    .filter(Boolean)
    .join(', ')

const onThreadClick = (thread: ForumThreadHierarchy) => {
  if (!thread.ever_opened && route.params.id === '1' && notificationsStore.announcements > 0) {
    notificationsStore.announcements--
  }
}

const toggleSubscription = () => {
  if (!forumSubCategory.value) return
  togglingSubscription.value = true
  const subCategoryId = parseInt(route.params.id as string)
  const action = forumSubCategory.value.is_subscribed
    ? removeForumSubCategoryThreadsSubscription(subCategoryId)
    : createForumSubCategoryThreadsSubscription(subCategoryId)
  action
    .then(() => {
      forumSubCategory.value!.is_subscribed = !forumSubCategory.value!.is_subscribed
      showToast('', t(`title_group.${forumSubCategory.value!.is_subscribed ? 'subscription_successful' : 'unsubscription_successful'}`), 'success', 3000)
    })
    .finally(() => (togglingSubscription.value = false))
}

const onSubCategoryDeleted = () => {
  deleteSubCategoryDialogVisible.value = false
  router.push('/forum')
}
</script>

<style scoped>
.top-bar {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 10px;
  .actions {
    i {
      color: white;
      margin-left: 7px;
    }
  }
}
.unread {
  color: var(--p-primary-color);
}
.left-icon {
  display: flex;
  .pi-sparkles {
    font-size: 0.95em;
    margin-left: -0.5em;
    margin-top: -0.4em;
  }
}
</style>
