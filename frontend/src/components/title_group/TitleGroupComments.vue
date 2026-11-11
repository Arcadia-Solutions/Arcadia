<template>
  <div class="comments">
    <GeneralComment
      v-for="comment in comments"
      :key="comment.id"
      :comment="comment"
      @commentEdited="commentEdited($event, comment.id)"
      @commentDeleted="emit('commentDeleted', $event)"
      :editCommentMethod="(post: EditedTitleGroupComment) => editTitleGroupComment({ EditedTitleGroupComment: post, id: comment.id })"
      :deleteCommentMethod="deleteTitleGroupComment"
      :hasEditPermission="userStore.permissions.includes('edit_title_group_comment') || (comment.created_by.id === userStore.id && !comment.locked)"
      :hasDeletePermission="userStore.permissions.includes('delete_title_group_comment')"
    >
      <template #top-right>
        <UserRatingDisplay v-if="ratingsByUser.has(comment.created_by.id)" :rating="ratingsByUser.get(comment.created_by.id) ?? null" />
      </template>
    </GeneralComment>
  </div>
  <Form v-slot="$form" :initialValues="new_comment" :resolver @submit="onFormSubmit" validateOnSubmit :validateOnValueUpdate="false">
    <div class="new-comment">
      <BBCodeEditor
        :empty-input="bbcodeEditorEmptyInput"
        @value-change="newCommentUpdated"
        @input-emptied="bbcodeEditorEmptyInput = false"
        :label="t('community.new_comment')"
        name="content"
      >
        <template #buttons>
          <Button type="submit" v-tooltip.top="'Post'" icon="pi pi-send" :loading="sending_comment" class="post-button" />
          <div v-if="!isSubscribedToComments" class="subscribe-checkbox">
            <Checkbox v-model="subscribeOnComment" binary size="small" inputId="subscribe-on-comment" />
            <label for="subscribe-on-comment">{{ t('general.subscribe_on_reply') }}</label>
          </div>
        </template>
        <template #message>
          <Message v-if="$form.content?.invalid" severity="error" size="small" variant="simple">
            {{ $form.content.error?.message }}
          </Message>
        </template>
      </BBCodeEditor>
    </div>
  </Form>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed, ref } from 'vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import GeneralComment from '../community/GeneralComment.vue'
import { Button, Checkbox } from 'primevue'
import BBCodeEditor from '../community/BBCodeEditor.vue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import Message from 'primevue/message'
import { useUserStore } from '@/stores/user'
import { useRoute } from 'vue-router'
import {
  createTitleGroupComment,
  createTitleGroupCommentsSubscription,
  deleteTitleGroupComment,
  editTitleGroupComment,
  type EditedTitleGroupComment,
  type TitleGroupCommentHierarchy,
  type TitleGroupReviewHierarchy,
  type UserCreatedTitleGroupComment,
} from '@/services/api-schema'

const props = defineProps<{
  comments: TitleGroupCommentHierarchy[]
  isSubscribedToComments: boolean
  reviews: TitleGroupReviewHierarchy[]
}>()

const emit = defineEmits<{
  newComment: [TitleGroupCommentHierarchy]
  commentEdited: [EditedTitleGroupComment, number]
  commentDeleted: [number]
  subscribed: []
}>()

const { t } = useI18n()

const route = useRoute()
const ratingsByUser = computed(() => new Map(props.reviews.map((r) => [r.user_id, r.rating])))
const userStore = useUserStore()

const new_comment = ref<UserCreatedTitleGroupComment>({
  content: '',
  refers_to_torrent_id: null,
  answers_to_comment_id: null,
  title_group_id: 0,
})
const sending_comment = ref(false)
const bbcodeEditorEmptyInput = ref(false)
const subscribeOnComment = ref(false)

const commentEdited = (editedComment: EditedTitleGroupComment, commentId: number) => {
  emit('commentEdited', editedComment, commentId)
}

const resolver = ({ values }: FormResolverOptions) => {
  const errors: Partial<Record<keyof UserCreatedTitleGroupComment, { message: string }[]>> = {}

  if (values.content.length < 5) {
    errors.content = [{ message: t('error.write_more_than_x_chars', [5]) }]
  }

  return {
    errors,
  }
}

const onFormSubmit = ({ valid }: FormSubmitEvent) => {
  if (valid) {
    sendComment()
  }
}

const newCommentUpdated = (content: string) => {
  new_comment.value.content = content
}

const sendComment = async () => {
  sending_comment.value = true
  new_comment.value.title_group_id = parseInt(route.params.id as string)
  const createdComment: TitleGroupCommentHierarchy = {
    ...(await createTitleGroupComment(new_comment.value)),
    created_by: useUserStore(),
  }
  if (subscribeOnComment.value && !props.isSubscribedToComments) {
    createTitleGroupCommentsSubscription(parseInt(route.params.id as string)).then(() => {
      emit('subscribed')
    })
  }
  new_comment.value.content = ''
  new_comment.value.refers_to_torrent_id = null
  new_comment.value.answers_to_comment_id = null
  emit('newComment', createdComment)
  bbcodeEditorEmptyInput.value = true
  sending_comment.value = false
}
</script>
<style scoped>
.new-comment {
  display: flex;
  flex-direction: column;
  margin-top: 30px;
  margin-bottom: 30px;
  align-items: flex-end;
}
.subscribe-checkbox {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-left: 5px;
}
</style>
