<template>
  <div class="title">{{ t('user_settings.settings') }}</div>
  <div class="settings" v-if="updatedSettings">
    <ContentContainer class="section" :container-title="t('user_settings.appearance')">
      <div class="line">
        {{ t('user_settings.css_sheet') }}:
        <Button
          :label="updatedSettings.css_sheet_name ?? 'Default'"
          size="small"
          style="margin-left: 5px"
          v-tooltip.top="t('user_settings.change_css_sheet')"
          @click="changeCssSheetDialogVisible = true"
        />
      </div>
      <div class="line" style="margin-top: 10px">
        <Checkbox
          v-model="updatedSettings.show_upload_method"
          name="show_upload_method"
          :binary="true"
          inputId="show_upload_method"
          style="margin-right: 5px"
        />
        <label for="show_upload_method">{{ t('user_settings.show_upload_method') }}</label>
      </div>
    </ContentContainer>
    <ContentContainer class="section" :container-title="t('user_settings.security')">
      <div class="line">
        {{ t('user_settings.password') }}:
        <Button :label="t('user.change_password')" size="small" style="margin-left: 5px" @click="changePasswordDialogVisible = true" />
      </div>
      <div class="line" style="margin-top: 10px">
        {{ t('user_settings.api_keys') }}:
        <Button :label="t('user_settings.manage_api_keys')" size="small" style="margin-left: 5px" @click="apiKeysDialogVisible = true" />
      </div>
      <div class="line" style="margin-top: 10px">
        {{ t('user_settings.passkey') }}:
        <Button :label="t('user_settings.reset_passkey')" size="small" severity="warn" style="margin-left: 5px" @click="resetPasskeyDialogVisible = true" />
      </div>
    </ContentContainer>
    <ContentContainer class="section" :container-title="t('user_settings.privacy')">
      <div class="line anonymous" style="margin-bottom: 15px">
        <Button
          :label="t('user_settings.paranoia_make_all_uploads_anonymous')"
          size="small"
          :disabled="nonAnonymousUploadedTorrents === 0"
          @click="uploadsAnonymityToConfirm = true"
        />
        <Button
          :label="t('user_settings.paranoia_make_all_uploads_non_anonymous')"
          size="small"
          :disabled="anonymousUploadedTorrents === 0"
          @click="uploadsAnonymityToConfirm = false"
        />
        <span>
          {{ t('user_settings.paranoia_uploads_anonymity', { anonymous: anonymousUploadedTorrents, nonAnonymous: nonAnonymousUploadedTorrents }) }}
        </span>
      </div>
      <ParanoiaSettingsTable v-model="updatedSettings" />
    </ContentContainer>
    <ContentContainer v-if="publicSettings.irc_enabled" class="section" :container-title="t('user_settings.irc')">
      <div class="line" v-if="publicSettings.irc_webchat_enabled" style="margin-bottom: 10px">
        <Checkbox
          v-model="updatedSettings.irc_site_embed_enabled"
          name="irc_site_embed_enabled"
          :binary="true"
          inputId="irc_site_embed_enabled"
          style="margin-right: 5px"
        />
        <label for="irc_site_embed_enabled">{{ t('user_settings.show_irc_on_homepage') }}</label>
      </div>
      <div class="line">
        {{ t('user_settings.irc_account') }}:
        <Button
          v-if="userStore.irc_password"
          :label="t('user_settings.reset_irc_password')"
          size="small"
          severity="warn"
          style="margin-left: 5px"
          @click="ircDialogVisible = true"
        />
        <Button v-else :label="t('user_settings.create_irc_account')" size="small" style="margin-left: 5px" @click="ircDialogVisible = true" />
      </div>
      <div v-if="userStore.irc_password" class="line" style="margin-top: 10px">
        <b>{{ t('user_settings.irc_username_label') }}:</b> {{ userStore.username }}
      </div>
      <div v-if="userStore.irc_password" class="line" style="margin-top: 5px">
        <b>{{ t('user_settings.irc_password_label') }}:</b>
        <code v-if="ircPasswordVisible" style="margin-left: 5px">{{ userStore.irc_password }}</code>
        <Button
          :label="ircPasswordVisible ? t('user_settings.hide_irc_password') : t('user_settings.show_irc_password')"
          size="small"
          severity="secondary"
          style="margin-left: 5px"
          @click="ircPasswordVisible = !ircPasswordVisible"
        />
      </div>
    </ContentContainer>
  </div>
  <div class="wrapper-center">
    <Button :label="t('user_settings.save')" size="small" @click="saveSettings" />
  </div>
  <Dialog closeOnEscape modal :header="t('user_settings.change_css_sheet')" v-model:visible="changeCssSheetDialogVisible">
    <CssSheetList @sheetClicked="cssSheetChanged" />
  </Dialog>
  <Dialog
    closeOnEscape
    modal
    :header="t('user_settings.paranoia_uploads_anonymity_warning')"
    :visible="uploadsAnonymityToConfirm !== null"
    @update:visible="uploadsAnonymityToConfirm = null"
  >
    <p>
      {{
        uploadsAnonymityToConfirm
          ? t('user_settings.paranoia_confirm_make_all_uploads_anonymous', { count: nonAnonymousUploadedTorrents })
          : t('user_settings.paranoia_confirm_make_all_uploads_non_anonymous', { count: anonymousUploadedTorrents })
      }}
    </p>
    <template #footer>
      <Button :label="t('general.cancel')" severity="secondary" size="small" @click="uploadsAnonymityToConfirm = null" />
      <Button :label="t('general.confirm')" size="small" :loading="updatingUploadsAnonymity" @click="setAllUploadsAnonymity" />
    </template>
  </Dialog>
  <Dialog
    closeOnEscape
    modal
    :header="userStore.irc_password ? t('user_settings.reset_irc_password') : t('user_settings.create_irc_account')"
    v-model:visible="ircDialogVisible"
  >
    <IrcAccountDialog v-if="ircDialogVisible" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user.change_password')" v-model:visible="changePasswordDialogVisible">
    <ChangePasswordDialog v-if="changePasswordDialogVisible" @saved="changePasswordDialogVisible = false" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user_settings.reset_passkey')" v-model:visible="resetPasskeyDialogVisible">
    <p>{{ t('user_settings.reset_passkey_confirm') }}</p>
    <template #footer>
      <Button :label="t('general.cancel')" severity="secondary" size="small" @click="resetPasskeyDialogVisible = false" />
      <Button :label="t('general.confirm')" severity="warn" size="small" :loading="resettingPasskey" @click="confirmResetPasskey" />
    </template>
  </Dialog>
  <Dialog closeOnEscape modal :header="t('user_settings.api_keys')" v-model:visible="apiKeysDialogVisible">
    <ApiKeysDialog v-if="apiKeysDialogVisible" />
  </Dialog>
