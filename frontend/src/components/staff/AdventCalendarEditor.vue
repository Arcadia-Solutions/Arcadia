<template>
  <div class="advent-calendar-editor">
    <div class="row">
      <FloatLabel>
        <InputText v-model="calendar.name" size="small" />
        <label>{{ t('general.name') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputText v-model="calendar.button_text" size="small" :maxlength="50" />
        <label>{{ t('advent_calendar.button_text') }}</label>
      </FloatLabel>
      <div class="checkbox-row">
        <Checkbox v-model="calendar.published" binary inputId="advent_calendar_published" />
        <label for="advent_calendar_published">{{ t('advent_calendar.published') }}</label>
      </div>
    </div>
    <div class="row dates-row">
      <FloatLabel>
        <DatePicker v-model="startsAt" showTime hourFormat="24" size="small" />
        <label>{{ t('advent_calendar.starts_at') }}</label>
      </FloatLabel>
      <span>{{ t('advent_calendar.ends_at') }}: {{ formatDate(endsAt) }}</span>
    </div>
    <div class="row">
      <FloatLabel>
        <Select
          v-model="calendar.late_opening_policy"
          :options="[
            { value: AdventCalendarLateOpeningPolicy.DuringDoorWindow, label: t('advent_calendar.late_opening_policy_during_door_window') },
            { value: AdventCalendarLateOpeningPolicy.GracePeriod, label: t('advent_calendar.late_opening_policy_grace_period') },
            { value: AdventCalendarLateOpeningPolicy.UntilEventEnd, label: t('advent_calendar.late_opening_policy_until_event_end') },
          ]"
          optionLabel="label"
          optionValue="value"
          size="small"
        />
        <label>{{ t('advent_calendar.late_opening_policy') }}</label>
      </FloatLabel>
      <FloatLabel v-if="calendar.late_opening_policy === AdventCalendarLateOpeningPolicy.GracePeriod">
        <InputNumber v-model="calendar.late_opening_grace_hours" :min="0" size="small" />
        <label>{{ t('advent_calendar.late_opening_grace_hours') }}</label>
      </FloatLabel>
    </div>
    <BBCodeEditor :label="t('advent_calendar.top_message')" :initialValue="calendar.top_message" @valueChange="calendar.top_message = $event" />
    <FloatLabel class="staff-notes">
      <Textarea v-model="calendar.staff_notes" class="staff-notes" rows="3" autoResize />
      <label>{{ t('advent_calendar.staff_notes') }}</label>
    </FloatLabel>

    <h3>{{ t('advent_calendar.door', 2) }}</h3>
    <div class="doors">
      <Button
        v-for="(door, index) in calendar.doors"
        :key="doorKey(door)"
        :label="door.name"
        size="small"
        severity="secondary"
        @click="doorBeingEditedIndex = index"
      />
      <Button :label="t('advent_calendar.add_door')" icon="pi pi-plus" size="small" @click="addDoor" />
    </div>
    <Dialog
      closeOnEscape
      modal
      :header="doorBeingEdited?.name"
      :visible="doorBeingEdited !== undefined"
      :style="{ width: '55em' }"
      @update:visible="doorBeingEditedIndex = null"
    >
      <template v-if="doorBeingEdited && doorBeingEditedIndex !== null">
        <AdventCalendarDoorEditor :key="doorKey(doorBeingEdited)" v-model="calendar.doors[doorBeingEditedIndex]" />
        <div class="door-actions">
          <Button
            :label="t('advent_calendar.move_earlier')"
            icon="pi pi-arrow-left"
            size="small"
            severity="secondary"
            :disabled="doorBeingEditedIndex === 0"
            @click="moveDoor(-1)"
          />
          <Button
            :label="t('advent_calendar.move_later')"
            icon="pi pi-arrow-right"
            iconPos="right"
            size="small"
            severity="secondary"
            :disabled="doorBeingEditedIndex === calendar.doors.length - 1"
            @click="moveDoor(1)"
          />
          <Button :label="t('general.delete')" icon="pi pi-trash" size="small" severity="danger" @click="deleteDoor" />
        </div>
      </template>
    </Dialog>
    <div class="wrapper-center" style="margin-top: 20px">
      <Button :label="t('general.save')" size="small" :loading="loading" @click="save" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Checkbox, DatePicker, Dialog, FloatLabel, InputNumber, InputText, Select, Textarea } from 'primevue'
import { computed, reactive, ref, toRaw } from 'vue'
import { useI18n } from 'vue-i18n'
import BBCodeEditor from '@/components/community/BBCodeEditor.vue'
import AdventCalendarDoorEditor from '@/components/staff/AdventCalendarDoorEditor.vue'
import {
  AdventCalendarLateOpeningPolicy,
  AdventCalendarRewardDistribution,
  createAdventCalendar,
  editAdventCalendar,
  type AdventCalendar,
  type UserCreatedAdventCalendar,
  type UserCreatedAdventCalendarDoor,
} from '@/services/api-schema'
import { formatDate } from '@/services/helpers'
import { showToast } from '@/main'

