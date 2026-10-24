<template>
  <ContentContainer class="search-form">
    <FloatLabel>
      <InputText v-model="searchForm.name" size="small" @keyup.enter="updateUrl" />
      <label for="name">{{ t('general.name') }}</label>
    </FloatLabel>
    <div class="show-deleted">
      <Checkbox v-model="searchForm.show_deleted" inputId="show_deleted" size="small" binary />
      <label for="show_deleted">{{ t('general.show_deleted') }}</label>
    </div>
    <div class="wrapper-center">
      <Button :label="t('general.search')" size="small" @click="updateUrl" :loading />
    </div>
  </ContentContainer>
  <PaginatedResults
    v-if="searchResults.length > 0"
    :totalItems="totalResults"
    :pageSize="searchForm.page_size"
    :initialPage="searchForm.page"
    :totalPages="totalPages"
    @changePage="onPageChange"
  >
    <DataTable :value="searchResults" size="small" lazy :sortField="searchForm.order_by_column" :sortOrder @sort="onSort">
      <Column :header="t('general.name')" field="name" sortable>
        <template #body="slotProps">
          <RouterLink :to="{ path: '/torrents', query: { title_group_tags: slotProps.data.name } }">
            {{ slotProps.data.name }}
          </RouterLink>
        </template>
      </Column>
      <Column :header="t('general.created_by')" field="created_by" sortable>
        <template #body="slotProps">
          <UsernameEnriched :user="slotProps.data.created_by" />
        </template>
      </Column>
      <Column :header="t('general.uses')" field="uses" sortable>
        <template #body="slotProps">
          {{ slotProps.data.uses }}
        </template>
      </Column>
      <Column :header="t('general.synonym', 2)">
        <template #body="slotProps">
          <span v-if="slotProps.data.synonyms.length > 0">
            {{ slotProps.data.synonyms.join(', ') }}
          </span>
        </template>
      </Column>
      <Column :header="t('general.created_at')" field="created_at" sortable>
        <template #body="slotProps">
          <span>
            <TimeAgo :date="slotProps.data.created_at" />
          </span>
        </template>
      </Column>
      <Column v-if="deletedColumnsVisible" :header="t('general.deleted_by')" field="deleted_by" sortable>
        <template #body="slotProps">
          <UsernameEnriched v-if="slotProps.data.deleted_by" :user="slotProps.data.deleted_by" />
        </template>
      </Column>
      <Column v-if="deletedColumnsVisible" :header="t('general.deleted_at')" field="deleted_at" sortable>
        <template #body="slotProps">
          <span v-if="slotProps.data.deleted_at">
            <TimeAgo :date="slotProps.data.deleted_at" />
          </span>
        </template>
      </Column>
      <Column v-if="deletedColumnsVisible" :header="t('title_group.deletion_reason')">
        <template #body="slotProps">
          {{ slotProps.data.deletion_reason }}
        </template>
      </Column>
      <Column :header="t('general.action', 2)" class="actions">
        <template #body="slotProps">
          <i
            class="pi pi-pen-to-square cursor-pointer"
            v-if="userStore.permissions.includes('edit_title_group_tag') && !slotProps.data.deleted_at"
            v-tooltip.top="t('general.edit')"
            @click="editTag(slotProps.data)"
          />
          <i
            class="pi pi-trash cursor-pointer"
            v-if="userStore.permissions.includes('manage_title_group_tags') && !slotProps.data.deleted_at"
            v-tooltip.top="t('general.delete')"
            @click="deleteTag(slotProps.data)"
          />
          <i
            class="pi pi-arrow-down-left-and-arrow-up-right-to-center cursor-pointer"
            v-if="userStore.permissions.includes('manage_title_group_tags') && !slotProps.data.deleted_at"
            v-tooltip.top="t('title_group.merge_tag')"
            @click="mergeTag(slotProps.data)"
          />
          <i
            class="pi pi-replay cursor-pointer"
            v-if="userStore.permissions.includes('manage_title_group_tags') && slotProps.data.deleted_at"
            v-tooltip.top="t('title_group.restore_tag')"
            @click="restoreTag(slotProps.data)"
          />
        </template>
      </Column>
    </DataTable>
  </PaginatedResults>
  <Dialog closeOnEscape modal :header="t('general.edit')" v-model:visible="editTagDialogVisible">
    <EditTitleGroupTagDialog v-if="tagBeingEdited" :initialTag="tagBeingEdited" @done="tagEdited" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('general.delete')" v-model:visible="deleteTagDialogVisible">
    <DeleteTitleGroupTagDialog v-if="tagBeingDeleted" :tag="tagBeingDeleted" @deleted="tagDeleted" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('title_group.merge_tag')" v-model:visible="mergeTagDialogVisible">
    <MergeTitleGroupTagDialog v-if="tagBeingMerged" :tag="tagBeingMerged" @merged="tagMerged" />
  </Dialog>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { onMounted, ref, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { Button, Checkbox, FloatLabel, InputText, DataTable, Column, Dialog } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { useUserStore } from '@/stores/user'
import EditTitleGroupTagDialog from '@/components/title_group_tag/EditTitleGroupTagDialog.vue'
import DeleteTitleGroupTagDialog from '@/components/title_group_tag/DeleteTitleGroupTagDialog.vue'
import MergeTitleGroupTagDialog from '@/components/title_group_tag/MergeTitleGroupTagDialog.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import {
  restoreTitleGroupTag,
  searchTitleGroupTags,
  type EditedTitleGroupTag,
  type SearchTitleGroupTagsQuery,
  type TitleGroupTagEnriched,
} from '@/services/api-schema'
import type { DataTableSortEvent } from 'primevue/datatable'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const userStore = useUserStore()

