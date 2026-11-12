<template>
  <div>
    <div class="page-title">{{ t('community.title_group_ratings') }}</div>
    <TitleGroupRatingFilters v-model="filters" :loading hideSorting @search="fetchRatings(1)" />
    <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="fetchRatings($event.page)">
      <DataTable :value="ratings" size="small" lazy :sortField="filters.order_by_column" :sortOrder @sort="onSort">
        <Column :header="t('title_group.title')">
          <template #body="{ data }: { data: TitleGroupRatingWithTitleGroup }">
            <TitleGroupSlimHeader
              :titleGroup="data.title_group"
              :series="data.title_group.series"
              :affiliatedArtists="data.title_group.affiliated_artists"
              nameLink
            />
          </template>
        </Column>
        <Column :header="t('title_group.rating')" field="rating" sortable>
          <template #body="{ data }: { data: TitleGroupRatingWithTitleGroup }">
            <UserRatingDisplay :rating="data.rating" />
          </template>
        </Column>
        <Column :header="t('title_group.comment')">
          <template #body="{ data }: { data: TitleGroupRatingWithTitleGroup }">
            <BBCodeRenderer v-if="data.comment" :content="data.comment" />
          </template>
        </Column>
        <Column :header="t('general.created_at')" field="created_at" sortable>
          <template #body="{ data }: { data: TitleGroupRatingWithTitleGroup }">
            <TimeAgo :date="data.created_at" />
          </template>
        </Column>
        <Column v-if="isOwnPage || canDelete" :header="t('general.action', 2)" class="actions">
          <template #body="{ data }: { data: TitleGroupRatingWithTitleGroup }">
            <i v-if="isOwnPage" v-tooltip.top="t('general.edit')" class="pi pi-pen-to-square cursor-pointer" @click="ratingToEdit = data" />
            <i v-if="canDelete" v-tooltip.top="t('general.delete')" class="pi pi-trash cursor-pointer" @click="ratingToDelete = data" />
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
    <Dialog closeOnEscape modal :header="t('title_group.your_rating')" :visible="ratingToEdit !== null" @update:visible="ratingToEdit = null">
      <TitleGroupVoteForm
        v-if="ratingToEdit !== null"
        :titleGroupId="ratingToEdit.title_group.id"
        :ownRating="{ ...ratingToEdit, title_group_id: ratingToEdit.title_group.id }"
        @saved="ratingSaved"
      />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.delete_rating')" :visible="ratingToDelete !== null" @update:visible="ratingToDelete = null">
      <DeleteDialog
        v-if="ratingToDelete !== null"
        :message="t('title_group.confirm_delete_rating')"
        :action="() => deleteTitleGroupRating(ratingToDelete!.id)"
        :successMessage="t('title_group.rating_deleted_success')"
        @deleted="ratingDeleted"
      />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { Column, DataTable, Dialog, type DataTableSortEvent } from 'primevue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import TitleGroupVoteForm from '@/components/title_group/TitleGroupVoteForm.vue'
import { useUserStore } from '@/stores/user'
import TimeAgo from '@/components/TimeAgo.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import UserRatingDisplay from '@/components/title_group/UserRatingDisplay.vue'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import TitleGroupRatingFilters, { type TitleGroupRatingFilters as Filters } from '@/components/title_group/TitleGroupRatingFilters.vue'
import {
  deleteTitleGroupRating,
  OrderByDirection,
  searchUserTitleGroupRatings,
  TitleGroupRatingSearchOrderByColumn,
  type TitleGroupRating,
  type TitleGroupRatingWithTitleGroup,
} from '@/services/api-schema'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const pageSize = 25
const page = ref(1)
const ratings = ref<TitleGroupRatingWithTitleGroup[]>([])
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))
const loading = ref(false)
const filters = ref<Filters>({
  minimum_rating: 0,
  maximum_rating: 100,
  content: '',
  order_by_column: TitleGroupRatingSearchOrderByColumn.CreatedAt,
  order_by_direction: OrderByDirection.Desc,
})

const ratingToEdit = ref<TitleGroupRatingWithTitleGroup | null>(null)
const ratingToDelete = ref<TitleGroupRatingWithTitleGroup | null>(null)
const isOwnPage = computed(() => userStore.id === parseInt(route.params.id.toString()))
const canDelete = computed(() => userStore.permissions.includes('delete_title_group_rating'))

const sortOrder = computed(() => (filters.value.order_by_direction === OrderByDirection.Asc ? 1 : -1))

const onSort = (event: DataTableSortEvent) => {
  filters.value.order_by_column =
    event.sortField === TitleGroupRatingSearchOrderByColumn.Rating ? TitleGroupRatingSearchOrderByColumn.Rating : TitleGroupRatingSearchOrderByColumn.CreatedAt
  filters.value.order_by_direction = event.sortOrder === 1 ? OrderByDirection.Asc : OrderByDirection.Desc
  fetchRatings(1)
}

const fetchRatings = (newPage: number) => {
  page.value = newPage
  loading.value = true
  searchUserTitleGroupRatings({
    user_id: parseInt(route.params.id.toString()),
    ...filters.value,
    page: newPage,
    page_size: pageSize,
  })
    .then((paginatedRatings) => {
      ratings.value = paginatedRatings.results
      totalItems.value = paginatedRatings.total_items
    })
    .finally(() => (loading.value = false))
}

const ratingSaved = (saved: TitleGroupRating) => {
  if (ratingToEdit.value !== null) Object.assign(ratingToEdit.value, { rating: saved.rating, comment: saved.comment, updated_at: saved.updated_at })
  ratingToEdit.value = null
}

const ratingDeleted = () => {
  ratingToDelete.value = null
  fetchRatings(page.value)
}

watch(
  () => route.params.id,
  () => fetchRatings(1),
  { immediate: true },
)
</script>

<style scoped>
.page-title {
  font-size: 1.2em;
  margin-bottom: 10px;
}
.actions i {
  margin-right: 5px;
}
</style>