const { t } = useI18n()

const props = defineProps<{
  initialCalendar?: AdventCalendar
}>()

const emit = defineEmits<{
  saved: [AdventCalendar]
}>()

const newDoor = (position: number): UserCreatedAdventCalendarDoor => ({
  name: t('advent_calendar.day_n', { n: position + 1 }),
  image_url: null,
  duration_minutes: 1440,
  opening_message: '',
  locked_message: null,
  passphrase: null,
  staff_notes: '',
  upload_min: 0,
  upload_max: 0,
  upload_distribution: AdventCalendarRewardDistribution.Linear,
  download_removed_min: 0,
  download_removed_max: 0,
  download_removed_distribution: AdventCalendarRewardDistribution.Linear,
  freeleech_tokens_min: 0,
  freeleech_tokens_max: 0,
  freeleech_tokens_distribution: AdventCalendarRewardDistribution.Linear,
  invitations_min: 0,
  invitations_max: 0,
  invitations_distribution: AdventCalendarRewardDistribution.Linear,
  bonus_points_min: 0,
  bonus_points_max: 0,
  bonus_points_distribution: AdventCalendarRewardDistribution.Linear,
})

const calendar = reactive<UserCreatedAdventCalendar>(
  props.initialCalendar
    ? {
        name: props.initialCalendar.name,
        starts_at: props.initialCalendar.starts_at,
        top_message: props.initialCalendar.top_message,
        button_text: props.initialCalendar.button_text,
        staff_notes: props.initialCalendar.staff_notes,
        late_opening_policy: props.initialCalendar.late_opening_policy,
        late_opening_grace_hours: props.initialCalendar.late_opening_grace_hours,
        published: props.initialCalendar.published,
        // the position is left out, the backend uses the index of each door in the list
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        doors: props.initialCalendar.doors.map(({ position, ...door }) => door),
      }
    : {
        name: '',
        starts_at: new Date().toISOString(),
        top_message: '',
        button_text: t('advent_calendar.advent_calendar'),
        staff_notes: '',
        late_opening_policy: AdventCalendarLateOpeningPolicy.DuringDoorWindow,
        late_opening_grace_hours: 0,
        published: false,
        doors: [newDoor(0)],
      },
)

const loading = ref(false)

const startsAt = computed({
  get: () => new Date(calendar.starts_at),
  set: (value: Date) => {
    calendar.starts_at = value.toISOString()
  },
})

const endsAt = computed(() => {
  const totalMinutes = calendar.doors.reduce((sum, door) => sum + door.duration_minutes, 0)
  return new Date(startsAt.value.getTime() + totalMinutes * 60_000).toISOString()
})

// doors without an id still need a stable key, so that reordering keeps each editor's state
const doorKeys = new WeakMap<UserCreatedAdventCalendarDoor, number>()
let nextDoorKey = 0
const doorKey = (door: UserCreatedAdventCalendarDoor) => {
  const rawDoor = toRaw(door)
  if (!doorKeys.has(rawDoor)) {
    doorKeys.set(rawDoor, nextDoorKey++)
  }
  return doorKeys.get(rawDoor)
}

const doorBeingEditedIndex = ref<number | null>(null)
const doorBeingEdited = computed(() => (doorBeingEditedIndex.value === null ? undefined : calendar.doors[doorBeingEditedIndex.value]))

const addDoor = () => {
  calendar.doors.push(newDoor(calendar.doors.length))
  doorBeingEditedIndex.value = calendar.doors.length - 1
}

const moveDoor = (offset: number) => {
  if (doorBeingEditedIndex.value === null) return
  const [door] = calendar.doors.splice(doorBeingEditedIndex.value, 1)
  doorBeingEditedIndex.value += offset
  calendar.doors.splice(doorBeingEditedIndex.value, 0, door)
}

const deleteDoor = () => {
  if (doorBeingEditedIndex.value === null) return
  calendar.doors.splice(doorBeingEditedIndex.value, 1)
  doorBeingEditedIndex.value = null
}

const save = () => {
  loading.value = true
  const request = props.initialCalendar
    ? editAdventCalendar({ id: props.initialCalendar.id, UserCreatedAdventCalendar: calendar })
    : createAdventCalendar(calendar)
  request
    .then((saved) => {
      showToast('Success', t('advent_calendar.saved'), 'success', 2000)
      emit('saved', saved)
    })
    .finally(() => {
      loading.value = false
    })
}
</script>

<style scoped>
.advent-calendar-editor {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding-top: 10px;
}
.row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 15px;
}
/* lines up the end date text with the text inside the start date input */
.dates-row {
  align-items: last baseline;
}
.staff-notes {
  width: 100%;
}
.checkbox-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.doors {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.door-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-top: 20px;
}
</style>