</template>
<script setup lang="ts">
import { isEqual } from 'lodash-es'
import { onBeforeUnmount, onMounted, ref, toRaw } from 'vue'
import { useI18n } from 'vue-i18n'
import ContentContainer from '@/components/ContentContainer.vue'
import { Button, Checkbox, Dialog } from 'primevue'
import CssSheetList from '@/components/CssSheetList.vue'
import IrcAccountDialog from '@/components/user/IrcAccountDialog.vue'
import ChangePasswordDialog from '@/components/user/ChangePasswordDialog.vue'
import ApiKeysDialog from '@/components/user/ApiKeysDialog.vue'
import ParanoiaSettingsTable from '@/components/user/ParanoiaSettingsTable.vue'
import { showToast } from '@/main'
import { useRouter, useRoute } from 'vue-router'
import { getUserSettings, resetPasskey, updateUploadedTorrentsAnonymity, updateUserSettings, type CssSheet, type UserSettings } from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const userStore = useUserStore()
const publicSettings = usePublicArcadiaSettingsStore()

const initialSettings = ref<UserSettings>()
const updatedSettings = ref<UserSettings>()
// those amounts are only read, they are not saved with the settings
const anonymousUploadedTorrents = ref(0)
const nonAnonymousUploadedTorrents = ref(0)
// anonymity awaiting the user's confirmation, `null` when the warning dialog is closed
const uploadsAnonymityToConfirm = ref<boolean | null>(null)
const updatingUploadsAnonymity = ref(false)
const changeCssSheetDialogVisible = ref(false)
const ircDialogVisible = ref(false)
const ircPasswordVisible = ref(false)
const changePasswordDialogVisible = ref(false)
const apiKeysDialogVisible = ref(false)
const resetPasskeyDialogVisible = ref(false)
const resettingPasskey = ref(false)

