<template>
  <div class="advent-calendar-door-dialog">
    <template v-if="opening">
      <BBCodeRenderer :content="opening.opening_message" />
      <div class="rewards">
        <template v-if="receivedRewards.length > 0">
          <div class="bold you-received">{{ t('advent_calendar.you_received') }}</div>
          <ul>
            <li v-for="reward in receivedRewards" :key="reward">{{ reward }}</li>
          </ul>
        </template>
        <span v-else>{{ t('advent_calendar.nothing_received') }}</span>
      </div>
    </template>
    <template v-else-if="door.requires_passphrase">
      <BBCodeRenderer v-if="door.locked_message" :content="door.locked_message" />
      <div class="passphrase">
        <FloatLabel>
          <InputText v-model="passphrase" size="small" @keyup.enter="openDoor" autofocus />
          <label>{{ t('advent_calendar.passphrase') }}</label>
        </FloatLabel>
        <Button :label="t('advent_calendar.open')" size="small" :loading @click="openDoor" />
      </div>
    </template>
    <div v-else class="wrapper-center">
      <ProgressSpinner />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, FloatLabel, InputText, ProgressSpinner } from 'primevue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import { openAdventCalendarDoor, type AdventCalendarDoorForUser, type AdventCalendarDoorOpeningResult } from '@/services/api-schema'
import { useAdventCalendarRewards } from '@/composables/useAdventCalendarRewards'
import { useUserStore } from '@/stores/user'

const { t } = useI18n()
const formatRewards = useAdventCalendarRewards()
const userStore = useUserStore()

const props = defineProps<{
  door: AdventCalendarDoorForUser
}>()

const emit = defineEmits<{
  opened: [AdventCalendarDoorOpeningResult]
  // the error toast is shown by the api client, the parent closes the dialog
  failed: []
}>()

const opening = ref<AdventCalendarDoorOpeningResult | null>(props.door.opening ?? null)
const passphrase = ref('')
const loading = ref(false)

const receivedRewards = computed(() => (opening.value ? formatRewards(opening.value) : []))

const openDoor = () => {
  loading.value = true
  openAdventCalendarDoor({
    door_id: props.door.id,
    AdventCalendarDoorOpeningAttempt: { passphrase: props.door.requires_passphrase ? passphrase.value : null },
  })
    .then((result) => {
      opening.value = result
      userStore.uploaded += result.upload
      userStore.downloaded = Math.max(userStore.downloaded - result.download_removed, 0)
      userStore.bonus_points += result.bonus_points
      userStore.freeleech_tokens += result.freeleech_tokens
      userStore.invitations += result.invitations
      emit('opened', result)
    })
    .catch(() => {
      // a wrong passphrase can be retried, any other error means the door can't be opened from here
      if (!props.door.requires_passphrase) emit('failed')
    })
    .finally(() => {
      loading.value = false
    })
}

onMounted(() => {
  if (!opening.value && !props.door.requires_passphrase) {
    openDoor()
  }
})
</script>

<style scoped>
.advent-calendar-door-dialog {
  display: flex;
  flex-direction: column;
  gap: 15px;
  min-width: 20em;
  max-width: 40em;
}
.you-received {
  text-align: center;
  margin-top: 30px;
}
.passphrase {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding-top: 10px;
}
</style>
