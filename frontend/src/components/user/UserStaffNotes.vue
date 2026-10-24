<template>
  <div>
    <ContentContainer :containerTitle="t('user_staff_note.staff_notes')">
      <template #top-right>
        <i v-tooltip.top="t('user_staff_note.add_note')" class="cursor-pointer pi pi-plus" @click="addDialogVisible = true" />
      </template>
      <div v-if="notes.length" class="notes">
        <div v-for="note in notes" :key="note.id" class="note">
          <div class="note-header">
            <RouterLink :to="`/user/${note.author_id}`">{{ note.author_username }}</RouterLink>
            <span class="note-date"><TimeAgo :date="note.created_at" /></span>
            <div v-if="canBeEdited(note)" class="note-actions">
              <i v-tooltip.top="t('general.edit')" class="cursor-pointer pi pi-pen-to-square" @click="editNote(note)" />
              <i v-tooltip.top="t('general.delete')" class="cursor-pointer pi pi-trash" @click="confirmRemoval(note)" />
            </div>
          </div>
          <BBCodeRenderer :content="note.content" />
        </div>
      </div>
      <div v-else class="empty">{{ t('user_staff_note.no_staff_notes') }}</div>
    </ContentContainer>

    <Dialog closeOnEscape modal :header="t('user_staff_note.add_note')" v-model:visible="addDialogVisible">
      <UserStaffNoteDialog v-if="addDialogVisible" :userId="userId" @saved="noteSaved" />
    </Dialog>

    <Dialog closeOnEscape modal :header="t('user_staff_note.edit_note')" v-model:visible="editDialogVisible">
      <UserStaffNoteDialog
        v-if="noteBeingEdited"
        :userId="userId"
        :staffNoteId="noteBeingEdited.id"
        :initialContent="noteBeingEdited.content"
        @saved="noteSaved"
      />
    </Dialog>

    <Dialog closeOnEscape modal :header="t('user_staff_note.delete_note')" v-model:visible="deleteDialogVisible">
      <DeleteDialog
        v-if="noteBeingRemoved"
        :message="t('user_staff_note.confirm_delete_note')"
        :action="removeNote"
        :successMessage="t('user_staff_note.note_deleted_success')"
        @deleted="noteRemoved"
      />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Dialog } from 'primevue'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import ContentContainer from '@/components/ContentContainer.vue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import { deleteUserStaffNote, getUserStaffNotes, type UserStaffNoteWithAuthor } from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import UserStaffNoteDialog from '@/components/user/UserStaffNoteDialog.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'

// The author of a note can edit and remove it during the 24 hours following its creation. The
// same window is applied by the backend, in STAFF_NOTE_EDIT_WINDOW_IN_HOURS.
const NOTE_EDIT_WINDOW_IN_MILLISECONDS = 24 * 60 * 60 * 1000

const { t } = useI18n()
const userStore = useUserStore()

const props = defineProps<{
  userId: number
}>()

const notes = ref<UserStaffNoteWithAuthor[]>([])
const addDialogVisible = ref(false)
const editDialogVisible = ref(false)
const deleteDialogVisible = ref(false)
const noteBeingEdited = ref<UserStaffNoteWithAuthor | null>(null)
const noteBeingRemoved = ref<UserStaffNoteWithAuthor | null>(null)

const canBeEdited = (note: UserStaffNoteWithAuthor) => {
  return (
    userStore.permissions.includes('edit_user_staff_notes') ||
    (note.author_id === userStore.id && Date.now() - new Date(note.created_at).getTime() < NOTE_EDIT_WINDOW_IN_MILLISECONDS)
  )
}

const fetchNotes = () => {
  getUserStaffNotes(props.userId)
    .then((data) => {
      notes.value = data
    })
    .catch(() => {
      notes.value = []
    })
}

const editNote = (note: UserStaffNoteWithAuthor) => {
  noteBeingEdited.value = note
  editDialogVisible.value = true
}

const confirmRemoval = (note: UserStaffNoteWithAuthor) => {
  noteBeingRemoved.value = note
  deleteDialogVisible.value = true
}

const removeNote = () => {
  if (!noteBeingRemoved.value) return Promise.resolve()

  return deleteUserStaffNote({
    id: props.userId,
    staff_note_id: noteBeingRemoved.value.id,
  })
}

const noteSaved = () => {
  addDialogVisible.value = false
  editDialogVisible.value = false
  fetchNotes()
}

const noteRemoved = () => {
  deleteDialogVisible.value = false
  notes.value = notes.value.filter((note) => note.id !== noteBeingRemoved.value?.id)
  noteBeingRemoved.value = null
}

fetchNotes()
</script>

<style scoped>
.notes {
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.note {
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}
.note:last-child {
  border-bottom: none;
}
.note-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 5px;
}
.note-date {
  opacity: 0.7;
  font-size: 0.8rem;
}
.note-actions {
  margin-left: auto;
  display: flex;
  gap: 8px;
}
.empty {
  opacity: 0.7;
  font-style: italic;
}
</style>
