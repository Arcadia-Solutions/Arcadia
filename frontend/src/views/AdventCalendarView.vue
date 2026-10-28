<template>
  <div id="advent-calendar" v-if="calendar">
    <ContentContainer>
      <h2 class="calendar-name">{{ calendar.name }}</h2>
      <BBCodeRenderer v-if="calendar.top_message" :content="calendar.top_message" class="top-message" />
      <div class="doors">
        <div
          v-for="door in calendar.doors"
          :key="door.id"
          :class="['door', `door-${door.state}`, { clickable: isClickable(door), 'with-image': door.image_url }]"
          :style="door.image_url ? { backgroundImage: `url(${JSON.stringify(door.image_url)})` } : undefined"
          @click="isClickable(door) && (doorBeingOpened = door)"
        >
          <span class="door-name">{{ door.name }}</span>
          <span v-if="door.state === AdventCalendarDoorState.Locked" class="door-status"> <i class="pi pi-lock" /> <TimeAgo :date="door.starts_at" /> </span>
          <span v-else-if="door.state === AdventCalendarDoorState.Missed" class="door-status">{{ t('advent_calendar.missed') }}</span>
          <i v-else-if="door.state === AdventCalendarDoorState.Opened" class="pi pi-check door-status" />
          <i v-else class="pi pi-gift door-status" />
        </div>
      </div>
    </ContentContainer>
    <Dialog closeOnEscape modal :header="doorBeingOpened?.name" :visible="doorBeingOpened !== null" @update:visible="doorBeingOpened = null">
      <AdventCalendarDoorDialog v-if="doorBeingOpened" :door="doorBeingOpened" @opened="onDoorOpened" @failed="onDoorFailed" />
    </Dialog>
  </div>
  <div v-else-if="notFound" class="wrapper-center">{{ t('advent_calendar.no_calendar') }}</div>
</template>

<script setup lang="ts">
import { Dialog } from 'primevue'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import AdventCalendarDoorDialog from '@/components/advent_calendar/AdventCalendarDoorDialog.vue'
import {
  AdventCalendarDoorState,
  getPublishedAdventCalendar,
  type AdventCalendarDoorForUser,
  type AdventCalendarDoorOpeningResult,
  type PublishedAdventCalendar,
} from '@/services/api-schema'

const { t } = useI18n()

const calendar = ref<PublishedAdventCalendar | null>(null)
const notFound = ref(false)
const doorBeingOpened = ref<AdventCalendarDoorForUser | null>(null)

const isClickable = (door: AdventCalendarDoorForUser) => door.state === AdventCalendarDoorState.Openable || door.state === AdventCalendarDoorState.Opened

const onDoorOpened = (result: AdventCalendarDoorOpeningResult) => {
  const door = calendar.value?.doors.find((door) => door.id === doorBeingOpened.value?.id)
  if (door) {
    door.state = AdventCalendarDoorState.Opened
    door.opening = result
  }
}

const onDoorFailed = () => {
  doorBeingOpened.value = null
  fetchCalendar()
}

const fetchCalendar = () => {
  getPublishedAdventCalendar()
    .then((data) => {
      calendar.value = data
    })
    .catch(() => {
      notFound.value = true
    })
}

onMounted(fetchCalendar)
</script>

<style scoped>
.calendar-name {
  text-align: center;
  font-size: 2em;
  font-weight: bold;
  margin-top: 0;
}
.top-message {
  margin-bottom: 20px;
}
.doors {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 15px;
}
.door {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  aspect-ratio: 1;
  padding: 10px;
  border-radius: 7px;
  background-color: var(--color-background-primary);
  text-align: center;
  background-size: cover;
  background-position: center;
}
.door.with-image .door-name,
.door.with-image .door-status {
  padding: 3px 8px;
  border-radius: 5px;
  background-color: rgba(0, 0, 0, 0.65);
  color: white;
}
.door.clickable {
  cursor: pointer;
}
.door.clickable:hover {
  outline: 2px solid var(--color-primary);
}
.door-locked,
.door-missed {
  opacity: 0.5;
}
.door-openable {
  outline: 1px solid var(--color-primary);
}
.door-name {
  font-weight: bold;
}
.door-status {
  font-size: 0.85em;
}
</style>
