<template>
  <div class="applied-tags">
    <ContentContainer>
      <Message severity="info" size="small" class="explanation">{{ t('applied_tag.explanation') }}</Message>
      <div class="search-inputs">
        <AutoComplete
          v-model="tagName"
          :suggestions="foundTagNames"
          @complete="searchTags"
          size="small"
          :placeholder="t('applied_tag.tag')"
          @option-select="search"
        />
        <UserSearchBar
          v-model="username"
          :placeholder="t('user.username')"
          :clearInputOnSelect="false"
          :clickableUserLink="false"
          @userSelected="
            (user) => {
              username = user.username
              search()
            }
          "
        />
        <InputText v-model="titleGroupName" size="small" :placeholder="t('general.title')" @keyup.enter="search" />
        <Select
          v-model="contentType"
          :options="getSelectableContentTypes().map((value) => ({ label: t(`title_group.content_type.${value}`), value }))"
          optionLabel="label"
          optionValue="value"
          :placeholder="t('title_group.content_type.content_type')"
          showClear
          size="small"
          @change="search"
        />
        <span>{{ t('applied_tag.user_applied_tags_period') }}</span>
        <Select
          v-model="period"
          :options="
            ['last_day', 'last_week', 'last_month', 'last_year', 'all_time', 'custom_range'].map((value) => ({ label: t(`applied_tag.${value}`), value }))
          "
          optionLabel="label"
          optionValue="value"
          size="small"
          @change="period !== 'custom_range' && search()"
        />
        <DatePicker
          v-if="period === 'custom_range'"
          v-model="customRange"
          selectionMode="range"
          :manualInput="false"
          dateFormat="yy-mm-dd"
          showIcon
          iconDisplay="input"
          size="small"
          @update:modelValue="customRange?.[1] && search()"
        />
        <Button :label="t('general.search')" size="small" :loading @click="search" />
      </div>
    </ContentContainer>
    <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="onPageChange">
      <DataTable :value="appliedTags" :loading size="small" scrollable lazy :sortField :sortOrder @sort="onSort" tableStyle="table-layout: fixed">
        <template #empty>{{ t('applied_tag.no_applied_tags') }}</template>
        <Column field="tag_name" style="width: 12%" :header="t('applied_tag.tag')" sortable />
        <Column field="title_group_name" :header="t('general.title')" sortable>
          <template #body="slotProps">
            <TitleGroupSlimHeader
              :titleGroup="slotProps.data.title_group"
              :series="slotProps.data.title_group.series"
              :affiliatedArtists="slotProps.data.title_group.affiliated_artists"
              nameLink
            />
          </template>
        </Column>
        <Column field="content_type" style="width: 9%" :header="t('title_group.content_type.content_type')" sortable>
          <template #body="slotProps">
            {{ t(`title_group.content_type.${slotProps.data.title_group.content_type}`) }}
          </template>
        </Column>
        <Column field="username" style="width: 11%" :header="t('applied_tag.applied_by')" sortable>
          <template #body="slotProps">
            <UsernameEnriched :user="slotProps.data.created_by" />
          </template>
        </Column>
        <Column field="created_at" style="width: 9%" :header="t('applied_tag.applied_at')" sortable>
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.created_at" />
          </template>
        </Column>
        <Column
          field="user_applied_tags_amount"
          style="width: 10%"
          :header="t('applied_tag.user_applied_tags', { period: t(`applied_tag.${period}`) })"
          sortable
        />
        <Column field="title_group_tags_amount" style="width: 7%" :header="t('applied_tag.tags_on_title')" sortable>
          <template #body="slotProps">
            <span v-tooltip.top="slotProps.data.title_group.tags.join(', ')">{{ slotProps.data.title_group.tags.length }}</span>
          </template>
        </Column>
        <Column style="width: 3em">
          <template #body="slotProps">
            <i class="pi pi-trash cursor-pointer" v-tooltip.top="t('title_group.remove_tag')" @click="removeTag(slotProps.data)" />
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { AutoComplete, Button, Column, DataTable, DatePicker, InputText, Message, Select, type DataTableSortEvent } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import UserSearchBar from '@/components/user/UserSearchBar.vue'
import { getSelectableContentTypes } from '@/services/helpers'
import { showToast } from '@/main'
import {
  OrderByDirection,
  removeTagFromTitleGroup,
  searchTitleGroupAppliedTags,
  searchTitleGroupTagsLite,
  TitleGroupAppliedTagOrderByColumn,
  type ContentType,
  type TitleGroupAppliedTag,
} from '@/services/api-schema'

