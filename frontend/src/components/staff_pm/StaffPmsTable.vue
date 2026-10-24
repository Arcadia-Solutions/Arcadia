<template>
  <DataTable :value="staffPMs" :rowClass="(pm) => (isPMReadOrResolved(pm) ? '' : 'bg-unread')">
    <Column :header="t('conversation.subject')">
      <template #body="slotProps">
        <RouterLink :to="`/staff-pm/${slotProps.data.id}`" @click="isPMReadOrResolved(slotProps.data) ? null : (notificationsStore.staff_pm_messages -= 1)">
          {{ slotProps.data.subject }}
        </RouterLink>
      </template>
    </Column>
    <Column :header="t('conversation.last_message')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.last_message.created_at" />
        {{ t('general.by') }}
        <UsernameEnriched :user="slotProps.data.last_message.created_by" displayAllInfo />
      </template>
    </Column>
    <Column :header="t('general.started')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.created_at" />
      </template>
    </Column>
    <Column :header="t('general.created_by')">
      <template #body="slotProps">
        <UsernameEnriched :user="slotProps.data.created_by" displayAllInfo />
      </template>
    </Column>
    <Column :header="t('staff_pm.resolved')">
      <template #body="slotProps">
        <i v-if="slotProps.data.resolved" class="pi pi-check" />
        <i v-else class="pi pi-times" />
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { listStaffPMs, type StaffPmOverview } from '@/services/api-schema'
import { useNotificationsStore } from '@/stores/notifications'
import { useUserStore } from '@/stores/user'
import { Column, DataTable } from 'primevue'
import { ref } from 'vue'
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import UsernameEnriched from '../user/UsernameEnriched.vue'

const { t } = useI18n()
const notificationsStore = useNotificationsStore()

const loading = ref(true)

const staffPMs = ref<StaffPmOverview[]>([])

const isPMReadOrResolved = (p: StaffPmOverview) => {
  const userId = useUserStore().id
  return p.last_message.created_by.id === userId || p.resolved
}

onMounted(() => {
  listStaffPMs()
    .then((data) => (staffPMs.value = data))
    .finally(() => (loading.value = false))
})
</script>

<style scoped></style>
