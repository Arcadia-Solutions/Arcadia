<template>
  <ContentContainer>
    <div class="title">{{ pageTitle }}</div>
    <Form v-if="!isEditMode || formData.id !== undefined" v-slot="$form" :initialValues="formData" :resolver @submit="onFormSubmit">
      <FloatLabel>
        <InputText v-model="formData.name" name="name" :class="{ 'p-invalid': $form.name?.invalid }" />
        <label for="name">{{ t('forum.subcategory_name') }}</label>
      </FloatLabel>
      <Message v-if="$form.name?.invalid" severity="error" size="small" variant="simple">
        {{ $form.name.error?.message }}
      </Message>
      <div class="checkbox-field">
        <Checkbox v-model="formData.new_threads_restricted" inputId="new_threads_restricted" :binary="true" />
        <label for="new_threads_restricted">{{ t('forum.new_threads_restricted') }}</label>
      </div>
      <div v-if="isEditMode" class="sort-by-field">
        <FloatLabel>
          <Dropdown
            v-model="formData.thread_sort_by"
            inputId="thread_sort_by"
            :options="[
              { label: t('forum.thread_sort_by_latest_post'), value: ForumThreadSortBy.LatestPost },
              { label: t('forum.thread_sort_by_created_at'), value: ForumThreadSortBy.CreatedAt },
              { label: t('forum.thread_sort_by_name'), value: ForumThreadSortBy.Name },
              { label: t('forum.thread_sort_by_posts_amount'), value: ForumThreadSortBy.PostsAmount },
              { label: t('forum.thread_sort_by_views_count'), value: ForumThreadSortBy.ViewsCount },
            ]"
            optionLabel="label"
            optionValue="value"
            size="small"
          />
          <label for="thread_sort_by">{{ t('forum.thread_sort_by') }}</label>
        </FloatLabel>
      </div>
      <div v-if="isEditMode" class="sort-direction-field">
        <FloatLabel>
          <Dropdown
            v-model="formData.thread_sort_direction"
            inputId="thread_sort_direction"
            :options="[
              { label: t('forum.thread_sort_direction_ascending'), value: ForumThreadSortDirection.Ascending },
              { label: t('forum.thread_sort_direction_descending'), value: ForumThreadSortDirection.Descending },
            ]"
            optionLabel="label"
            optionValue="value"
            size="small"
          />
          <label for="thread_sort_direction">{{ t('forum.thread_sort_direction') }}</label>
        </FloatLabel>
      </div>
      <div v-if="userStore.permissions.includes('edit_forum_sub_category')" class="allowed-users-field">
        <ForumItemClassVisibility v-if="classVisibility" :classVisibility />
        <div>{{ t('forum.allowed_users_sub_category') }}</div>
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
import { Button, Checkbox, Dropdown, FloatLabel, InputText, Message } from 'primevue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import ContentContainer from '@/components/ContentContainer.vue'
import AllowedUsersEditor from '@/components/forum/AllowedUsersEditor.vue'
import ForumItemClassVisibility from '@/components/forum/ForumItemClassVisibility.vue'
import { useUserStore } from '@/stores/user'
import { createForumSubCategory, editForumSubCategory, getForumSubCategoryThreads } from '@/services/api-schema/api'
import {
  ForumThreadSortBy,
  ForumThreadSortDirection,
  type ForumItemClassVisibility as ForumItemClassVisibilityType,
  type UserLite,
} from '@/services/api-schema'
import { showToast } from '@/main'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const isEditMode = computed(() => route.path.includes('/edit'))
const loading = ref(false)
const categoryName = ref('')
const formData = ref<{
  id?: number
  name: string
  forum_category_id?: number
  new_threads_restricted: boolean
  thread_sort_by: ForumThreadSortBy
  thread_sort_direction: ForumThreadSortDirection
}>({
  name: '',
  new_threads_restricted: false,
  thread_sort_by: ForumThreadSortBy.LatestPost,
  thread_sort_direction: ForumThreadSortDirection.Descending,
})
const allowedUsers = ref<UserLite[]>([])
const classVisibility = ref<ForumItemClassVisibilityType | null>(null)

const pageTitle = computed(() => {
  if (isEditMode.value) {
    return t('forum.edit_subcategory')
  }
  return categoryName.value ? `${t('forum.create_sub_category')} in category "${categoryName.value}"` : t('forum.create_sub_category')
})

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
  const savedSubCategory = isEditMode.value
    ? editForumSubCategory({
        id: formData.value.id!,
        name: formData.value.name,
        new_threads_restricted: formData.value.new_threads_restricted,
        thread_sort_by: formData.value.thread_sort_by,
        thread_sort_direction: formData.value.thread_sort_direction,
        allowed_user_ids: allowedUserIds,
      })
    : createForumSubCategory({
        forum_category_id: formData.value.forum_category_id!,
        name: formData.value.name,
        new_threads_restricted: formData.value.new_threads_restricted,
        allowed_user_ids: allowedUserIds,
      })
  savedSubCategory.then(() => router.go(-1)).finally(() => (loading.value = false))
}

onMounted(() => {
  if (isEditMode.value) {
    getForumSubCategoryThreads(Number(route.params.id))
      .then((subCategory) => {
        formData.value = {
          id: subCategory.id,
          name: subCategory.name,
          forum_category_id: subCategory.category.id,
          new_threads_restricted: subCategory.new_threads_restricted,
          thread_sort_by: subCategory.thread_sort_by,
          thread_sort_direction: subCategory.thread_sort_direction,
        }
        allowedUsers.value = subCategory.allowed_users ?? []
        classVisibility.value = subCategory.class_visibility ?? null
        categoryName.value = subCategory.category.name
      })
      .catch(() => router.push('/forum'))
  } else {
    const categoryId = route.query.categoryId
    const catName = route.query.categoryName

    if (!categoryId || !catName) {
      showToast('', 'Category information missing', 'error', 2000)
      router.push('/forum')
      return
    }

    formData.value.forum_category_id = Number(categoryId)
    categoryName.value = String(catName)
  }
})
</script>

<style scoped>
.checkbox-field {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
}
.sort-by-field {
  margin-top: 20px;
}
.sort-direction-field {
  margin-top: 20px;
}
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
