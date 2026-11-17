<template>
  <div class="edit-thread-form">
    <FloatLabel>
      <InputText v-model="editedThread.name" id="thread-name" class="thread-name-input" />
      <label for="thread-name">{{ t('forum.thread_name') }}</label>
    </FloatLabel>

    <FloatLabel>
      <InputNumber v-model="editedThread.forum_sub_category_id" id="subcategory" />
      <label for="subcategory">{{ t('forum.subcategory') }}</label>
    </FloatLabel>

    <div v-if="userStore.permissions.includes('edit_forum_thread')" class="checkbox-row">
      <Checkbox v-model="editedThread.locked" binary inputId="thread-locked" size="small" />
      <label for="thread-locked">{{ t('general.locked') }}</label>
    </div>

    <Button :label="t('general.submit')" :loading="submitting" @click="submitEdit" class="submit-button" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Checkbox, FloatLabel, InputNumber, InputText } from 'primevue'
import { useUserStore } from '@/stores/user'
import { editForumThread, type EditedForumThread, type ForumThreadEnriched } from '@/services/api-schema'

const props = defineProps<{
  forumThread: ForumThreadEnriched
}>()

const emit = defineEmits<{
  done: [ForumThreadEnriched]
}>()

const { t } = useI18n()
const userStore = useUserStore()

const editedThread = ref<EditedForumThread>({
  id: props.forumThread.id,
  name: props.forumThread.name,
  forum_sub_category_id: props.forumThread.forum_sub_category_id,
  locked: props.forumThread.locked,
})

const submitting = ref(false)

const submitEdit = async () => {
  submitting.value = true
  editForumThread(editedThread.value)
    .then((thread) => emit('done', thread))
    .finally(() => (submitting.value = false))
}
</script>

<style scoped>
.edit-thread-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  min-width: 300px;
}

.thread-name-input {
  width: 100%;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.submit-button {
  align-self: flex-end;
}
</style>
