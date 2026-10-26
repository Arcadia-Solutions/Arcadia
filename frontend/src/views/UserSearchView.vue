<template>
  <ContentContainer class="search-form">
    <div class="filters-row">
      <FloatLabel>
        <InputText v-model="searchForm.username" size="small" />
        <label>{{ t('user.username') }}</label>
      </FloatLabel>
      <FloatLabel>
        <DatePicker v-model="searchForm.registered_after" size="small" dateFormat="yy-mm-dd" showButtonBar />
        <label>{{ t('user.registered_after') }}</label>
      </FloatLabel>
      <FloatLabel>
        <DatePicker v-model="searchForm.registered_before" size="small" dateFormat="yy-mm-dd" showButtonBar />
        <label>{{ t('user.registered_before') }}</label>
      </FloatLabel>
      <template v-if="canFilterByPermissions">
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.permissions"
            :options="permissionOptions"
            optionLabel="label"
            optionValue="value"
            size="small"
            display="chip"
            filter
            :maxSelectedLabels="5"
            input-id="permissionsMultiSelect"
            style="min-width: 20em"
          />
          <label for="permissionsMultiSelect">{{ t('user.permissions') }}</label>
        </FloatLabel>
        <FloatLabel>
          <Select
            v-model="searchForm.permissions_match"
            :options="[
              { label: t('user.permissions_match_any'), value: UserPermissionMatchMode.Any },
              { label: t('user.permissions_match_all'), value: UserPermissionMatchMode.All },
            ]"
            optionLabel="label"
            optionValue="value"
            size="small"
            input-id="permissionsMatchSelect"
            style="min-width: 20em"
          />
          <label for="permissionsMatchSelect">{{ t('user.permissions_match') }}</label>
        </FloatLabel>
      </template>
    </div>
    <div class="actions">
      <Button :label="t('general.search')" size="small" :loading="loading" @click="updateUrl" />
      <RouterLink v-if="userStore.permissions.includes('send_mass_pm')" :to="massPmLink">
        <Button :label="t('user.mass_pm')" size="small" severity="secondary" icon="pi pi-send" />
      </RouterLink>
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
    <DataTable :value="searchResults" size="small" :sortField="searchForm.order_by" :sortOrder="sortOrder" lazy @sort="onSort">
      <Column style="width: 50px">
        <template #body="slotProps">
          <img :src="slotProps.data.avatar ?? '/default_user_avatar.png'" class="avatar" />
        </template>
      </Column>
      <Column field="username" :header="t('user.username')" sortable>
        <template #body="slotProps">
          <UsernameEnriched :user="slotProps.data" />
        </template>
      </Column>
      <Column field="class_name" :header="t('user.class')" />
      <Column v-if="shouldStatBeDisplayed('joined_at')" field="created_at" :header="userStatLabel('joined_at')" sortable>
        <template #body="slotProps"><TimeAgo v-if="slotProps.data.created_at" :date="slotProps.data.created_at" /></template>
      </Column>
      <Column v-if="shouldStatBeDisplayed('last_seen')" field="last_seen" :header="userStatLabel('last_seen')" sortable>
        <template #body="slotProps"><TimeAgo v-if="slotProps.data.last_seen" :date="slotProps.data.last_seen" /></template>
      </Column>
      <Column v-if="shouldStatBeDisplayed('uploaded')" field="uploaded" :header="userStatLabel('uploaded')" sortable>
        <template #body="slotProps">{{ slotProps.data.uploaded != null ? bytesToReadable(slotProps.data.uploaded) : '' }}</template>
      </Column>
      <Column v-if="shouldStatBeDisplayed('downloaded')" field="downloaded" :header="userStatLabel('downloaded')" sortable>
        <template #body="slotProps">{{ slotProps.data.downloaded != null ? bytesToReadable(slotProps.data.downloaded) : '' }}</template>
      </Column>
      <Column v-if="shouldStatBeDisplayed('torrents')" field="torrents" :header="userStatLabel('torrents')" sortable />
      <Column v-if="shouldStatBeDisplayed('title_groups')" field="title_groups" :header="userStatLabel('title_groups')" sortable />
      <Column v-if="shouldStatBeDisplayed('title_group_comments')" field="title_group_comments" :header="userStatLabel('title_group_comments')" sortable />
      <Column field="title_group_tags_applied" :header="t('user.title_group_tags_applied')" sortable />
      <Column v-if="shouldStatBeDisplayed('forum_posts')" field="forum_posts" :header="userStatLabel('forum_posts')" sortable />
      <Column v-if="shouldStatBeDisplayed('forum_threads')" field="forum_threads" :header="userStatLabel('forum_threads')" sortable />
      <Column v-if="shouldStatBeDisplayed('seeding')" field="seeding" :header="userStatLabel('seeding')" sortable />
      <Column v-if="shouldStatBeDisplayed('bonus_points')" field="bonus_points" :header="userStatLabel('bonus_points')" sortable>
        <template #body="slotProps">{{
          slotProps.data.bonus_points != null ? formatBp(slotProps.data.bonus_points, publicArcadiaSettings.bonus_points_decimal_places) : ''
        }}</template>
      </Column>
    </DataTable>
  </PaginatedResults>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { onMounted, ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { Button, FloatLabel, InputText, DataTable, Column, DatePicker, MultiSelect, Select } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import {
  searchUsers,
  type UserSearchResult,
  UserSearchOrderBy,
  OrderByDirection,
  UserPermission,
  UserPermissionMatchMode,
  type DisplayableUserStats,
} from '@/services/api-schema'
import { bytesToReadable, formatBp } from '@/services/helpers'
import { useUserStatLabel } from '@/composables/useUserStatLabel'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { useUserStore } from '@/stores/user'
import type { DataTableSortEvent } from 'primevue/datatable'
import type { RouteLocationRaw } from 'vue-router'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const userStore = useUserStore()

const userStatLabel = useUserStatLabel()

const shouldStatBeDisplayed = (stat: DisplayableUserStats) => publicArcadiaSettings.displayable_user_stats.includes(stat)

const permissionOptions = computed(() =>
  Object.values(UserPermission).map((permission) => ({
    value: permission,
    label: t(`user_permissions.${permission}`),
  })),
)

interface SearchForm {
  username: string
  registered_after: Date | null
  registered_before: Date | null
  permissions: UserPermission[]
  permissions_match: UserPermissionMatchMode
  order_by: UserSearchOrderBy
  order_by_direction: OrderByDirection
  page: number
  page_size: number
}

const canFilterByPermissions = computed(() => userStore.permissions.includes('set_and_view_user_permissions'))

const searchForm = ref<SearchForm>({
  username: '',
  registered_after: null,
  registered_before: null,
  permissions: [],
  permissions_match: UserPermissionMatchMode.Any,
  order_by: UserSearchOrderBy.CreatedAt,
  order_by_direction: OrderByDirection.Asc,
  page: 1,
  page_size: 25,
})

const searchResults = ref<UserSearchResult[]>([])
const totalResults = ref(0)
const loading = ref(false)
const totalPages = computed(() => Math.ceil(totalResults.value / searchForm.value.page_size))

// Reuses the "new conversation" page in mass mode, passing the current filter along.
const massPmLink = computed<RouteLocationRaw>(() => ({
  path: '/conversation/new',
  query: {
    mass: 'true',
    username: searchForm.value.username || undefined,
    registered_after: searchForm.value.registered_after?.toISOString() ?? undefined,
    registered_before: searchForm.value.registered_before?.toISOString() ?? undefined,
    permissions: searchForm.value.permissions.length > 0 ? searchForm.value.permissions : undefined,
    permissions_match: searchForm.value.permissions.length > 0 ? searchForm.value.permissions_match : undefined,
  },
}))

const userSearchOrderByValues: string[] = Object.values(UserSearchOrderBy)
const isUserSearchOrderBy = (value: unknown): value is UserSearchOrderBy => typeof value === 'string' && userSearchOrderByValues.includes(value)

const orderByDirectionValues: string[] = Object.values(OrderByDirection)
const isOrderByDirection = (value: unknown): value is OrderByDirection => typeof value === 'string' && orderByDirectionValues.includes(value)

const userPermissionValues: string[] = Object.values(UserPermission)
const isUserPermission = (value: unknown): value is UserPermission => typeof value === 'string' && userPermissionValues.includes(value)

const userPermissionMatchModeValues: string[] = Object.values(UserPermissionMatchMode)
const isUserPermissionMatchMode = (value: unknown): value is UserPermissionMatchMode =>
  typeof value === 'string' && userPermissionMatchModeValues.includes(value)

// Vue router exposes a repeated query parameter as a string or as an array, depending on its count.
const readPermissionsFromRoute = (): UserPermission[] => {
  const value = route.query.permissions
  return (Array.isArray(value) ? value : [value]).filter(isUserPermission)
}

const sortOrder = computed(() => (searchForm.value.order_by_direction === OrderByDirection.Asc ? 1 : -1))

const onSort = (event: DataTableSortEvent) => {
  if (typeof event.sortField === 'string' && isUserSearchOrderBy(event.sortField)) {
    searchForm.value.order_by = event.sortField
    searchForm.value.order_by_direction = event.sortOrder === 1 ? OrderByDirection.Asc : OrderByDirection.Desc
    updateUrl()
  }
}

const onPageChange = (pagination: { page: number }) => {
  searchForm.value.page = pagination.page
  updateUrl()
}

const updateUrl = () => {
  router.push({
    query: {
      username: searchForm.value.username || undefined,
      registered_after: searchForm.value.registered_after?.toISOString() ?? undefined,
      registered_before: searchForm.value.registered_before?.toISOString() ?? undefined,
      permissions: searchForm.value.permissions.length > 0 ? searchForm.value.permissions : undefined,
      permissions_match: searchForm.value.permissions.length > 0 ? searchForm.value.permissions_match : undefined,
      order_by: searchForm.value.order_by,
      order_by_direction: searchForm.value.order_by_direction,
      page: searchForm.value.page.toString(),
    },
  })
}

const fetchSearchResults = () => {
  const orderBy = route.query.order_by
  const orderByDirection = route.query.order_by_direction
  const registeredAfter = route.query.registered_after
  const registeredBefore = route.query.registered_before
  const permissionsMatch = route.query.permissions_match

  searchForm.value.page = route.query.page ? parseInt(route.query.page.toString()) : 1
  searchForm.value.username = route.query.username?.toString() ?? ''
  searchForm.value.registered_after = registeredAfter ? new Date(registeredAfter.toString()) : null
  searchForm.value.registered_before = registeredBefore ? new Date(registeredBefore.toString()) : null
  // Permissions may only be searched on by staff, so a hand-crafted URL cannot grant access to them.
  searchForm.value.permissions = canFilterByPermissions.value ? readPermissionsFromRoute() : []
  searchForm.value.permissions_match = isUserPermissionMatchMode(permissionsMatch) ? permissionsMatch : UserPermissionMatchMode.Any
  searchForm.value.order_by = isUserSearchOrderBy(orderBy) ? orderBy : UserSearchOrderBy.CreatedAt
  searchForm.value.order_by_direction = isOrderByDirection(orderByDirection) ? orderByDirection : OrderByDirection.Desc

  const searchedPermissions = searchForm.value.permissions.length > 0 ? searchForm.value.permissions : undefined

  loading.value = true
  searchUsers({
    username: searchForm.value.username || undefined,
    registered_after: searchForm.value.registered_after?.toISOString() ?? undefined,
    registered_before: searchForm.value.registered_before?.toISOString() ?? undefined,
    permissions: searchedPermissions,
    permissions_match: searchForm.value.permissions_match,
    order_by: searchForm.value.order_by,
    order_by_direction: searchForm.value.order_by_direction,
    page: searchForm.value.page,
    page_size: searchForm.value.page_size,
  })
    .then((response) => {
      searchResults.value = response.results
      totalResults.value = response.total_items
    })
    .finally(() => {
      loading.value = false
    })
}

onMounted(() => fetchSearchResults())

watch(
  () => route.query,
  () => fetchSearchResults(),
  { deep: true },
)
</script>

<style scoped>
.search-form {
  margin-bottom: 15px;
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
  align-items: center;
}
.filters-row {
  display: flex;
  gap: 15px;
  align-items: center;
  flex-wrap: wrap;
}
.actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
}
.avatar {
  width: 50px;
  border-radius: 7px;
  object-fit: cover;
}
</style>
