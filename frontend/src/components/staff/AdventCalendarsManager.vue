<template>
  <div class="advent-calendars-manager">
    <div class="wrapper-center" style="margin-bottom: 20px">
      <i class="pi pi-plus cursor-pointer" v-tooltip.top="t('advent_calendar.add_new')" @click="openEditor()" />
    </div>
    <DataTable :value="calendars" size="small" dataKey="id">
      <Column :header="t('general.name')" field="name" />
      <Column :header="t('advent_calendar.starts_at')">
        <template #body="slotProps"><TimeAgo :date="slotProps.data.starts_at" /></template>
      </Column>
      <Column :header="t('advent_calendar.ends_at')">
        <template #body="slotProps"><TimeAgo :date="calendarEnd(slotProps.data)" /></template>
      </Column>
      <Column :header="t('advent_calendar.door', 2)" field="door_count" />
      <Column :header="t('advent_calendar.published')">
        <template #body="slotProps">
          <i :class="slotProps.data.published ? 'pi pi-check' : 'pi pi-times'" />
        </template>
      </Column>
      <Column :header="t('general.action', 2)">
        <template #body="slotProps">
          <i class="pi pi-pen-to-square cursor-pointer" v-tooltip.top="t('general.edit')" @click="openEditor(slotProps.data)" style="margin-right: 10px" />
          <i class="pi pi-trash cursor-pointer" v-tooltip.top="t('general.delete')" @click="calendarBeingDeleted = slotProps.data" style="margin-right: 10px" />
          <i class="pi pi-replay cursor-pointer" v-tooltip.top="t('advent_calendar.reset_my_progress')" @click="resetMyProgress(slotProps.data)" />
        </template>
      </Column>
    </DataTable>

    <!-- no closeOnEscape: escape is meant for the door dialog opened inside, and would discard the unsaved calendar -->
    <Dialog
      :closeOnEscape="false"
      modal
      :header="calendarBeingEdited ? t('advent_calendar.edit') : t('advent_calendar.add_new')"
      v-model:visible="editorVisible"
      :style="{ width: '60em' }"
    >
      <AdventCalendarEditor :initialCalendar="calendarBeingEdited" @saved="onCalendarSaved" />
    </Dialog>
    <Dialog
      closeOnEscape
      modal
      :header="t('advent_calendar.delete')"
      :visible="calendarBeingDeleted !== undefined"
      @update:visible="calendarBeingDeleted = undefined"
    >
      <DeleteDialog
        v-if="calendarBeingDeleted"
        :message="t('advent_calendar.confirm_delete')"
        :action="() => deleteAdventCalendar(calendarBeingDeleted!.id)"
        :successMessage="t('advent_calendar.deleted')"
        @deleted="onCalendarDeleted"
      />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { Dialog } from 'primevue'
import AdventCalendarEditor from '@/components/staff/AdventCalendarEditor.vue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import {
  listAdventCalendars,
  getAdventCalendar,
  deleteAdventCalendar,
  resetOwnAdventCalendarProgress,
  type AdventCalendar,
  type AdventCalendarLite,
} from '@/services/api-schema'
import { showToast } from '@/main'
import TimeAgo from '@/components/TimeAgo.vue'

const { t } = useI18n()

const calendars = ref<AdventCalendarLite[]>([])
const editorVisible = ref(false)
const calendarBeingEdited = ref<AdventCalendar | undefined>()
const calendarBeingDeleted = ref<AdventCalendarLite | undefined>()

const calendarEnd = (calendar: AdventCalendarLite) => new Date(new Date(calendar.starts_at).getTime() + calendar.total_duration_minutes * 60_000).toISOString()

const fetchCalendars = () => {
  listAdventCalendars().then((data) => {
    calendars.value = data
  })
}

const openEditor = (calendar?: AdventCalendarLite) => {
  if (!calendar) {
    calendarBeingEdited.value = undefined
    editorVisible.value = true
    return
  }
  getAdventCalendar(calendar.id).then((data) => {
    calendarBeingEdited.value = data
    editorVisible.value = true
  })
}

const onCalendarSaved = () => {
  editorVisible.value = false
  fetchCalendars()
}

const onCalendarDeleted = () => {
  calendars.value = calendars.value.filter((calendar) => calendar.id !== calendarBeingDeleted.value?.id)
  calendarBeingDeleted.value = undefined
}

const resetMyProgress = (calendar: AdventCalendarLite) => {
  resetOwnAdventCalendarProgress(calendar.id).then(() => {
    showToast('Success', t('advent_calendar.my_progress_reset'), 'success', 2000)
  })
}

onMounted(() => {
  fetchCalendars()
})
</script>
