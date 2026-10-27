<template>
  <div class="advent-calendar-door-editor">
    <div class="row">
      <FloatLabel>
        <InputText v-model="door.name" size="small" />
        <label>{{ t('general.name') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputText :modelValue="door.image_url ?? ''" @update:modelValue="door.image_url = $event || null" size="small" />
        <label>{{ t('advent_calendar.image_url') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputText :modelValue="door.passphrase ?? ''" @update:modelValue="door.passphrase = $event || null" size="small" />
        <label>{{ t('advent_calendar.passphrase_optional') }}</label>
      </FloatLabel>
    </div>
    <div class="row">
      <FloatLabel>
        <InputNumber v-model="durationHours" :min="0" size="small" suffix=" h" />
        <label>{{ t('advent_calendar.duration') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputNumber v-model="durationMinutes" :min="0" :max="59" size="small" suffix=" min" />
        <label>&nbsp;</label>
      </FloatLabel>
    </div>
    <BBCodeEditor :label="t('advent_calendar.opening_message')" :initialValue="door.opening_message" :rows="3" @valueChange="door.opening_message = $event" />
    <BBCodeEditor
      :label="t('advent_calendar.locked_message')"
      :initialValue="door.locked_message ?? ''"
      :rows="2"
      @valueChange="door.locked_message = $event || null"
    />
    <FloatLabel class="staff-notes">
      <Textarea v-model="door.staff_notes" class="staff-notes" rows="3" autoResize />
      <label>{{ t('advent_calendar.staff_notes') }}</label>
    </FloatLabel>

    <div class="rewards">
      <div v-for="rewardName in rewardNames" :key="rewardName" class="reward-row">
        <div class="checkbox-row reward-type">
          <Checkbox
            :modelValue="door[maxKey(rewardName)] > 0"
            binary
            :inputId="`reward_${rewardName}_${uniqueId}`"
            @update:modelValue="toggleReward(rewardName, $event)"
          />
          <label :for="`reward_${rewardName}_${uniqueId}`">{{ rewardLabel(rewardName) }}</label>
        </div>
        <template v-if="door[maxKey(rewardName)] > 0">
          <FloatLabel>
            <InputNumber
              :modelValue="rawToDisplay(rewardName, door[minKey(rewardName)])"
              @update:modelValue="door[minKey(rewardName)] = displayToRaw(rewardName, $event ?? 0)"
              :min="0"
              :maxFractionDigits="maxFractionDigits(rewardName)"
              size="small"
            />
            <label>{{ t('advent_calendar.min') }}</label>
          </FloatLabel>
          <FloatLabel>
            <InputNumber
              :modelValue="rawToDisplay(rewardName, door[maxKey(rewardName)])"
              @update:modelValue="door[maxKey(rewardName)] = displayToRaw(rewardName, $event ?? 0)"
              :min="0"
              :maxFractionDigits="maxFractionDigits(rewardName)"
              size="small"
            />
            <label>{{ t('advent_calendar.max') }}</label>
          </FloatLabel>
          <Select v-if="isBytesReward(rewardName)" v-model="byteUnits[rewardName]" :options="['MiB', 'GiB', 'TiB']" size="small" />
          <Select
            v-model="door[distributionKey(rewardName)]"
            :options="[
              { value: AdventCalendarRewardDistribution.Linear, label: t('advent_calendar.distribution_linear') },
              { value: AdventCalendarRewardDistribution.Gaussian, label: t('advent_calendar.distribution_gaussian') },
              { value: AdventCalendarRewardDistribution.Logarithmic, label: t('advent_calendar.distribution_logarithmic') },
            ]"
            optionLabel="label"
            optionValue="value"
            size="small"
          />
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Checkbox, FloatLabel, InputNumber, InputText, Select, Textarea } from 'primevue'
import { computed, reactive, useId } from 'vue'
import { useI18n } from 'vue-i18n'
import BBCodeEditor from '@/components/community/BBCodeEditor.vue'
import { AdventCalendarRewardDistribution, type UserCreatedAdventCalendarDoor } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const uniqueId = useId()

const door = defineModel<UserCreatedAdventCalendarDoor>({ required: true })

const durationHours = computed({
  get: () => Math.floor(door.value.duration_minutes / 60),
  set: (hours: number) => {
    door.value.duration_minutes = (hours ?? 0) * 60 + (door.value.duration_minutes % 60)
  },
})
const durationMinutes = computed({
  get: () => door.value.duration_minutes % 60,
  set: (minutes: number) => {
    door.value.duration_minutes = Math.floor(door.value.duration_minutes / 60) * 60 + (minutes ?? 0)
  },
})

const rewardNames = ['upload', 'download_removed', 'freeleech_tokens', 'invitations', 'bonus_points'] as const
type RewardName = (typeof rewardNames)[number]
const minKey = (rewardName: RewardName) => `${rewardName}_min` as const
const maxKey = (rewardName: RewardName) => `${rewardName}_max` as const
const distributionKey = (rewardName: RewardName) => `${rewardName}_distribution` as const

const bytesPerUnit = { MiB: 1024 ** 2, GiB: 1024 ** 3, TiB: 1024 ** 4 }
type ByteUnit = keyof typeof bytesPerUnit
type BytesRewardName = 'upload' | 'download_removed'
const byteUnits = reactive<Record<BytesRewardName, ByteUnit>>({ upload: 'GiB', download_removed: 'GiB' })

const isBytesReward = (rewardName: RewardName): rewardName is BytesRewardName => rewardName === 'upload' || rewardName === 'download_removed'

// how many raw units one displayed unit is worth
const unitMultiplier = (rewardName: RewardName) => {
  if (isBytesReward(rewardName)) return bytesPerUnit[byteUnits[rewardName]]
  if (rewardName === 'bonus_points') return 10 ** publicArcadiaSettings.bonus_points_decimal_places
  return 1
}
const maxFractionDigits = (rewardName: RewardName) => {
  if (isBytesReward(rewardName)) return 3
  if (rewardName === 'bonus_points') return publicArcadiaSettings.bonus_points_decimal_places
  return 0
}
const rawToDisplay = (rewardName: RewardName, raw: number) => raw / unitMultiplier(rewardName)
const displayToRaw = (rewardName: RewardName, display: number) => Math.round(display * unitMultiplier(rewardName))

const rewardLabel = (rewardName: RewardName) => {
  if (rewardName === 'bonus_points') return publicArcadiaSettings.bonus_points_alias
  return t(`advent_calendar.reward_${rewardName}`)
}

// a reward with a maximum of 0 is disabled
const toggleReward = (rewardName: RewardName, enabled: boolean) => {
  const amount = enabled ? unitMultiplier(rewardName) : 0
  door.value[minKey(rewardName)] = amount
  door.value[maxKey(rewardName)] = amount
  door.value[distributionKey(rewardName)] = AdventCalendarRewardDistribution.Linear
}
</script>

<style scoped>
.advent-calendar-door-editor {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.row,
.reward-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 15px;
}
.rewards {
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.staff-notes {
  width: 100%;
}
.checkbox-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.reward-type {
  width: 12em;
}
</style>
