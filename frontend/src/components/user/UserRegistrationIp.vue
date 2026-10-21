<template>
  <ContentContainer v-if="registrationIp" :containerTitle="`${t('user_registration_ip.registration_ip')}: ${registrationIp.registered_from_ip}`">
    <template #top-right>
      <i
        v-tooltip.top="t('user_registration_ip.check_now')"
        :class="['cursor-pointer', checking ? 'pi pi-spin pi-spinner' : 'pi pi-refresh']"
        @click="checkNow"
      />
    </template>
    <IpCheckTable v-if="registrationIp.provider" :check="registrationIp" />
    <div v-else class="empty">{{ t('user_registration_ip.not_checked') }}</div>
    <Dialog closeOnEscape modal :header="t('user_registration_ip.current_check')" v-model:visible="dialogVisible">
      <template v-if="currentCheck">
        <IpCheckTable :check="{ provider: currentCheck.provider, ...currentCheck.result }" />
        <div class="actions">
          <Button size="small" :label="t('user_registration_ip.save_check')" :loading="saving" @click="saveCheck" />
        </div>
      </template>
    </Dialog>
  </ContentContainer>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import ContentContainer from '@/components/ContentContainer.vue'
import IpCheckTable from '@/components/user/IpCheckTable.vue'
import { checkUserRegistrationIp, getUserRegistrationIp, setUserRegistrationIpCheck, type IpCheck, type UserRegistrationIp } from '@/services/api-schema'

const { t } = useI18n()

const props = defineProps<{
  userId: number
}>()

const registrationIp = ref<UserRegistrationIp | null>(null)
const currentCheck = ref<IpCheck | null>(null)
const dialogVisible = ref(false)
const checking = ref(false)
const saving = ref(false)

const checkNow = () => {
  if (checking.value) return
  checking.value = true
  checkUserRegistrationIp(props.userId)
    .then((data) => {
      currentCheck.value = data
      dialogVisible.value = true
    })
    .finally(() => {
      checking.value = false
    })
}

const saveCheck = () => {
  if (!currentCheck.value || !registrationIp.value) return
  const check = currentCheck.value
  saving.value = true
  setUserRegistrationIpCheck({ id: props.userId, IpCheck: check })
    .then(() => {
      registrationIp.value = { registered_from_ip: registrationIp.value!.registered_from_ip, provider: check.provider, ...check.result }
      dialogVisible.value = false
    })
    .finally(() => {
      saving.value = false
    })
}

getUserRegistrationIp(props.userId).then((data) => {
  registrationIp.value = data
})
</script>

<style scoped>
.empty {
  opacity: 0.7;
  font-style: italic;
}
.actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 15px;
}
</style>