const loading = ref(false)
const searchForm = ref<SearchTitleGroupTagsQuery>({
  name: '',
  show_deleted: false,
  order_by_column: 'name',
  order_by_direction: 'asc',
  page: 1,
  page_size: 50,
})
const searchResults = ref<TitleGroupTagEnriched[]>([])
const totalResults = ref<number>(0)
const totalPages = computed(() => Math.ceil(totalResults.value / searchForm.value.page_size))
const sortOrder = computed(() => (searchForm.value.order_by_direction === 'asc' ? 1 : -1))
const deletedColumnsVisible = computed(() => route.query.show_deleted === 'true')

const onSort = (event: DataTableSortEvent) => {
  router.push({
    query: {
      ...route.query,
      order_by_column: event.sortField as string,
      order_by_direction: event.sortOrder === 1 ? 'asc' : 'desc',
    },
  })
}
const editTagDialogVisible = ref(false)
const deleteTagDialogVisible = ref(false)
const mergeTagDialogVisible = ref(false)
const tagBeingEdited = ref<EditedTitleGroupTag | null>(null)
const tagBeingDeleted = ref<EditedTitleGroupTag | null>(null)
const tagBeingMerged = ref<EditedTitleGroupTag | null>(null)

const onPageChange = (pagination: { page: number }) => {
  searchForm.value.page = pagination.page
  router.push({ query: { ...searchForm.value, show_deleted: String(searchForm.value.show_deleted) } })
}

const updateUrl = () => {
  searchForm.value.page = 1
  router.push({ query: { ...searchForm.value, show_deleted: String(searchForm.value.show_deleted) } })
}

const fetchSearchResultsFromUrl = async () => {
  loading.value = true
  searchForm.value.page = route.query.page ? parseInt(route.query.page as string) : 1
  searchForm.value.page_size = route.query.page_size ? parseInt(route.query.page_size as string) : 50
  searchForm.value.name = route.query.name ? (route.query.name as string) : ''
  searchForm.value.show_deleted = route.query.show_deleted === 'true'
  // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
  searchForm.value.order_by_column = route.query.order_by_column ? (route.query.order_by_column as string) : 'created_at'
  // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
  searchForm.value.order_by_direction = route.query.order_by_direction ? (route.query.order_by_direction as string) : 'desc'
  const response = await searchTitleGroupTags(searchForm.value)
  searchResults.value.length = 0
  await nextTick()
  searchResults.value = response.results
  totalResults.value = response.total_items
  loading.value = false
}

onMounted(async () => {
  await fetchSearchResultsFromUrl()
})

watch(
  () => route.query,
  () => {
    fetchSearchResultsFromUrl()
  },
  { deep: true },
)

const editTag = (tag: TitleGroupTagEnriched) => {
  tagBeingEdited.value = {
    id: tag.id,
    name: tag.name,
    synonyms: tag.synonyms,
  }
  editTagDialogVisible.value = true
}

const deleteTag = (tag: EditedTitleGroupTag) => {
  tagBeingDeleted.value = {
    id: tag.id,
    name: tag.name,
    synonyms: tag.synonyms,
  }
  deleteTagDialogVisible.value = true
}

const tagEdited = () => {
  editTagDialogVisible.value = false
  fetchSearchResultsFromUrl()
}

const tagDeleted = () => {
  deleteTagDialogVisible.value = false
  fetchSearchResultsFromUrl()
}

const mergeTag = (tag: EditedTitleGroupTag) => {
  tagBeingMerged.value = {
    id: tag.id,
    name: tag.name,
    synonyms: tag.synonyms,
  }
  mergeTagDialogVisible.value = true
}

const tagMerged = () => {
  mergeTagDialogVisible.value = false
  fetchSearchResultsFromUrl()
}

const restoreTag = (tag: TitleGroupTagEnriched) => {
  restoreTitleGroupTag({ id: tag.id }).then(() => {
    fetchSearchResultsFromUrl()
  })
}
</script>

<style scoped>
.search-form {
  margin-bottom: 15px;
}
.show-deleted {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-top: 10px;
}
.actions {
  i {
    margin-right: 5px;
  }
}
</style>
