<template>
  <div class="all-conversations-table">
    <ContentContainer class="search-form">
      <div class="search-term">
        <InputText
          v-model="searchForm.search_term"
          size="small"
          :placeholder="searchForm.search_titles_only ? t('conversation.search_subjects') : t('conversation.search_subjects_and_messages')"
          @keyup.enter="updateUrl"
        />
        <div class="search-titles-only">
          <Checkbox v-model="searchForm.search_titles_only" :binary="true" inputId="all-conversations-search-titles-only" />
          <label for="all-conversations-search-titles-only">{{ t('conversation.search_titles_only') }}</label>
        </div>
      </div>
      <UserSearchBar
        v-model="filterUsername"
        :placeholder="t('conversation.involving_user')"
        :clearInputOnSelect="false"
        :clickableUserLink="false"
        :initialValue="filterUsername"
        @userSelected="onUserSelected"
      />
      <Button :label="t('general.search')" icon="pi pi-search" size="small" @click="updateUrl" />
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
          :sortField="searchForm.order_by_column"
          :sortOrder="searchForm.order_by_direction === OrderByDirection.Asc ? 1 : -1"
          @sort="onSort"
          removableSort
        >
          <Column :header="t('conversation.subject')" sortable :sortField="ConversationSearchOrderByColumn.Subject">
            <template #body="slotProps">
              <RouterLink :to="`/conversation/${slotProps.data.conversation_id}`">
                {{ slotProps.data.subject }}
              </RouterLink>
            </template>
          </Column>
          <Column :header="t('conversation.sender')">
            <template #body="slotProps">
              <UsernameEnriched
                :user="{
                  id: slotProps.data.sender_id,
                  username: slotProps.data.sender_username,
                  warned: slotProps.data.sender_warned,
                  banned: slotProps.data.sender_banned,
                }"
              />
            </template>
          </Column>
          <Column :header="t('conversation.receiver')">
            <template #body="slotProps">
              <UsernameEnriched
                :user="{
                  id: slotProps.data.receiver_id,
                  username: slotProps.data.receiver_username,
                  warned: slotProps.data.receiver_warned,
                  banned: slotProps.data.receiver_banned,
                }"
              />
            </template>
          </Column>
          <Column :header="t('conversation.last_message')" sortable :sortField="ConversationSearchOrderByColumn.LastMessage">
            <template #body="slotProps">
              <TimeAgo :date="slotProps.data.last_message_created_at" />
              {{ t('general.by') }}
              <UsernameEnriched
                :user="{
                  id: slotProps.data.last_message_created_by_id,
                  username: slotProps.data.last_message_created_by_username,
                  warned: false,
                  banned: false,
                }"
              />
            </template>
          </Column>
          <Column :header="t('general.created_at')" sortable :sortField="ConversationSearchOrderByColumn.CreatedAt">
            <template #body="slotProps">
              <TimeAgo :date="slotProps.data.conversation_created_at" />
            </template>
          </Column>
          <Column :header="t('conversation.messages_amount')" field="messages_amount" sortable :sortField="ConversationSearchOrderByColumn.MessagesAmount" />
        </DataTable>
      </div>
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { ConversationSearchOrderByColumn, OrderByDirection } from '@/services/api-schema'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import UserSearchBar from '@/components/user/UserSearchBar.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { Button, Checkbox, InputText } from 'primevue'
import { useConversationSearch } from '@/composables/useConversationSearch'

const { t } = useI18n()

const { searchForm, filterUsername, searchResults, totalResults, totalPages, updateUrl, onChangePage, onSort, onUserSelected } = useConversationSearch({
  tab: 'allConversations',
  allConversations: true,
})
</script>

<style scoped>
.search-form {
  margin-bottom: 15px;
}
.search-form :deep(.content-body) {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 10px;
}
.search-term input {
  width: 300px;
}
.all-conversations-table {
  margin-top: 20px;
}
.search-titles-only {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-top: 5px;
}
</style>