const { t } = useI18n()

const tagName = ref('')
const username = ref('')
const titleGroupName = ref('')
const foundTagNames = ref<string[]>([])
const contentType = ref<ContentType | null>(null)
const period = ref('all_time')
const customRange = ref<(Date | null)[] | null>(null)
const periodDays: Record<string, number> = { last_day: 1, last_week: 7, last_month: 30, last_year: 365 }

const appliedTags = ref<TitleGroupAppliedTag[]>([])
const loading = ref(true)
const sortField = ref<TitleGroupAppliedTagOrderByColumn>(TitleGroupAppliedTagOrderByColumn.CreatedAt)
const sortOrder = ref<1 | -1>(-1)
const page = ref(1)
const pageSize = 50
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))

const searchTags = () => {
  searchTitleGroupTagsLite({ name: tagName.value, page: 1, page_size: 10 }).then((tags) => {
    foundTagNames.value = tags.results.map((tag) => tag.name)
  })
}

const userAppliedTagsDateRange = (): { user_applied_tags_after?: string; user_applied_tags_before?: string } => {
  if (period.value in periodDays) {
    return { user_applied_tags_after: new Date(Date.now() - periodDays[period.value] * 24 * 60 * 60 * 1000).toISOString() }
  }
  const [start, end] = customRange.value ?? []
  if (period.value !== 'custom_range' || !start || !end) {
    return {}
  }
  const endOfDay = new Date(end)
  endOfDay.setHours(23, 59, 59, 999)
  return { user_applied_tags_after: start.toISOString(), user_applied_tags_before: endOfDay.toISOString() }
}

const fetchAppliedTags = () => {
  loading.value = true
  searchTitleGroupAppliedTags({
    content_type: contentType.value ?? undefined,
    ...userAppliedTagsDateRange(),
    tag_name: tagName.value || undefined,
    username: username.value || undefined,
    title_group_name: titleGroupName.value || undefined,
    page: page.value,
    page_size: pageSize,
    order_by_column: sortField.value,
    order_by_direction: sortOrder.value === 1 ? OrderByDirection.Asc : OrderByDirection.Desc,
  })
    .then((data) => {
      appliedTags.value = data.results
      totalItems.value = data.total_items
    })
    .finally(() => {
      loading.value = false
    })
}

const removeTag = (appliedTag: TitleGroupAppliedTag) => {
  removeTagFromTitleGroup({ title_group_id: appliedTag.title_group.id, tag_name: appliedTag.tag_name }).then(() => {
    showToast('', t('applied_tag.tag_removed'), 'success', 2000)
    fetchAppliedTags()
  })
}

const search = () => {
  page.value = 1
  fetchAppliedTags()
}

const onSort = (event: DataTableSortEvent) => {
  sortField.value = event.sortField as TitleGroupAppliedTagOrderByColumn
  sortOrder.value = (event.sortOrder as 1 | -1) ?? -1
  search()
}

const onPageChange = (pagination: { page: number }) => {
  page.value = pagination.page
  fetchAppliedTags()
}

onMounted(fetchAppliedTags)
</script>

<style scoped>
.explanation {
  margin-bottom: 15px;
}
.search-inputs {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
}
.applied-tags :deep(.p-datatable-column-header-content) {
  margin-right: 0.8rem;
}
</style>
