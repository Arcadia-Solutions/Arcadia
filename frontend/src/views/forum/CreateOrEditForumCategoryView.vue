<template>
  <ContentContainer :title="isEditMode ? t('forum.edit_category') : t('forum.create_category')">
    <Form v-if="!isEditMode || formData.id !== undefined" v-slot="$form" :initialValues="formData" :resolver @submit="onFormSubmit">
      <FloatLabel>
        <InputText v-model="formData.name" name="name" :class="{ 'p-invalid': $form.name?.invalid }" />
        <label for="name">{{ t('forum.category_name') }}</label>
      </FloatLabel>
      <Message v-if="$form.name?.invalid" severity="error" size="small" variant="simple">
        {{ $form.name.error?.message }}
      </Message>

      <div v-if="userStore.permissions.includes('edit_forum_category')" class="allowed-users-field">
        <ForumItemClassVisibility v-if="classVisibility" :classVisibility />
        <div>{{ t('forum.allowed_users') }}</div>
        <AllowedUsersEditor v-model="allowedUsers" />
      </div>

      <div class="actions">
        <Button type="submit" :label="isEditMode ? t('general.save') : t('general.create')" :loading="loading" />
      </div>
    </Form>
  </ContentContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Button, FloatLabel, InputText, Message } from 'primevue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import ContentContainer from '@/components/ContentContainer.vue'
import AllowedUsersEditor from '@/components/forum/AllowedUsersEditor.vue'
import ForumItemClassVisibility from '@/components/forum/ForumItemClassVisibility.vue'
import { useUserStore } from '@/stores/user'
import {
  createForumCategory,
  editForumCategory,
  getForumCategory,
  type ForumItemClassVisibility as ForumItemClassVisibilityType,
  type UserLite,
} from '@/services/api-schema/api'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const isEditMode = computed(() => route.path.includes('/edit'))
const loading = ref(false)
const formData = ref<{ id?: number; name: string }>({
  name: '',
})
const allowedUsers = ref<UserLite[]>([])
const classVisibility = ref<ForumItemClassVisibilityType | null>(null)

const resolver = ({ values }: FormResolverOptions) => {
  const errors: Record<string, { message: string }[]> = {}

  if (!values.name || values.name.trim().length === 0) {
    errors.name = [{ message: t('error.field_required') }]
  } else if (values.name.trim().length < 2) {
    errors.name = [{ message: t('error.write_more_than_x_chars', [1]) }]
  }

  return { errors }
}

const onFormSubmit = ({ valid }: FormSubmitEvent) => {
  if (!valid) return
  loading.value = true
  const allowedUserIds = allowedUsers.value.map((user) => user.id)
  const savedCategory = isEditMode.value
    ? editForumCategory({ id: formData.value.id!, name: formData.value.name, allowed_user_ids: allowedUserIds })
    : createForumCategory({ name: formData.value.name, allowed_user_ids: allowedUserIds })
  savedCategory.then(() => router.push('/forum')).finally(() => (loading.value = false))
}

onMounted(() => {
  if (isEditMode.value) {
    getForumCategory(Number(route.params.id))
      .then((category) => {
        formData.value = {
          id: category.id,
          name: category.name,
        }
        allowedUsers.value = category.allowed_users
        classVisibility.value = category.class_visibility
      })
      .catch(() => router.push('/forum'))
  }
})
</script>

<style scoped>
.allowed-users-field {
  margin-top: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}
</style>
