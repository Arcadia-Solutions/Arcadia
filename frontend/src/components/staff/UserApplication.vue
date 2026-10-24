<template>
  <ContentContainer class="user-application">
    <div class="information">
      <div class="item">
        <span>{{ t('staff.user_application.body') }}</span>
        <div class="content">{{ userApplication.body }}</div>
      </div>
      <div class="item">
        <span>{{ t('user.email') }}</span>
        <div class="content">{{ userApplication.email }}</div>
      </div>
      <div class="item status">
        <span>{{ t('staff.user_application.status') }}</span>
        <div :class="`content ${userApplication.status}`">{{ userApplication.status }}</div>
      </div>
      <div class="item">
        <span>{{ t('general.sent') }}</span>
        <div class="content"><TimeAgo :date="userApplication.created_at" /></div>
      </div>
      <div class="item" v-if="userApplication.user">
        <span>{{ t('user.user') }}</span>
        <div class="content"><UsernameEnriched :user="userApplication.user" /></div>
      </div>
    </div>
    <div class="buttons" v-if="userApplication.status === 'pending'">
      <Button :label="t('user.send_invitation')" size="small" severity="success" @click="sendInvitationDialogVisible = true" />
      <Button :label="t('staff.user_application.reject')" size="small" severity="danger" @click="updateApplication('rejected')" :loading="rejectLoading" />
    </div>
  </ContentContainer>
  <Dialog closeOnEscape modal :header="t('user.send_invitation')" v-model:visible="sendInvitationDialogVisible">
    <SendInvitationDialog
      @invitationSent="emit('applicationUpdated', { ...userApplication, status: 'accepted' })"
      :receiverEmail="userApplication.email"
      :applicationId="userApplication.id"
    />
  </Dialog>
</template>
<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { useI18n } from 'vue-i18n'
import ContentContainer from '../ContentContainer.vue'
import { Dialog } from 'primevue'
import { Button } from 'primevue'
import SendInvitationDialog from '../user/SendInvitationDialog.vue'
import { ref } from 'vue'
import { updateUserApplicationStatus, type PaginatedResultsUserApplicationHierarchyResultsInner, type UserApplicationStatus } from '@/services/api-schema'
import UsernameEnriched from '../user/UsernameEnriched.vue'

const { t } = useI18n()

const props = defineProps<{
  userApplication: PaginatedResultsUserApplicationHierarchyResultsInner
}>()

const emit = defineEmits<{
  applicationUpdated: [PaginatedResultsUserApplicationHierarchyResultsInner]
}>()

const rejectLoading = ref(false)
const sendInvitationDialogVisible = ref(false)

const updateApplication = (status: UserApplicationStatus) => {
  if (status === 'rejected') {
    rejectLoading.value = true
  }
  updateUserApplicationStatus({ status: status, user_application_id: props.userApplication.id })
    .then((updatedApplication) => {
      emit('applicationUpdated', updatedApplication)
    })
    .finally(() => {
      if (status === 'rejected') {
        rejectLoading.value = false
      }
    })
}
</script>

<style scoped>
.user-application {
  margin-bottom: 20px;
}
.information {
  .item {
    margin-bottom: 20px;
    span {
      font-weight: bold;
    }
    .content {
      margin-top: 5px;
    }
  }
}
.status {
  .pending {
    color: #edc01e;
  }
  .accepted {
    color: green;
  }
  .rejected {
    color: red;
  }
}
.buttons {
  text-align: center;
  .p-button {
    margin: 0 5px;
  }
}
</style>
