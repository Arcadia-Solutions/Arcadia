<template>
  <div class="reorder-dialog">
    <Message severity="info" size="small">{{ t('user_badge.reorder_applies_to_everyone') }}</Message>
    <DataTable :value="badges" size="small" @row-reorder="(event) => (badges = event.value)">
      <Column rowReorder style="width: 2rem" />
      <Column style="width: 40px">
        <template #body="{ data }">
          <img :src="data.image_url" :alt="data.name" class="badge-thumb" />
        </template>
      </Column>
      <Column field="name" :header="t('general.name')" />
    </DataTable>
    <Button :label="t('general.save')" size="small" :loading="saving" @click="save" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Column, DataTable, Message } from 'primevue'
import { showToast } from '@/main'
import { reorderUserBadges, type UserBadge } from '@/services/api-schema'

const { t } = useI18n()

const props = defineProps<{
  initialBadges: UserBadge[]
}>()

const emit = defineEmits<{
  reordered: []
}>()

const badges = ref([...props.initialBadges])
const saving = ref(false)

const save = () => {
  saving.value = true
  reorderUserBadges({ badges: badges.value.map((badge, index) => ({ id: badge.id, sort_order: index })) })
    .then(() => {
      showToast('', t('user_badge.reorder_success'), 'success', 2000)
      emit('reordered')
    })
    .finally(() => {
      saving.value = false
    })
}
</script>

<style scoped>
.reorder-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
}
.badge-thumb {
  width: 32px;
  height: 32px;
  object-fit: contain;
}
</style>
