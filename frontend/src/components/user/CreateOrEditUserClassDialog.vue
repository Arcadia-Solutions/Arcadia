<template>
  <div class="user-class-dialog">
    <FloatLabel>
      <InputText name="name" v-model="userClass.name" />
      <label>{{ t('general.name') }}</label>
    </FloatLabel>
    <FloatLabel style="width: 30em">
      <Select
        v-model="userClass.previous_user_class"
        :options="availableClasses.filter((userClass) => userClass.name !== initialUserClass?.name)"
        size="small"
        optionLabel="name"
        optionValue="name"
        style="width: 100%"
      />
      <label>{{ t('user_class.previous_user_class') }}</label>
    </FloatLabel>
    <div class="checkbox-group">
      <div class="checkbox-item">
        <Checkbox v-model="userClass.automatic_promotion" :binary="true" inputId="automatic_promotion" />
        <label for="automatic_promotion">{{ t('user_class.automatic_promotion') }}</label>
      </div>
      <div class="checkbox-item">
        <Checkbox v-model="userClass.automatic_demotion" :binary="true" inputId="automatic_demotion" />
        <label for="automatic_demotion">{{ t('user_class.automatic_demotion') }}</label>
      </div>
      <div class="checkbox-item">
        <Checkbox v-model="userClass.promotion_allowed_while_warned" :binary="true" inputId="promotion_allowed_while_warned" />
        <label for="promotion_allowed_while_warned">{{ t('user_class.promotion_allowed_while_warned') }}</label>
      </div>
    </div>

    <FloatLabel>
      <InputNumber v-model="userClass.max_snatches_per_day" name="max_snatches_per_day" :min="0" />
      <label>{{ t('user_class.max_snatches_per_day') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="displayPromotionCostBonusPoints" name="promotion_cost_bonus_points" :min="0" :step="1" size="small" />
      <label>{{ t('user_class.promotion_cost') }} ({{ publicArcadiaSettings.bonus_points_alias }})</label>
    </FloatLabel>

    <h3>{{ t('user_class.requirements') }}</h3>

    <FloatLabel>
      <InputNumber v-model="userClass.required_uploaded" name="required_uploaded" :min="0" />
      <label>{{ t('user_class.required_uploaded') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_downloaded" name="required_downloaded" :min="0" />
      <label>{{ t('user_class.required_downloaded') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_ratio" name="required_ratio" :min="0" :minFractionDigits="2" :maxFractionDigits="2" />
      <label>{{ t('user_class.required_ratio') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_account_age_in_days" name="required_account_age_in_days" :min="0" />
      <label>{{ t('user_class.required_account_age_in_days') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_forum_posts" name="required_forum_posts" :min="0" />
      <label>{{ t('user_class.required_forum_posts') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_forum_posts_in_unique_threads" name="required_forum_posts_in_unique_threads" :min="0" />
      <label>{{ t('user_class.required_forum_posts_in_unique_threads') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_torrent_uploads" name="required_torrent_uploads" :min="0" />
      <label>{{ t('user_class.required_torrent_uploads') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_torrent_uploads_in_unique_title_groups" name="required_torrent_uploads_in_unique_title_groups" :min="0" />
      <label>{{ t('user_class.required_torrent_uploads_in_unique_title_groups') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_torrent_snatched" name="required_torrent_snatched" :min="0" />
      <label>{{ t('user_class.required_torrent_snatched') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_seeding_size" name="required_seeding_size" :min="0" />
      <label>{{ t('user_class.required_seeding_size') }}</label>
    </FloatLabel>
    <FloatLabel>
      <InputNumber v-model="userClass.required_title_group_comments" name="required_title_group_comments" :min="0" />
      <label>{{ t('user_class.required_title_group_comments') }}</label>
    </FloatLabel>
    <h3 style="margin-top: 30px">{{ t('user_class.new_permissions') }}</h3>
    <FloatLabel>
      <MultiSelect
        v-model="userClass.new_permissions"
        :options="allPermissions"
        optionLabel="label"
        optionValue="value"
        class="permissions-select"
        display="chip"
        filter
      />
      <label>{{ t('user_class.new_permissions') }}</label>
    </FloatLabel>
    <template v-if="canEditForbiddenForumItems">
      <h3 style="margin-top: 30px">{{ t('user_class.hidden_forum_items') }}</h3>
      <ForbiddenForumItemsEditor
        v-if="forbiddenForumItemsLoaded"
        v-model:categoryIds="forbiddenForumItems.category_ids"
        v-model:subCategoryIds="forbiddenForumItems.sub_category_ids"
      />
    </template>

    <div class="wrapper-center" style="margin-top: 30px">
      <Button :label="t('general.confirm')" size="small" :loading="loading" :disabled="!forbiddenForumItemsLoaded" @click="save()" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { FloatLabel, InputText, InputNumber, Checkbox, MultiSelect, Button, Select } from 'primevue'
import { ref, onMounted, computed, toRaw } from 'vue'
import { useI18n } from 'vue-i18n'
import { rawToDisplayBp, displayToRawBp } from '@/services/helpers'
import ForbiddenForumItemsEditor from '@/components/forum/ForbiddenForumItemsEditor.vue'
import {
  createUserClass,
  editUserClass,
  editUserClassForbiddenForumItems,
  getUserClassForbiddenForumItems,
  UserPermission,
  type EditedUserClassForbiddenForumItems,
  type UserClass,
  type UserCreatedUserClass,
} from '@/services/api-schema'
import { showToast } from '@/main'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { useUserStore } from '@/stores/user'

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const userStore = useUserStore()

const props = defineProps<{
  initialUserClass?: UserClass
  availableClasses: UserClass[]
}>()

const emit = defineEmits<{
  done: [UserClass]
}>()

const userClass = ref<UserCreatedUserClass>({
  name: '',
  automatic_promotion: false,
  automatic_demotion: false,
  promotion_allowed_while_warned: false,
  new_permissions: [],
  previous_user_class: null,
  max_snatches_per_day: null,
  promotion_cost_bonus_points: 0,
  required_uploaded: 0,
  required_downloaded: 0,
  required_ratio: 0,
  required_account_age_in_days: 0,
  required_forum_posts: 0,
  required_forum_posts_in_unique_threads: 0,
  required_torrent_uploads: 0,
  required_torrent_uploads_in_unique_title_groups: 0,
  required_torrent_snatched: 0,
  required_seeding_size: 0,
  required_title_group_comments: 0,
})

const displayPromotionCostBonusPoints = computed({
  get: () => rawToDisplayBp(userClass.value.promotion_cost_bonus_points, publicArcadiaSettings.bonus_points_decimal_places),
  set: (value: number) => {
    userClass.value.promotion_cost_bonus_points = displayToRawBp(value, publicArcadiaSettings.bonus_points_decimal_places)
  },
})

const forbiddenForumItems = ref<EditedUserClassForbiddenForumItems>({
  category_ids: [],
  sub_category_ids: [],
})

const loading = ref(false)
const isEditMode = computed(() => !!props.initialUserClass)
const canEditForbiddenForumItems = computed(() => userStore.permissions.includes('edit_user_class'))
// saving before the current restrictions are loaded would replace them with empty ones
const forbiddenForumItemsLoaded = ref(!isEditMode.value || !canEditForbiddenForumItems.value)
// set once the class exists, so that a save retried after a failed restrictions update edits it instead of creating it again
const savedUserClassName = ref(props.initialUserClass?.name)

const allPermissions = computed(() =>
  Object.values(UserPermission).map((permission) => ({
    value: permission,
    label: t(`user_permissions.${permission}`),
  })),
)

const save = () => {
  loading.value = true
  const savedUserClass = savedUserClassName.value
    ? editUserClass({ name: savedUserClassName.value, EditedUserClass: userClass.value })
    : createUserClass(userClass.value)
  savedUserClass
    .then(async (result) => {
      savedUserClassName.value = result.name
      if (canEditForbiddenForumItems.value) {
        await editUserClassForbiddenForumItems({ name: result.name, EditedUserClassForbiddenForumItems: forbiddenForumItems.value })
      }
      showToast('Success', t(isEditMode.value ? 'user_class.user_class_edited_success' : 'user_class.user_class_created_success'), 'success', 4000)
      emit('done', result)
    })
    .finally(() => (loading.value = false))
}

onMounted(() => {
  if (props.initialUserClass) {
    userClass.value = structuredClone(toRaw(props.initialUserClass))
  }
  if (props.initialUserClass && canEditForbiddenForumItems.value) {
    getUserClassForbiddenForumItems(props.initialUserClass.name).then((items) => {
      forbiddenForumItems.value = {
        category_ids: items.categories.map((category) => category.id),
        sub_category_ids: items.sub_categories.map((subCategory) => subCategory.id),
      }
      forbiddenForumItemsLoaded.value = true
    })
  }
})
</script>

<style scoped>
.user-class-dialog {
  width: 60vw;
  max-width: 800px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
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

.permissions-select {
  min-width: 100%;
}

.permissions-select :deep(.p-multiselect-label) {
  display: flex;
  flex-wrap: wrap;
}

h3 {
  margin: 0;
  font-size: 1.2rem;
}
</style>
