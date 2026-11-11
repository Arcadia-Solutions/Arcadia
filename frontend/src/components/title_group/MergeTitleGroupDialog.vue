<template>
  <div class="merge-dialog">
    <p>{{ t('title_group.merge_description') }}</p>
    <div class="target-input">
      <FloatLabel variant="on">
        <InputText v-model="targetIdInput" inputId="target_id" size="small" :disabled="!!targetData" />
        <label for="target_id">{{ t('title_group.target_title_group_id') }}</label>
      </FloatLabel>
      <Button v-if="!targetData" :label="t('general.check')" size="small" :loading="fetching" :disabled="!targetIdInput" @click="fetchTarget" />
      <Button v-else :label="t('general.reset')" size="small" severity="secondary" @click="((targetData = undefined), (targetIdInput = ''))" />
    </div>
    <template v-if="targetData">
      <div class="preview-section">
        <h4>{{ t('title_group.source') }} (ID: {{ titleGroupId }})</h4>
        <TitleGroupPreviewTable :title_group="sourceTitleGroupHierarchy" />
      </div>
      <div class="preview-section">
        <h4>{{ t('title_group.target') }} (ID: {{ targetData.title_group.id }})</h4>
        <TitleGroupPreviewTable :title_group="targetTitleGroupHierarchy!" />
      </div>
      <p class="merge-warning">{{ t('title_group.confirm_merge_title_group') }}</p>
      <Button :label="t('general.confirm')" severity="danger" size="small" :loading="merging" @click="handleMerge" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { showToast } from '@/main'
import { getTitleGroup, mergeTitleGroups, type TitleGroupAndAssociatedData, type TitleGroupHierarchyLite } from '@/services/api-schema'
import TitleGroupPreviewTable from './TitleGroupPreviewTable.vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import FloatLabel from 'primevue/floatlabel'

const { t } = useI18n()

const props = defineProps<{
  titleGroupId: number
  sourceData: TitleGroupAndAssociatedData
}>()

const emit = defineEmits<{
  merged: [targetId: number]
}>()

const targetIdInput = ref('')
const targetId = computed(() => {
  const parsed = parseInt(targetIdInput.value)
  return isNaN(parsed) ? undefined : parsed
})
const targetData = ref<TitleGroupAndAssociatedData>()
const fetching = ref(false)
const merging = ref(false)

const toHierarchyLite = (data: TitleGroupAndAssociatedData): TitleGroupHierarchyLite => ({
  id: data.title_group.id,
  name: data.title_group.name,
  content_type: data.title_group.content_type,
  covers: data.title_group.covers,
  original_release_date: data.title_group.original_release_date,
  original_release_date_only_year_known: data.title_group.original_release_date_only_year_known,
  tags: data.title_group.tags,
  category: data.title_group.category,
  platform: data.title_group.platform,
  user_rating_average: data.title_group.user_rating_average,
  user_rating_count: data.title_group.user_rating_count,
  series: data.series,
  affiliated_artists: data.affiliated_artists.map((a) => ({ artist_id: a.artist.id, name: a.artist.name })),
  edition_groups: data.edition_groups.map((eg) => ({
    id: eg.id,
    name: eg.name,
    title_group_id: eg.title_group_id,
    covers: eg.covers,
    release_date: eg.release_date,
    release_date_only_year_known: eg.release_date_only_year_known,
    additional_information: eg.additional_information,
    source: eg.source,
    distributor: eg.distributor,
    torrents: eg.torrents,
  })),
})

const sourceTitleGroupHierarchy = computed(() => toHierarchyLite(props.sourceData))

const targetTitleGroupHierarchy = computed(() => (targetData.value ? toHierarchyLite(targetData.value) : undefined))

const fetchTarget = () => {
  if (!targetId.value) return
  fetching.value = true
  getTitleGroup(targetId.value)
    .then((data) => {
      targetData.value = data
    })
    .finally(() => (fetching.value = false))
}

const handleMerge = () => {
  if (!targetId.value) return
  merging.value = true
  mergeTitleGroups({ source_title_group_id: props.titleGroupId, target_title_group_id: targetId.value })
    .then(() => {
      showToast('', t('title_group.merge_title_group_success'), 'success', 2000)
      emit('merged', targetId.value!)
    })
    .finally(() => (merging.value = false))
}
</script>

<style scoped>
.merge-dialog {
  display: flex;
  flex-direction: column;
  gap: 15px;
  width: 60vw;
}
.target-input {
  display: flex;
  align-items: center;
  gap: 10px;
}
.preview-section h4 {
  margin-bottom: 5px;
}
.merge-warning {
  color: var(--p-red-400);
  font-weight: 500;
}
</style>
