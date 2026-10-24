<template>
  <ContentContainer class="comment-container" :id="`post-${comment.id}`">
    <template #top-right>
      <div class="actions">
        <i
          class="pi pi-pen-to-square"
          v-if="(userStore.id === comment.created_by.id && 'locked' in comment && comment.locked === false) || hasEditPermission"
          @click="editCommentDialogVisible = true"
        />
        <i class="pi pi-trash" v-if="hasDeletePermission" @click="deleteCommentDialogVisible = true" />
        <RouterLink
          :to="{
            query: { post_id: comment.id },
            hash: `#post-${comment.id}`,
          }"
        >
          <i class="pi pi-link" />
        </RouterLink>
      </div>
    </template>
    <template #top-left>
      <div class="top-left">
        <UsernameEnriched :user="comment.created_by" displayAllInfo />
        <span> <TimeAgo :date="comment.created_at" /></span>
        <i
          v-if="userStore.id !== comment.created_by.id"
          v-tooltip.top="t('user.gift.send_gift', [comment.created_by.username])"
          class="cursor-pointer pi pi-gift"
          @click="sendGiftDialogVisible = true"
        />
      </div>
    </template>
    <div class="comment">
      <div class="user">
        <img class="avatar" :src="comment.created_by.avatar ?? '/default_user_avatar.png'" :alt="comment.created_by.username + '\'s avatar'" />
      </div>
      <div class="comment-body">
        <BBCodeRenderer :content="comment.content" />
        <ContentReactions v-if="reactions && 'forum_thread_id' in comment" :reactions="reactions" :forumPostId="comment.id" />
      </div>
    </div>
  </ContentContainer>
  <!-- Only comments that containt the 'locked' key can be edited -->
  <Dialog closeOnEscape modal v-model:visible="editCommentDialogVisible" v-if="'locked' in comment">
    <EditCommentDialog :initialComment="comment" @commentEdited="updateComment" :showLockOption="hasEditPermission" />
  </Dialog>
  <Dialog closeOnEscape modal :header="deleteDialogHeader" v-model:visible="deleteCommentDialogVisible">
    <div class="delete-dialog">
      <p>{{ deleteConfirmationMessage }}</p>
      <Button :label="t('general.delete')" severity="danger" size="small" :loading="deletingComment" @click="deleteComment" />
    </div>
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.gift.send_gift', [comment.created_by.username])" v-model:visible="sendGiftDialogVisible">
    <SendGiftDialog :receiverId="comment.created_by.id" @sent="sendGiftDialogVisible = false" v-if="sendGiftDialogVisible" />
  </Dialog>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import ContentReactions from '@/components/community/ContentReactions.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import type {
  ContentReaction,
  ConversationMessageHierarchy,
  EditedForumPost,
  EditedTitleGroupComment,
  ForumPostHierarchy,
  TitleGroupCommentHierarchy,
  TorrentRequestCommentHierarchy,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { Button, Dialog } from 'primevue'
import EditCommentDialog from './EditCommentDialog.vue'
import SendGiftDialog from '@/components/user/SendGiftDialog.vue'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { showToast } from '@/main'

const props = defineProps<{
  comment: TitleGroupCommentHierarchy | ForumPostHierarchy | ConversationMessageHierarchy | TorrentRequestCommentHierarchy
  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  editCommentMethod?: Function
  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  deleteCommentMethod?: Function
  hasEditPermission: boolean
  hasDeletePermission?: boolean
  reactions?: ContentReaction[]
}>()

const emit = defineEmits<{
  commentEdited: [EditedForumPost | EditedTitleGroupComment]
  commentDeleted: [number]
}>()

const userStore = useUserStore()
const { t } = useI18n()
const editCommentDialogVisible = ref(false)
const deleteCommentDialogVisible = ref(false)
const sendGiftDialogVisible = ref(false)
const loadingUpdatingComment = ref(false)
const deletingComment = ref(false)

const isForumPost = computed(() => 'forum_thread_id' in props.comment)
const deleteDialogHeader = computed(() => (isForumPost.value ? t('forum.delete_post') : t('community.delete_comment')))
const deleteConfirmationMessage = computed(() => (isForumPost.value ? t('forum.confirm_delete_post') : t('community.confirm_delete_comment')))
const deleteSuccessMessage = computed(() => (isForumPost.value ? t('forum.post_deleted_success') : t('community.comment_deleted_success')))

const updateComment = async (comment: EditedForumPost | EditedTitleGroupComment) => {
  if (!props.editCommentMethod) return
  loadingUpdatingComment.value = true
  props
    .editCommentMethod(comment)
    .then(() => {
      emit('commentEdited', comment)
      editCommentDialogVisible.value = false
    })
    .finally(() => (loadingUpdatingComment.value = false))
}

const deleteComment = async () => {
  if (!props.deleteCommentMethod) return
  deletingComment.value = true
  props
    .deleteCommentMethod(props.comment.id)
    .then(() => {
      showToast('', deleteSuccessMessage.value, 'success', 2000)
      deleteCommentDialogVisible.value = false
      emit('commentDeleted', props.comment.id)
    })
    .finally(() => (deletingComment.value = false))
}
</script>

<style scoped>
.comment-container {
  margin-top: 7px;
}
.comment-container :deep(.top) {
  background-color: #111111;
}
.top-left {
  margin-top: -5px;
  i {
    margin-left: 7px;
  }
}
.comment {
  display: flex;
}
.avatar {
  width: 9em;
  border-radius: 7px;
  margin-top: 7px;
}
@media (max-width: 767px) {
  .avatar {
    width: 4em;
  }
}
.actions {
  float: right;
  i {
    margin-left: 7px;
    cursor: pointer;
  }
}
.comment-body {
  /* Column so the reactions can be pushed to the bottom of the post, which is at least as tall
     as the avatar next to it. */
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 7px;
  padding-right: 0;
}
.delete-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}
</style>
