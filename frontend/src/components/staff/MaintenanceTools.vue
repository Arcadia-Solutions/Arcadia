<template>
  <div class="maintenance-tools">
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.rehash_torrents') }}</h3>
        <p>{{ t('maintenance_tools.rehash_torrents_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="rehashing" @click="rehashTorrents" />
    </div>
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.recompute_cached_amounts') }}</h3>
        <p>{{ t('maintenance_tools.recompute_cached_amounts_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="recomputingCachedAmounts" @click="recomputeCachedAmountsTool" />
    </div>
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.check_missing_registration_ips') }}</h3>
        <p>{{ t('maintenance_tools.check_missing_registration_ips_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="checkingRegistrationIps" @click="checkMissingRegistrationIpsTool" />
    </div>
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.delete_empty_title_groups') }}</h3>
        <p>{{ t('maintenance_tools.delete_empty_title_groups_description') }}</p>
        <div class="tool-options">
          <DatePicker
            v-model="emptyTitleGroupsDates"
            selectionMode="range"
            :manualInput="false"
            showButtonBar
            size="small"
            :placeholder="t('maintenance_tools.creation_date_range')"
          />
          <div class="checkbox">
            <Checkbox v-model="listEmptyTitleGroups" inputId="list_empty_title_groups" binary />
            <label for="list_empty_title_groups">{{ t('maintenance_tools.show_title_groups_to_delete') }}</label>
          </div>
        </div>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="fetchingEmptyTitleGroups" @click="openDeleteEmptyTitleGroupsDialog" />
    </div>
    <Dialog modal :header="t('maintenance_tools.delete_empty_title_groups')" v-model:visible="deleteEmptyTitleGroupsDialogVisible">
      <div class="delete-empty-title-groups">
        <template v-if="listEmptyTitleGroups">
          <div class="empty-title-groups">
            <LazyTitleGroupPreviewTable
              v-for="titleGroup in emptyTitleGroups"
              :key="titleGroup.id"
              :title_group="titleGroup"
              :class="{ deselected: !selectedEmptyTitleGroupIds.has(titleGroup.id) }"
              @click="toggleEmptyTitleGroup($event, titleGroup.id)"
            />
          </div>
          <p>{{ t('maintenance_tools.selected_title_groups', { selected: selectedEmptyTitleGroupIds.size, total: emptyTitleGroups.length }) }}</p>
        </template>
        <p v-else>{{ t('maintenance_tools.title_groups_to_delete', { amount: emptyTitleGroupsAmount }) }}</p>
        <DeleteDialog
          :message="t('maintenance_tools.confirm_delete_empty_title_groups')"
          :action="deleteEmptyTitleGroupsTool"
          @deleted="deleteEmptyTitleGroupsDialogVisible = false"
        />
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Checkbox, DatePicker, Dialog } from 'primevue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import LazyTitleGroupPreviewTable from '@/components/title_group/LazyTitleGroupPreviewTable.vue'
import {
  rehashTorrentsWithSourceTag,
  recomputeCachedAmounts,
  checkMissingRegistrationIps,
  getEmptyTitleGroups,
  deleteEmptyTitleGroups,
  type TitleGroupHierarchyLite,
} from '@/services/api-schema'
import { showToast } from '@/main'

const { t } = useI18n()

const rehashing = ref(false)
const recomputingCachedAmounts = ref(false)
const checkingRegistrationIps = ref(false)
const emptyTitleGroupsDates = ref<(Date | null)[] | null>(null)
const listEmptyTitleGroups = ref(false)
const emptyTitleGroups = ref<TitleGroupHierarchyLite[]>([])
const emptyTitleGroupsAmount = ref(0)
const selectedEmptyTitleGroupIds = ref(new Set<number>())
const fetchingEmptyTitleGroups = ref(false)
const deleteEmptyTitleGroupsDialogVisible = ref(false)

const rehashTorrents = () => {
  rehashing.value = true
  // No timeout: rehashing every torrent can take a long time.
  rehashTorrentsWithSourceTag({ timeout: 0 })
    .then((data) => {
      showToast('Success', t('maintenance_tools.rehash_torrents_success', { amount: data.updated_torrents_amount }), 'success', 5000)
    })
    .finally(() => {
      rehashing.value = false
    })
}

const recomputeCachedAmountsTool = () => {
  recomputingCachedAmounts.value = true
  // No timeout: recomputing every cached count can take a long time.
  recomputeCachedAmounts({ timeout: 0 })
    .then(() => {
      showToast('Success', t('maintenance_tools.recompute_cached_amounts_success'), 'success', 5000)
    })
    .finally(() => {
      recomputingCachedAmounts.value = false
    })
}

const checkMissingRegistrationIpsTool = () => {
  checkingRegistrationIps.value = true
  // No timeout: every check is a request to a provider.
  checkMissingRegistrationIps({ timeout: 0 })
    .then((data) => {
      showToast('Success', t('maintenance_tools.check_missing_registration_ips_success', { amount: data.checked_users_amount }), 'success', 5000)
    })
    .finally(() => {
      checkingRegistrationIps.value = false
    })
}

// Picking a single date in the range covers that whole day, the end date is included
const emptyTitleGroupsCreationRange = () => {
  const [start, end] = emptyTitleGroupsDates.value ?? []
  const last = end ?? start
  return {
    created_after: start?.toISOString(),
    created_before: last ? new Date(last.getFullYear(), last.getMonth(), last.getDate() + 1).toISOString() : undefined,
  }
}

const openDeleteEmptyTitleGroupsDialog = () => {
  fetchingEmptyTitleGroups.value = true
  getEmptyTitleGroups({ ...emptyTitleGroupsCreationRange(), count_only: !listEmptyTitleGroups.value })
    .then((data) => {
      emptyTitleGroupsAmount.value = data.amount
      emptyTitleGroups.value = data.title_groups
      selectedEmptyTitleGroupIds.value = new Set(data.title_groups.map((titleGroup) => titleGroup.id))
      deleteEmptyTitleGroupsDialogVisible.value = true
    })
    .finally(() => {
      fetchingEmptyTitleGroups.value = false
    })
}

const toggleEmptyTitleGroup = (event: MouseEvent, titleGroupId: number) => {
  // Clicking a link (name, artist, tag...) follows it instead
  if ((event.target as HTMLElement).closest('a')) return
  if (!selectedEmptyTitleGroupIds.value.delete(titleGroupId)) {
    selectedEmptyTitleGroupIds.value.add(titleGroupId)
  }
}

const deleteEmptyTitleGroupsTool = () =>
  deleteEmptyTitleGroups(
    {
      ...emptyTitleGroupsCreationRange(),
      title_group_ids: listEmptyTitleGroups.value ? [...selectedEmptyTitleGroupIds.value] : undefined,
    },
    // No timeout: there can be a lot of title groups to delete.
    { timeout: 0 },
  ).then((data) => {
    showToast('Success', t('maintenance_tools.delete_empty_title_groups_success', { amount: data.deleted_title_groups_amount }), 'success', 5000)
  })
</script>

<style scoped>
.maintenance-tools {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.tool {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 15px;
  border: 1px solid var(--color-border);
  border-radius: 5px;
}
.tool-info h3 {
  margin: 0 0 5px;
}
.tool-info p {
  margin: 0;
  opacity: 0.8;
}
.tool-options {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 15px;
  margin-top: 10px;
}
.checkbox {
  display: flex;
  align-items: center;
  gap: 5px;
}
.delete-empty-title-groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: min(60vw, 100%);
}
.empty-title-groups {
  max-height: 60vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.empty-title-groups > * {
  cursor: pointer;
  border: 2px solid var(--p-green-500);
  border-radius: 12px;
}
.empty-title-groups .deselected {
  border-color: var(--p-red-500);
}
/* only the content is greyed out, so the red border stays visible */
.empty-title-groups .deselected > * {
  opacity: 0.3;
}
</style>
