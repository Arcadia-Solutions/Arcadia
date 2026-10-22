<template>
  <div class="report" v-for="report in reports" :key="report.id">
    <UsernameEnriched :user="report.reported_by" />
    <span>({{ timeAgo(report.reported_at) }})</span>
    : {{ report.description }}
    <i
      v-if="userStore.permissions.includes('delete_torrent_report')"
      v-tooltip.top="t('general.delete')"
      class="pi pi-trash delete-btn"
      @click="deleteReport(report.id)"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { timeAgo } from '@/services/helpers'
import { useUserStore } from '@/stores/user'
import { deleteTorrentReport, type TorrentReport } from '@/services/api-schema'
import { showToast } from '@/main'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'

const { t } = useI18n()
const userStore = useUserStore()

defineProps<{
  reports: TorrentReport[]
}>()

const emit = defineEmits<{
  deleted: [reportId: number]
}>()

const deleteReport = (reportId: number) => {
  deleteTorrentReport(reportId).then(() => {
    showToast('', t('torrent.report_deleted'), 'success', 2000)
    emit('deleted', reportId)
  })
}
</script>

<style scoped>
.report {
  display: flex;
  align-items: center;
  gap: 0.5em;
}

.delete-btn {
  cursor: pointer;
  margin-left: auto;
}
</style>
