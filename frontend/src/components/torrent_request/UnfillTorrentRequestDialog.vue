<template>
  <div class="unfill-torrent-request">
    <Message severity="warn" size="small" variant="simple" class="warning">
      {{ t('torrent_request.unfill_warning') }}
    </Message>
    <FloatLabel class="input">
      <Textarea v-model="reason" rows="4" fluid />
      <label>{{ t('general.reason') }}</label>
    </FloatLabel>
    <Button :label="t('torrent_request.unfill_request')" severity="danger" size="small" :loading :disabled="!reason.trim()" @click="unfillRequest" />
  </div>
</template>

<script setup lang="ts">
import { unfillTorrentRequest } from '@/services/api-schema'
import { FloatLabel, Message, Textarea } from 'primevue'
import Button from 'primevue/button'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  torrentRequestId: number
}>()

const emit = defineEmits<{
  unfilled: []
}>()

const reason = ref('')
const loading = ref(false)

const unfillRequest = () => {
  loading.value = true
  unfillTorrentRequest({ torrent_request_id: props.torrentRequestId, reason: reason.value.trim() })
    .then(() => emit('unfilled'))
    .finally(() => (loading.value = false))
}
</script>

<style scoped>
.unfill-torrent-request {
  padding-top: 20px;
  width: 30em !important;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.warning {
  margin-bottom: 20px;
  text-align: center;
}
.input {
  width: 25em;
  margin-bottom: 20px;
}
</style>