const cssSheetChanged = (cssSheet: CssSheet) => {
  if (!updatedSettings.value) return
  updatedSettings.value.css_sheet_name = cssSheet.name
  changeCssSheetDialogVisible.value = false
}

// the anonymity of the uploaded torrents is not saved with the other settings, it is applied
// immediately with its own endpoint, once the user confirmed it in the warning dialog
const setAllUploadsAnonymity = () => {
  const anonymous = uploadsAnonymityToConfirm.value
  if (anonymous === null) return
  updatingUploadsAnonymity.value = true
  updateUploadedTorrentsAnonymity({ anonymous })
    .then(() => {
      const uploadedTorrents = anonymousUploadedTorrents.value + nonAnonymousUploadedTorrents.value
      anonymousUploadedTorrents.value = anonymous ? uploadedTorrents : 0
      nonAnonymousUploadedTorrents.value = anonymous ? 0 : uploadedTorrents
      uploadsAnonymityToConfirm.value = null
      showToast(
        '',
        anonymous ? t('user_settings.paranoia_all_uploads_made_anonymous') : t('user_settings.paranoia_all_uploads_made_non_anonymous'),
        'success',
        3000,
      )
    })
    .finally(() => {
      updatingUploadsAnonymity.value = false
    })
}

const confirmResetPasskey = () => {
  resettingPasskey.value = true
  resetPasskey()
    .then(() => {
      resetPasskeyDialogVisible.value = false
      showToast('', t('user_settings.passkey_reset'), 'success', 5000)
    })
    .finally(() => {
      resettingPasskey.value = false
    })
}

const saveSettings = () => {
  if (!updatedSettings.value || !initialSettings.value) return
  if (isEqual(initialSettings.value, updatedSettings.value)) {
    showToast('', t('user_settings.settings_were_not_changed'), 'info', 2000)
    return
  }
  // the css sheet is only loaded when the app starts, the page is reloaded to apply a new one.
  // the other settings are applied without reloading
  const cssSheetChanged = initialSettings.value.css_sheet_name !== updatedSettings.value.css_sheet_name
  const savedSettings = updatedSettings.value
  updateUserSettings(savedSettings).then(() => {
    initialSettings.value = structuredClone(toRaw(savedSettings))
    // the settings that are also part of the user are read from the user store everywhere else
    userStore.irc_site_embed_enabled = savedSettings.irc_site_embed_enabled
    userStore.show_upload_method = savedSettings.show_upload_method
    if (cssSheetChanged) {
      router.push({ query: { saved: 'true' } }).then(() => {
        router.go(0)
      })
    } else {
      showToast('', t('user_settings.saved'), 'success', 3000)
    }
  })
}

onMounted(() => {
  getUserSettings().then(({ anonymous_uploaded_torrents, non_anonymous_uploaded_torrents, ...settings }) => {
    anonymousUploadedTorrents.value = anonymous_uploaded_torrents
    nonAnonymousUploadedTorrents.value = non_anonymous_uploaded_torrents
    initialSettings.value = settings
    updatedSettings.value = structuredClone(toRaw(settings))
  })
  if (route.query.saved === 'true') {
    showToast('', t('user_settings.saved'), 'success', 3000)
    router.push({ query: {} })
  }
})
onBeforeUnmount(() => {
  if (updatedSettings.value) {
    if (!isEqual(initialSettings.value, updatedSettings.value)) {
      // TODO: alert of unsaved settings
    }
  }
})
</script>

<style scoped>
.line {
  align-items: center;
}
.section {
  margin-bottom: 15px;
}
.anonymous {
  display: flex;
  align-items: center;
  gap: 10px;
}
</style>
