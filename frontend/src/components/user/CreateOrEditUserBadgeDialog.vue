<template>
  <div class="user-badge-dialog">
    <FloatLabel>
      <InputText name="name" v-model="badge.name" style="width: 100%" />
      <label>{{ t('general.name') }}</label>
    </FloatLabel>

    <FloatLabel>
      <Textarea name="description" v-model="badge.description" rows="3" style="width: 100%" />
      <label>{{ t('general.description') }}</label>
    </FloatLabel>

    <FloatLabel>
      <InputText name="image_url" v-model="badge.image_url" style="width: 100%" />
      <label>{{ t('user_badge.image_url') }}</label>
    </FloatLabel>
    <img v-if="badge.image_url" :src="badge.image_url" class="badge-preview" alt="badge preview" />

    <FloatLabel>
      <Select v-model="badge.category_id" :options="categories" optionLabel="name" optionValue="id" size="small" style="width: 100%" />
      <label>{{ t('user_badge.category') }}</label>
    </FloatLabel>

    <FloatLabel>
      <Select
        v-model="badge.badge_type"
        :options="[
          { value: UserBadgeType.Manual, label: t('user_badge.type_manual') },
          { value: UserBadgeType.TorrentsUploaded, label: t('user_badge.type_torrents_uploaded') },
          { value: UserBadgeType.ForumPosts, label: t('user_badge.type_forum_posts') },
          { value: UserBadgeType.ForumThreads, label: t('user_badge.type_forum_threads') },
          { value: UserBadgeType.AdventCalendarDoors, label: t('user_badge.type_advent_calendar_doors') },
        ]"
        optionLabel="label"
        optionValue="value"
        size="small"
        style="width: 100%"
        @update:modelValue="onTypeChanged"
      />
      <label>{{ t('user_badge.badge_type') }}</label>
    </FloatLabel>

    <div class="checkbox-group">
      <div class="checkbox-item">
        <Checkbox v-model="badge.is_secret" :binary="true" inputId="is_secret" />
        <label for="is_secret">
          {{ t('user_badge.is_secret') }}
          <span class="hint">— {{ t('user_badge.is_secret_help') }}</span>
        </label>
      </div>
      <div class="checkbox-item" v-if="badge.badge_type !== UserBadgeType.Manual">
        <Checkbox v-model="badge.revoke_when_criteria_unmet" :binary="true" inputId="revoke_when_criteria_unmet" />
        <label for="revoke_when_criteria_unmet">{{ t('user_badge.revoke_when_criteria_unmet') }}</label>
      </div>
    </div>

    <template v-if="badge.badge_type === UserBadgeType.Manual">
      <p class="hint">{{ t('user_badge.no_criteria_for_manual') }}</p>
    </template>

    <template v-else-if="badge.badge_type === UserBadgeType.TorrentsUploaded && torrentsUploadedCriteria">
      <h3>{{ t('user_badge.criteria') }}</h3>
      <FloatLabel>
        <InputNumber v-model="torrentsUploadedCriteria.minimum_title_group_amount" :min="1" />
        <label>{{ t('user_badge.minimum_title_group_amount') }}</label>
      </FloatLabel>
      <TorrentSearchInputs :loading="false" :initialForm="torrentsUploadedCriteria.search" displaySearchButton />
    </template>

    <template v-else-if="badge.badge_type === UserBadgeType.ForumPosts && forumPostsCriteria">
      <h3>{{ t('user_badge.criteria') }}</h3>
      <FloatLabel>
        <InputNumber v-model="forumPostsCriteria.minimum_post_amount" :min="1" />
        <label>{{ t('user_badge.minimum_post_amount') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputNumber v-model="forumPostsCriteria.minimum_post_character_count" :min="0" />
        <label>{{ t('user_badge.minimum_post_character_count') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputText v-model="forumPostsSubstringProxy" style="width: 100%" />
        <label>{{ t('user_badge.required_substring') }}</label>
      </FloatLabel>
    </template>

    <template v-else-if="badge.badge_type === UserBadgeType.ForumThreads && forumThreadsCriteria">
      <h3>{{ t('user_badge.criteria') }}</h3>
      <FloatLabel>
        <InputNumber v-model="forumThreadsCriteria.minimum_thread_amount" :min="1" />
        <label>{{ t('user_badge.minimum_thread_amount') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputNumber v-model="forumThreadsCriteria.minimum_thread_name_character_count" :min="0" />
        <label>{{ t('user_badge.minimum_thread_name_character_count') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputText v-model="forumThreadsSubstringProxy" style="width: 100%" />
        <label>{{ t('user_badge.required_substring') }}</label>
      </FloatLabel>
    </template>

    <template v-else-if="badge.badge_type === UserBadgeType.AdventCalendarDoors && adventCalendarDoorsCriteria">
      <h3>{{ t('user_badge.criteria') }}</h3>
      <FloatLabel>
        <Select
          v-model="adventCalendarDoorsCriteria.calendar_id"
          :options="adventCalendars"
          optionLabel="name"
          optionValue="id"
          size="small"
          style="width: 100%"
        />
        <label>{{ t('user_badge.advent_calendar') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputNumber v-model="adventCalendarDoorsCriteria.minimum_opened_door_amount" :min="1" />
        <label>{{ t('user_badge.minimum_opened_door_amount') }}</label>
      </FloatLabel>
      <FloatLabel>
        <MultiSelect v-model="adventCalendarDoorsCriteria.door_names" :options="adventCalendarDoorNames" filter size="small" style="width: 100%" />
        <label>{{ t('user_badge.door_names') }}</label>
      </FloatLabel>
    </template>

    <div class="wrapper-center" style="margin-top: 20px">
      <Button :label="t('general.confirm')" size="small" :loading="loading" :disabled="adventCalendarDoorsCriteria?.calendar_id === 0" @click="save()" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { FloatLabel, InputText, InputNumber, Textarea, Checkbox, Button, Select, MultiSelect } from 'primevue'
import { ref, computed, onMounted, toRaw, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  createUserBadge,
  editUserBadge,
  getAdventCalendar,
  listAdventCalendars,
  UserBadgeType,
  UserBadgeCriteriaOneOfTypeEnum,
  UserBadgeCriteriaOneOf1TypeEnum,
  UserBadgeCriteriaOneOf2TypeEnum,
  UserBadgeCriteriaOneOf3TypeEnum,
  type AdventCalendarLite,
  type UserBadge,
  type UserBadgeCategory,
  type UserBadgeCriteriaOneOf,
  type UserBadgeCriteriaOneOf1,
  type UserBadgeCriteriaOneOf2,
  type UserBadgeCriteriaOneOf3,
  type UserCreatedUserBadge,
  type TorrentSearch,
} from '@/services/api-schema'
import { showToast } from '@/main'
import TorrentSearchInputs from '@/components/torrent/TorrentSearchInputs.vue'

const { t } = useI18n()

const props = defineProps<{
  initialBadge?: UserBadge
  categories: UserBadgeCategory[]
}>()

const emit = defineEmits<{
  done: [UserBadge]
}>()

const loading = ref(false)
const isEditMode = computed(() => !!props.initialBadge)

const defaultTorrentSearch = (): TorrentSearch => ({
  title_group_name: null,
  title_group_tags: null,
  title_group_include_empty_groups: false,
  title_group_content_type: [],
  title_group_category: [],
  edition_group_source: [],
  torrent_created_by_id: null,
  torrent_snatched_by_id: null,
  torrent_staff_checked: null,
  torrent_reported: null,
  torrent_trumpable: null,
  torrent_language: [],
  torrent_video_resolution: [],
  page: 1,
  page_size: 25,
  order_by_column: 'torrent_created_at',
  order_by_direction: 'desc',
})

const badge = ref<UserCreatedUserBadge>({
  name: '',
  description: '',
  image_url: '',
  category_id: props.categories[0]?.id ?? 0,
  badge_type: UserBadgeType.Manual,
  is_secret: false,
  revoke_when_criteria_unmet: false,
  criteria: null,
})

const torrentsUploadedCriteria = computed<UserBadgeCriteriaOneOf | null>(() =>
  badge.value.badge_type === UserBadgeType.TorrentsUploaded ? (badge.value.criteria as UserBadgeCriteriaOneOf) : null,
)
const forumPostsCriteria = computed<UserBadgeCriteriaOneOf1 | null>(() =>
  badge.value.badge_type === UserBadgeType.ForumPosts ? (badge.value.criteria as UserBadgeCriteriaOneOf1) : null,
)
const forumThreadsCriteria = computed<UserBadgeCriteriaOneOf2 | null>(() =>
  badge.value.badge_type === UserBadgeType.ForumThreads ? (badge.value.criteria as UserBadgeCriteriaOneOf2) : null,
)
const adventCalendarDoorsCriteria = computed<UserBadgeCriteriaOneOf3 | null>(() =>
  badge.value.badge_type === UserBadgeType.AdventCalendarDoors ? (badge.value.criteria as UserBadgeCriteriaOneOf3) : null,
)

// only fetched when needed, listing the calendars requires the permission to manage them
const adventCalendars = ref<AdventCalendarLite[]>([])
watch(adventCalendarDoorsCriteria, (criteria) => {
  if (criteria && adventCalendars.value.length === 0) {
    listAdventCalendars().then((calendars) => (adventCalendars.value = calendars))
  }
})

const adventCalendarDoorNames = ref<string[]>([])
watch(
  () => adventCalendarDoorsCriteria.value?.calendar_id,
  (calendarId, previousCalendarId) => {
    adventCalendarDoorNames.value = []
    // the names of the previous calendar's doors are invalid, unless the badge is just being loaded for edition
    if (previousCalendarId !== undefined && adventCalendarDoorsCriteria.value) {
      adventCalendarDoorsCriteria.value.door_names = []
    }
    if (calendarId) {
      getAdventCalendar(calendarId).then((calendar) => (adventCalendarDoorNames.value = calendar.doors.map((door) => door.name)))
    }
  },
)

const forumPostsSubstringProxy = computed({
  get: () => forumPostsCriteria.value?.required_substring ?? '',
  set: (v: string) => {
    if (forumPostsCriteria.value) forumPostsCriteria.value.required_substring = v || null
  },
})
const forumThreadsSubstringProxy = computed({
  get: () => forumThreadsCriteria.value?.required_substring ?? '',
  set: (v: string) => {
    if (forumThreadsCriteria.value) forumThreadsCriteria.value.required_substring = v || null
  },
})

const onTypeChanged = (newType: typeof badge.value.badge_type) => {
  if (newType === UserBadgeType.Manual) {
    badge.value.criteria = null
    badge.value.revoke_when_criteria_unmet = false
  } else if (newType === UserBadgeType.TorrentsUploaded) {
    const c: UserBadgeCriteriaOneOf = {
      type: UserBadgeCriteriaOneOfTypeEnum.TorrentsUploaded,
      minimum_title_group_amount: 1,
      search: defaultTorrentSearch(),
    }
    badge.value.criteria = c
  } else if (newType === UserBadgeType.ForumPosts) {
    const c: UserBadgeCriteriaOneOf1 = {
      type: UserBadgeCriteriaOneOf1TypeEnum.ForumPosts,
      minimum_post_amount: 1,
      minimum_post_character_count: 0,
      required_substring: null,
    }
    badge.value.criteria = c
  } else if (newType === UserBadgeType.ForumThreads) {
    const c: UserBadgeCriteriaOneOf2 = {
      type: UserBadgeCriteriaOneOf2TypeEnum.ForumThreads,
      minimum_thread_amount: 1,
      minimum_thread_name_character_count: 0,
      required_substring: null,
    }
    badge.value.criteria = c
  } else if (newType === UserBadgeType.AdventCalendarDoors) {
    const c: UserBadgeCriteriaOneOf3 = {
      type: UserBadgeCriteriaOneOf3TypeEnum.AdventCalendarDoors,
      calendar_id: adventCalendars.value[0]?.id ?? 0,
      minimum_opened_door_amount: 1,
      door_names: [],
    }
    badge.value.criteria = c
  }
}

const save = () => {
  loading.value = true

  const promise = isEditMode.value && props.initialBadge ? editUserBadge({ ...badge.value, id: props.initialBadge.id }) : createUserBadge(badge.value)

  promise
    .then((result) => {
      showToast('', t(isEditMode.value ? 'user_badge.user_badge_edited_success' : 'user_badge.user_badge_created_success'), 'success', 2000)
      emit('done', result)
    })
    .finally(() => {
      loading.value = false
    })
}

onMounted(() => {
  if (props.initialBadge) {
    const raw = toRaw(props.initialBadge)
    badge.value = structuredClone({
      name: raw.name,
      description: raw.description,
      image_url: raw.image_url,
      category_id: raw.category_id,
      badge_type: raw.badge_type,
      is_secret: raw.is_secret,
      revoke_when_criteria_unmet: raw.revoke_when_criteria_unmet,
      criteria: raw.criteria,
    })
  }
})
</script>

<style scoped>
.user-badge-dialog {
  width: 70vw;
  max-width: 900px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.checkbox-item {
  display: flex;
  align-items: center;
  gap: 10px;
}
.checkbox-item label {
  cursor: pointer;
  user-select: none;
}
.hint {
  font-size: 0.875rem;
  opacity: 0.7;
}
.badge-preview {
  max-width: 96px;
  max-height: 96px;
  align-self: center;
  border-radius: 8px;
}
h3 {
  margin: 0;
  font-size: 1.1rem;
}
</style>
