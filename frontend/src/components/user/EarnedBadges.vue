<template>
  <div>
    <ContentContainer :containerTitle="t('user_badge.recent_badges')" :containerTitleLink="`/user/${userId}/badges`">
      <template #top-right>
        <i v-if="canAward" v-tooltip.top="t('user_badge.award_badge')" class="cursor-pointer pi pi-plus" @click="awardDialogVisible = true" />
      </template>
      <div v-if="badges.length" class="badges-row">
        <div v-for="badge in badges" :key="badge.id" class="badge">
          <div class="badge-image-wrapper" v-tooltip.top="{ value: tooltip(badge), escape: false }">
            <img :src="badge.badge_image_url" :alt="badge.badge_name" />
            <i v-if="canRevoke" v-tooltip.top="t('user_badge.revoke_badge')" class="revoke-button pi pi-times-circle" @click="confirmRevoke(badge)" />
          </div>
        </div>
      </div>
      <div v-else class="empty">{{ t('user_badge.no_earned_badges') }}</div>
    </ContentContainer>

    <Dialog closeOnEscape modal :header="t('user_badge.award_badge_to', [username])" v-model:visible="awardDialogVisible">
      <AwardUserBadgeDialog v-if="awardDialogVisible" :userId="userId" @awarded="onAwarded" />
    </Dialog>

    <Dialog closeOnEscape modal :header="t('user_badge.revoke_badge')" v-model:visible="revokeDialogVisible">
      <div class="revoke-dialog">
        <p>{{ t('user_badge.confirm_revoke_badge') }}</p>
        <Button :label="t('user_badge.revoke_badge')" severity="danger" size="small" :loading="revoking" @click="revoke" />
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, Button } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import { timeAgo } from '@/services/helpers'
import { revokeUserEarnedBadge, type UserEarnedBadgeWithDetails } from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'
import AwardUserBadgeDialog from '@/components/user/AwardUserBadgeDialog.vue'

const { t } = useI18n()
const userStore = useUserStore()

defineProps<{
  badges: UserEarnedBadgeWithDetails[]
  userId: number
  username: string
}>()

const emit = defineEmits<{
  awarded: []
  revoked: [number]
}>()

const canAward = computed(() => userStore.permissions.includes('award_user_badge'))
const canRevoke = computed(() => userStore.permissions.includes('revoke_user_badge'))

const awardDialogVisible = ref(false)
const revokeDialogVisible = ref(false)
const badgeBeingRevoked = ref<UserEarnedBadgeWithDetails | null>(null)
const revoking = ref(false)

// the tooltip is rendered as HTML, so the badge's texts are escaped
const escapeHtml = (text: string) => {
  const element = document.createElement('div')
  element.textContent = text
  return element.innerHTML
}

const tooltip = (badge: UserEarnedBadgeWithDetails) => {
  const parts = [`<span class="bold">${escapeHtml(badge.badge_name)}</span>`]
  if (badge.badge_description) parts.push(escapeHtml(badge.badge_description))
  parts.push(`<span style="opacity: 0.7">${escapeHtml(`${t('user_badge.awarded_at')} ${timeAgo(badge.awarded_at)}`)}</span>`)
  if (badge.note) parts.push(escapeHtml(`${t('user_badge.note')}: ${badge.note}`))
  return parts.join('<br>')
}

const onAwarded = () => {
  awardDialogVisible.value = false
  emit('awarded')
}

const confirmRevoke = (badge: UserEarnedBadgeWithDetails) => {
  badgeBeingRevoked.value = badge
  revokeDialogVisible.value = true
}

const revoke = () => {
  if (!badgeBeingRevoked.value) return
  revoking.value = true
  const id = badgeBeingRevoked.value.id
  revokeUserEarnedBadge(id)
    .then(() => {
      showToast('', t('user_badge.badge_revoked_success'), 'success', 2000)
      revokeDialogVisible.value = false
      emit('revoked', id)
    })
    .finally(() => {
      revoking.value = false
    })
}
</script>

<style scoped>
.badges-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 12px;
}
.badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 80px;
  text-align: center;
}
.badge-image-wrapper {
  position: relative;
}
.badge img {
  width: 64px;
  height: 64px;
  object-fit: contain;
  border-radius: 8px;
}
.revoke-button {
  position: absolute;
  top: -6px;
  right: -6px;
  background-color: var(--color-background-secondary);
  border-radius: 50%;
  color: var(--p-red-500);
  cursor: pointer;
  font-size: 1.1rem;
  display: none;
}
.badge-image-wrapper:hover .revoke-button {
  display: block;
}
.empty {
  opacity: 0.7;
  font-style: italic;
}
.revoke-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}
</style>
