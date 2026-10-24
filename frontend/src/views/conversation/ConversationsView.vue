<template>
  <div class="title">{{ t('conversation.conversation', 2) }}</div>
  <ContentContainer class="search-form">
    <FloatLabel>
      <InputText v-model="searchForm.search_term" size="small" />
      <label>{{ t('general.search') }}</label>
    </FloatLabel>
    <div class="search-titles-only">
      <Checkbox v-model="searchForm.search_titles_only" :binary="true" inputId="search-titles-only" />
      <label for="search-titles-only">{{ t('conversation.search_titles_only') }}</label>
    </div>
    <div class="wrapper-center">
      <Button :label="t('general.search')" size="small" @click="updateUrl" />
    </div>
  </ContentContainer>
  <PaginatedResults
    v-if="searchResults.length > 0"
    :totalItems="totalResults"
    :pageSize="searchForm.page_size"
    :initialPage="searchForm.page"
    :totalPages="totalPages"
    @changePage="onChangePage"
  >
    <div class="card">
      <DataTable
        :value="searchResults"
        size="small"
        :rowClass="(conversation) => (isConversationRead(conversation) ? '' : 'bg-unread')"
        :sortField="searchForm.order_by_column"
        :sortOrder="searchForm.order_by_direction === OrderByDirection.Asc ? 1 : -1"
        @sort="onSort"
        removableSort
      >
        <Column :header="t('conversation.subject')" sortable :sortField="ConversationSearchOrderByColumn.Subject">
          <template #body="slotProps">
            <RouterLink
              :to="`/conversation/${slotProps.data.conversation_id}`"
              @click="isConversationRead(slotProps.data) ? null : (notificationsStore.conversations -= 1)"
            >
              {{ slotProps.data.subject }}
            </RouterLink>
          </template>
        </Column>
        <Column :header="t('conversation.last_message')" sortable :sortField="ConversationSearchOrderByColumn.LastMessage">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.last_message_created_at" />
            {{ t('general.by') }}
            <UsernameEnriched
              :user="{ id: slotProps.data.last_message_created_by_id, username: slotProps.data.last_message_created_by_username, warned: false, banned: false }"
            />
          </template>
        </Column>
        <Column :header="t('general.started')" sortable :sortField="ConversationSearchOrderByColumn.CreatedAt">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.conversation_created_at" />
          </template>
        </Column>
        <Column :header="t('conversation.correspondent')">
          <template #body="slotProps">
            <UsernameEnriched
              :user="{
                id: slotProps.data.correspondant_id,
                username: slotProps.data.correspondant_username,
                warned: slotProps.data.correspondant_warned,
                banned: slotProps.data.correspondant_banned,
              }"
            />
          </template>
        </Column>
      </DataTable>
    </div>
  </PaginatedResults>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { useNotificationsStore } from '@/stores/notifications'
import { ConversationSearchOrderByColumn, OrderByDirection, type PaginatedResultsConversationSearchResultResultsInner } from '@/services/api-schema'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { Button, Checkbox, FloatLabel, InputText } from 'primevue'
import { useConversationSearch } from '@/composables/useConversationSearch'

const { t } = useI18n()
const notificationsStore = useNotificationsStore()

const { searchForm, searchResults, totalResults, totalPages, updateUrl, onChangePage, onSort } = useConversationSearch()

const isConversationRead = (c: PaginatedResultsConversationSearchResultResultsInner) => {
  const userId = useUserStore().id
  return (
    c.last_message_created_by_id === userId ||
    (userId === c.receiver_id
      ? c.receiver_last_seen_at != null && new Date(c.receiver_last_seen_at).getTime() > new Date(c.last_message_created_at).getTime()
      : new Date(c.sender_last_seen_at).getTime() > new Date(c.last_message_created_at).getTime())
  )
}
</script>

<style scoped>
.title {
  margin-bottom: 10px;
}
.search-form {
  margin-bottom: 15px;
}
.search-titles-only {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-top: 5px;
}
</style>
