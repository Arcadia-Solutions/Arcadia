<template>
  <div class="user-edit-logs-table">
    <UserEditLogsSearchForm v-if="initialForm" ref="searchFormRef" :loading :initialForm />
    <PaginatedResults
      v-if="initialForm && records"
      :totalPages
      :initialPage="initialForm.page"
      :totalItems="totalItems"
      :pageSize
      @changePage="searchFormRef.changePage($event.page)"
    >
      <div v-if="canDelete && records.length > 0" class="delete-all-container">
        <i class="pi pi-trash cursor-pointer" v-tooltip.top="t('user_edit_log.delete_all')" @click="deleteAllDialogVisible = true" />
      </div>
      <DataTable :value="records" scrollable scrollHeight="70vh" size="small">
        <Column field="edited_by.username" :header="t('user_edit_log.edited_by')">
          <template #body="slotProps">
            <UsernameEnriched :user="slotProps.data.edited_by" />
          </template>
        </Column>
        <Column field="item_type" :header="t('user_edit_log.item_type')">
          <template #body="slotProps">
            {{ slotProps.data.item_type }}
          </template>
        </Column>
        <Column field="item_id" :header="t('user_edit_log.item_id')">
          <template #body="slotProps">
            <RouterLink
              v-if="getItemLink(slotProps.data.item_type, slotProps.data.item_id)"
              :to="getItemLink(slotProps.data.item_type, slotProps.data.item_id)!"
            >
              {{ slotProps.data.item_id }}
            </RouterLink>
            <span v-else>{{ slotProps.data.item_id }}</span>
          </template>
        </Column>
        <Column field="edits" :header="t('user_edit_log.edits')">
          <template #body="slotProps">
            <DiffViewer :edits="slotProps.data.edits" />
          </template>
        </Column>
        <Column field="edited_at" :header="t('user_edit_log.edited_at')">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.edited_at" />
          </template>
        </Column>
        <Column v-if="canDelete" :header="t('user_edit_log.actions')">
          <template #body="slotProps">
            <i class="pi pi-trash cursor-pointer" v-tooltip.top="t('general.delete')" @click="openDeleteDialog(slotProps.data.id)" />
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
    <Dialog closeOnEscape modal :header="t('general.delete')" v-model:visible="deleteDialogVisible">
      <div class="delete-dialog">
        <p>{{ t('user_edit_log.confirm_delete') }}</p>
        <Button :label="t('general.delete')" severity="danger" size="small" :loading="deleting" @click="confirmDelete" />
      </div>
    </Dialog>
    <Dialog closeOnEscape modal :header="t('user_edit_log.delete_all')" v-model:visible="deleteAllDialogVisible">
      <div class="delete-dialog">
        <p>{{ t('user_edit_log.confirm_delete_all') }}</p>
        <Button :label="t('user_edit_log.delete_all')" severity="danger" size="small" :loading="deletingAll" @click="confirmDeleteAll" />
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, RouterLink } from 'vue-router'
import { Button, Column, DataTable, Dialog } from 'primevue'
import {
  searchUserEditChangeLogs,
  deleteUserEditChangeLog,
  deleteAllUserEditChangeLogs,
  type PaginatedResultsUserEditChangeLogResultResultsInner,
  type SearchUserEditChangeLogsRequest,
  UserEditChangeLogSortByColumn,
} from '@/services/api-schema'
import PaginatedResults from '@/components/PaginatedResults.vue'
import UserEditLogsSearchForm from '@/components/staff/UserEditLogsSearchForm.vue'
import DiffViewer from '@/components/staff/DiffViewer.vue'
import type { VNodeRef } from 'vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const canDelete = computed(() => userStore.permissions.includes('delete_user_edit_change_log'))

const getItemLink = (itemType: string, itemId: number): string | null => {
  const routes: Record<string, string> = {
    artist: `/artist/${itemId}`,
    title_group: `/title-group/${itemId}`,
    series: `/series/${itemId}`,
    forum_thread: `/forum/thread/${itemId}`,
    forum_sub_category: `/forum/sub-category/${itemId}`,
    torrent_request: `/torrent-request/${itemId}`,
    wiki_article: `/wiki/article/${itemId}`,
    collage: `/collage/${itemId}`,
  }
  return routes[itemType] ?? null
}

const searchFormRef = ref<VNodeRef | null>(null)
const records = ref<PaginatedResultsUserEditChangeLogResultResultsInner[]>()
const loading = ref(false)
const initialForm = ref<SearchUserEditChangeLogsRequest | null>(null)
const totalItems = ref(0)
const pageSize = ref(20)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize.value))

const search = async (form: SearchUserEditChangeLogsRequest) => {
  loading.value = true
  const response = await searchUserEditChangeLogs(form).finally(() => {
    loading.value = false
  })
  pageSize.value = form.page_size
  totalItems.value = response.total_items
  records.value = response.results
}

const deleteDialogVisible = ref(false)
const deleteAllDialogVisible = ref(false)
const deleting = ref(false)
const deletingAll = ref(false)
const deleteTargetId = ref<number | null>(null)

const openDeleteDialog = (id: number) => {
  deleteTargetId.value = id
  deleteDialogVisible.value = true
}

const confirmDelete = async () => {
  if (deleteTargetId.value === null) return
  deleting.value = true
  await deleteUserEditChangeLog(deleteTargetId.value).finally(() => {
    deleting.value = false
  })
  deleteDialogVisible.value = false
  showToast('', t('user_edit_log.deleted_success'), 'success', 2000)
  loadFormFromUrl()
}

const confirmDeleteAll = async () => {
  deletingAll.value = true
  await deleteAllUserEditChangeLogs().finally(() => {
    deletingAll.value = false
  })
  deleteAllDialogVisible.value = false
  showToast('', t('user_edit_log.deleted_all_success'), 'success', 2000)
  loadFormFromUrl()
}

const loadFormFromUrl = async () => {
  loading.value = true
  initialForm.value = null
  await nextTick()

  const form: SearchUserEditChangeLogsRequest = {
    page: route.query.page ? parseInt(route.query.page as string) : 1,
    page_size: route.query.page_size ? parseInt(route.query.page_size as string) : 20,
    user_id: route.query.user_id ? parseInt(route.query.user_id as string) : undefined,
    item_type: route.query.item_type ? (route.query.item_type as string) : undefined,
    sort_by_column:
      (route.query.sort_by_column as (typeof UserEditChangeLogSortByColumn)[keyof typeof UserEditChangeLogSortByColumn]) ??
      UserEditChangeLogSortByColumn.EditedAt,
    sort_by_direction: (route.query.sort_by_direction as 'asc' | 'desc') ?? 'desc',
  }

  initialForm.value = form
  pageSize.value = form.page_size
  search(form)
}

onMounted(() => {
  loadFormFromUrl()
})

watch(
  () => route.query,
  () => {
    loadFormFromUrl()
  },
  { deep: true },
)
</script>

<style scoped>
.user-edit-logs-table {
  margin-top: 20px;
}

.delete-all-container {
  display: flex;
  justify-content: center;
  margin-bottom: 10px;
}

.delete-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}
</style>
