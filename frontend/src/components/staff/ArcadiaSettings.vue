<template>
  <div class="arcadia-settings" v-if="settings">
    <Form v-slot="$form" :initialValues="settings" :resolver @submit="saveSettings" validateOnSubmit :validateOnValueUpdate="false">
      <FloatLabel>
        <Select
          v-model="settings.default_css_sheet_name"
          :options="cssSheets"
          optionLabel="name"
          optionValue="name"
          name="default_css_sheet_name"
          size="small"
        />
        <label>{{ t('arcadia_settings.default_css_sheet_name') }}</label>
      </FloatLabel>
      <Message v-if="$form.default_css_sheet_name?.invalid" severity="error" size="small" variant="simple">
        {{ $form.default_css_sheet_name.error.message }}
      </Message>

      <FloatLabel>
        <Select
          v-model="settings.user_class_name_on_signup"
          :options="userClasses"
          optionLabel="name"
          optionValue="name"
          name="user_class_name_on_signup"
          size="small"
        />
        <label>{{ t('arcadia_settings.user_class_name_on_signup') }}</label>
      </FloatLabel>
      <Message v-if="$form.user_class_name_on_signup?.invalid" severity="error" size="small" variant="simple">
        {{ $form.user_class_name_on_signup.error.message }}
      </Message>

      <FloatLabel>
        <InputNumber v-model="settings.global_download_factor" name="global_download_factor" :min="0" :step="1" size="small" />
        <label>{{ t('arcadia_settings.global_download_factor') }}</label>
      </FloatLabel>
      <Message v-if="$form.global_download_factor?.invalid" severity="error" size="small" variant="simple">
        {{ $form.global_download_factor.error.message }}
      </Message>

      <FloatLabel>
        <InputNumber v-model="settings.global_upload_factor" name="global_upload_factor" :min="0" :step="1" size="small" />
        <label>{{ t('arcadia_settings.global_upload_factor') }}</label>
      </FloatLabel>
      <Message v-if="$form.global_upload_factor?.invalid" severity="error" size="small" variant="simple">
        {{ $form.global_upload_factor.error.message }}
      </Message>

      <FloatLabel>
        <InputNumber
          v-model="settings.global_snatch_bonus_points_cost_factor"
          name="global_snatch_bonus_points_cost_factor"
          :min="0"
          :max="32767"
          :step="1"
          size="small"
        />
        <label>{{ t('arcadia_settings.global_snatch_bonus_points_cost_factor') }}</label>
      </FloatLabel>
      <Message v-if="$form.global_snatch_bonus_points_cost_factor?.invalid" severity="error" size="small" variant="simple">
        {{ $form.global_snatch_bonus_points_cost_factor.error.message }}
      </Message>

      <FloatLabel>
        <InputNumber
          v-model="settings.global_snatch_bonus_points_reward_factor"
          name="global_snatch_bonus_points_reward_factor"
          :min="0"
          :max="32767"
          :step="1"
          size="small"
        />
        <label>{{ t('arcadia_settings.global_snatch_bonus_points_reward_factor') }}</label>
      </FloatLabel>
      <Message v-if="$form.global_snatch_bonus_points_reward_factor?.invalid" severity="error" size="small" variant="simple">
        {{ $form.global_snatch_bonus_points_reward_factor.error.message }}
      </Message>

      <FloatLabel>
        <InputText v-model="settings.logo_subtitle" name="logo_subtitle" :min="0" :step="1" size="small" />
        <label>{{ t('arcadia_settings.logo_subtitle') }}</label>
      </FloatLabel>

      <FloatLabel>
        <InputText v-model="settings.bonus_points_alias" name="bonus_points_alias" size="small" />
        <label>{{ t('arcadia_settings.bonus_points_alias') }}</label>
      </FloatLabel>

      <FloatLabel>
        <InputNumber v-model="settings.bonus_points_decimal_places" name="bonus_points_decimal_places" :min="0" :max="8" :step="1" size="small" />
        <label>{{ t('arcadia_settings.bonus_points_decimal_places') }}</label>
      </FloatLabel>

      <div style="margin-top: 15px">
        <label>{{ t('arcadia_settings.bonus_points_per_endpoint') }}</label>
        <BonusPointsEndpointEditor v-model="settings.bonus_points_per_endpoint" :bpDecimalPlaces="bpDecimalPlaces" />
      </div>

      <div style="margin-top: 10px; margin-bottom: 10px">
        <Checkbox
          v-model="settings.reward_bonus_points_per_seeding_client"
          name="reward_bonus_points_per_seeding_client"
          :binary="true"
          inputId="reward_bonus_points_per_seeding_client"
          style="margin-right: 5px"
        />
        <label for="reward_bonus_points_per_seeding_client">{{ t('arcadia_settings.reward_bonus_points_per_seeding_client') }}</label>
      </div>

      <div style="margin-top: 10px; margin-bottom: 10px">
        <Checkbox
          v-model="settings.charge_bonus_points_on_resnatch"
          name="charge_bonus_points_on_resnatch"
          :binary="true"
          inputId="charge_bonus_points_on_resnatch"
          style="margin-right: 5px"
        />
        <label for="charge_bonus_points_on_resnatch">{{ t('arcadia_settings.charge_bonus_points_on_resnatch') }}</label>
      </div>

      <div style="margin-top: 10px; margin-bottom: 10px">
        <Checkbox v-model="settings.uploads_disabled" name="uploads_disabled" :binary="true" inputId="uploads_disabled" style="margin-right: 5px" />
        <label for="uploads_disabled">{{ t('arcadia_settings.uploads_disabled') }}</label>
      </div>

      <FloatLabel>
        <Chips v-model="settings.approved_image_hosts" name="approved_image_hosts" separator="," size="small" style="width: 40em" />
        <label>{{ t('arcadia_settings.approved_image_hosts') }} {{ t('arcadia_settings.approved_image_hosts_hint') }}</label>
      </FloatLabel>

      <div style="margin-top: 10px; margin-bottom: 10px">
        <Checkbox
          v-model="settings.display_image_host_drag_and_drop"
          name="display_image_host_drag_and_drop"
          :binary="true"
          inputId="display_image_host_drag_and_drop"
          style="margin-right: 5px"
        />
        <label for="display_image_host_drag_and_drop">{{ t('arcadia_settings.display_image_host_drag_and_drop') }}</label>
      </div>

      <FloatLabel>
        <MultiSelect
          v-model="settings.displayed_top_bar_stats"
          :options="
            Object.entries(DisplayedTopBarStats).map(([key, value]) => ({
              label: value,
              value,
            }))
          "
          optionLabel="label"
          optionValue="value"
          name="displayed_top_bar_stats"
          size="small"
          display="chip"
          class="displayed-top-bar-stats"
        />
        <label>{{ t('arcadia_settings.displayed_top_bar_stats') }}</label>
      </FloatLabel>

      <FloatLabel>
        <MultiSelect
          v-model="settings.displayable_user_stats"
          :options="
            Object.entries(DisplayableUserStats).map(([_, value]) => ({
              label: value,
              value,
            }))
          "
          optionLabel="label"
          optionValue="value"
          name="displayable_user_stats"
          size="small"
          display="chip"
          class="displayable-user-stats"
        />
        <label>{{ t('arcadia_settings.displayable_user_stats') }}</label>
      </FloatLabel>

      <FloatLabel>
        <MultiSelect
          v-model="settings.torrent_request_vote_currencies"
          :options="
            Object.entries(TorrentRequestVoteCurrency).map(([_, value]) => ({
              label: value,
              value,
            }))
          "
          optionLabel="label"
          optionValue="value"
          name="torrent_request_vote_currencies"
          size="small"
          display="chip"
          class="torrent-request-vote-currencies"
        />
        <label>{{ t('arcadia_settings.torrent_request_vote_currencies') }}</label>
      </FloatLabel>

      <ContentContainer class="settings-section" :containerTitle="t('arcadia_settings.torrent_upload_settings')" style="margin-top: 20px">
        <FloatLabel>
          <InputNumber v-model="settings.min_amount_tags_title_group" name="min_amount_tags_title_group" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.min_amount_tags_title_group') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputNumber v-model="settings.reseed_requestable_after_hours" name="reseed_requestable_after_hours" :min="1" :step="1" size="small" />
          <label>{{ t('arcadia_settings.reseed_requestable_after_hours') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputNumber v-model="displayBonusPointsGivenOnUpload" name="bonus_points_given_on_upload" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.bonus_points_given_on_upload') }}</label>
        </FloatLabel>
        <Message v-if="$form.bonus_points_given_on_upload?.invalid" severity="error" size="small" variant="simple">
          {{ $form.bonus_points_given_on_upload.error.message }}
        </Message>

        <FloatLabel>
          <InputNumber v-model="displayDefaultTorrentBonusPointsCost" name="default_torrent_bonus_points_cost" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.default_torrent_bonus_points_cost') }}</label>
        </FloatLabel>

        <div style="margin-top: 10px; margin-bottom: 30px">
          <Checkbox
            v-model="settings.allow_uploader_set_torrent_bonus_points_cost"
            name="allow_uploader_set_torrent_bonus_points_cost"
            :binary="true"
            inputId="allow_uploader_set_torrent_bonus_points_cost"
            style="margin-right: 5px"
          />
          <label for="allow_uploader_set_torrent_bonus_points_cost">{{ t('arcadia_settings.allow_uploader_set_torrent_bonus_points_cost') }}</label>
        </div>

        <FloatLabel>
          <InputNumber v-model="displayTorrentBonusPointsCostMin" name="torrent_bonus_points_cost_min" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.torrent_bonus_points_cost_min') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber v-model="displayTorrentBonusPointsCostMax" name="torrent_bonus_points_cost_max" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.torrent_bonus_points_cost_max') }}</label>
        </FloatLabel>

        <FloatLabel>
          <Select
            v-model="settings.snatched_torrent_bonus_points_transferred_to"
            :options="snatchBonusTransferOptions"
            optionLabel="label"
            optionValue="value"
            name="snatched_torrent_bonus_points_transferred_to"
            size="small"
            style="width: 25em"
          />
          <label>{{ t('arcadia_settings.snatched_torrent_bonus_points_transferred_to') }}</label>
        </FloatLabel>

        <FloatLabel>
          <DatePicker v-model="torrentMaxReleaseDateAllowed" name="torrent_max_release_date_allowed" dateFormat="yy-mm-dd" size="small" showButtonBar />
          <label>{{ t('arcadia_settings.torrent_max_release_date_allowed') }}</label>
        </FloatLabel>

        <BBCodeEditor
          :label="t('arcadia_settings.upload_page_top_text')"
          :initialValue="settings.upload_page_top_text ?? ''"
          :rows="4"
          @valueChange="(val) => (settings!.upload_page_top_text = val || null)"
          style="margin-top: 15px"
        />

        <BBCodeEditor
          :label="t('arcadia_settings.automated_message_on_torrent_marked_trumpable')"
          :initialValue="settings.automated_message_on_torrent_marked_trumpable ?? ''"
          :rows="4"
          @valueChange="(val) => (settings!.automated_message_on_torrent_marked_trumpable = val || null)"
          style="margin-top: 15px"
        />
      </ContentContainer>

      <ContentContainer class="settings-section" :containerTitle="t('arcadia_settings.signup_settings')">
        <Checkbox v-model="settings.open_signups" name="open_signups" :binary="true" inputId="open_signups" style="margin-right: 5px" />
        <label for="open_signups">{{ t('arcadia_settings.open_signups') }}</label>

        <FloatLabel>
          <InputNumber v-model="settings.invitation_expiration_days" name="invitation_expiration_days" :min="1" :step="1" size="small" showButtons />
          <label>{{ t('arcadia_settings.invitation_expiration_days') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber v-model="settings.default_user_uploaded_on_registration" name="default_user_uploaded_on_registration" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.default_user_uploaded_on_registration') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber
            v-model="settings.default_user_downloaded_on_registration"
            name="default_user_downloaded_on_registration"
            :min="0"
            :step="1"
            size="small"
          />
          <label>{{ t('arcadia_settings.default_user_downloaded_on_registration') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber v-model="displayDefaultBonusPointsOnRegistration" name="default_user_bonus_points_on_registration" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.default_user_bonus_points_on_registration') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber
            v-model="settings.default_user_freeleech_tokens_on_registration"
            name="default_user_freeleech_tokens_on_registration"
            :min="0"
            :step="1"
            size="small"
          />
          <label>{{ t('arcadia_settings.default_user_freeleech_tokens_on_registration') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber
            v-model="settings.inactive_user_ban_after_days"
            name="inactive_user_ban_after_days"
            :min="1"
            :step="1"
            style="width: 30em"
            size="small"
            showButtons
          />
          <label>{{ t('arcadia_settings.inactive_user_ban_after_days') }}</label>
        </FloatLabel>

        <BBCodeEditor
          v-tooltip.top="t('arcadia_settings.automated_message_on_signup_placeholders_hint')"
          :label="t('arcadia_settings.automated_message_on_signup')"
          :initialValue="settings.automated_message_on_signup ?? ''"
          :rows="4"
          @valueChange="(val) => (settings!.automated_message_on_signup = val || null)"
          style="margin-top: 15px"
        />
        <FloatLabel>
          <InputText
            v-model="settings.automated_message_on_signup_conversation_name"
            name="automated_message_on_signup_conversation_name"
            size="small"
            style="width: 20em"
          />
          <label>{{ t('arcadia_settings.automated_message_on_signup_conversation_name') }}</label>
        </FloatLabel>
        <Message v-if="$form.automated_message_fields?.invalid" severity="error" size="small" variant="simple">
          {{ $form.automated_message_fields.error?.message }}
        </Message>

        <FloatLabel>
          <InputNumber v-model="settings.automated_message_on_signup_sender_id" name="automated_message_on_signup_sender_id" :min="1" :step="1" size="small" />
          <label>{{ t('arcadia_settings.automated_message_on_signup_sender_id') }}</label>
        </FloatLabel>

        <div>
          <Checkbox
            v-model="settings.automated_message_on_signup_locked"
            name="automated_message_on_signup_locked"
            :binary="true"
            inputId="automated_message_on_signup_locked"
            style="margin-top: 10px; margin-right: 5px"
          />
          <label for="automated_message_on_signup_locked">{{ t('arcadia_settings.automated_message_on_signup_locked') }}</label>
        </div>
      </ContentContainer>

      <ContentContainer class="settings-section" :containerTitle="t('arcadia_settings.irc_settings')">
        <div style="margin-bottom: 10px">
          <Checkbox v-model="settings.irc_webchat_enabled" name="irc_webchat_enabled" :binary="true" inputId="irc_webchat_enabled" style="margin-right: 5px" />
          <label for="irc_webchat_enabled">{{ t('arcadia_settings.irc_webchat_enabled') }}</label>
        </div>

        <FloatLabel>
          <Chips v-model="settings.irc_webchat_default_channels" name="irc_webchat_default_channels" separator="," size="small" style="width: 40em" />
          <label>{{ t('arcadia_settings.irc_webchat_default_channels') }}</label>
        </FloatLabel>
      </ContentContainer>

      <ContentContainer class="settings-section" :containerTitle="t('arcadia_settings.shop_settings')">
        <FloatLabel>
          <MultiSelect
            v-model="settings.available_shop_items"
            :options="
              Object.entries(AvailableShopItem).map(([_, value]) => ({
                label: value,
                value,
              }))
            "
            optionLabel="label"
            optionValue="value"
            name="available_shop_items"
            size="small"
            display="chip"
            class="available-shop-items"
          />
          <label>{{ t('arcadia_settings.available_shop_items') }}</label>
        </FloatLabel>

        <FloatLabel>
          <InputNumber v-model="displayShopFreeleechTokenBasePrice" name="shop_freeleech_token_base_price" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.shop_freeleech_token_base_price') }}</label>
        </FloatLabel>

        <div style="margin-top: 15px">
          <label>{{ t('arcadia_settings.shop_freeleech_token_discount_tiers') }}</label>
          <ShopDiscountTiersEditor v-model="settings.shop_freeleech_token_discount_tiers" tierType="freeleech" />
        </div>

        <FloatLabel>
          <InputNumber v-model="displayShopUploadBasePricePerGb" name="shop_upload_base_price_per_gb" :min="0" :step="1" size="small" />
          <label>{{ t('arcadia_settings.shop_upload_base_price_per_gb') }}</label>
        </FloatLabel>

        <div style="margin-top: 15px">
          <label>{{ t('arcadia_settings.shop_upload_discount_tiers') }}</label>
          <ShopDiscountTiersEditor v-model="settings.shop_upload_discount_tiers" tierType="upload" />
        </div>
      </ContentContainer>

      <Textarea
        v-model="settings.custom_js_code"
        name="custom_js_code"
        :rows="10"
        style="width: 100%; font-family: monospace"
        :placeholder="t('arcadia_settings.custom_js_code_hint')"
      />

      <Textarea
        v-model="settings.custom_footer"
        name="custom_footer"
        :rows="4"
        style="width: 100%; margin-top: 15px"
        :placeholder="t('arcadia_settings.custom_footer_hint')"
      />

      <div class="form-actions" style="margin-top: 20px">
        <Button type="submit" :label="t('general.save')" :loading="saving" />
      </div>
    </Form>
  </div>
</template>

<script setup lang="ts">
import { FloatLabel, InputNumber, Checkbox, Button, Message, Select, InputText, Chips, DatePicker, MultiSelect, Textarea } from 'primevue'
import BBCodeEditor from '@/components/community/BBCodeEditor.vue'
import ShopDiscountTiersEditor from '@/components/staff/ShopDiscountTiersEditor.vue'
import BonusPointsEndpointEditor from '@/components/staff/BonusPointsEndpointEditor.vue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import { useI18n } from 'vue-i18n'
import { ref, onMounted, computed } from 'vue'
import {
  getArcadiaSettings,
  updateArcadiaSettings,
  type ArcadiaSettings,
  getCSSSheets,
  getAllUserClasses,
  type CssSheet,
  type UserClass,
  SnatchedTorrentBonusPointsTransferredTo,
  DisplayedTopBarStats,
  DisplayableUserStats,
  TorrentRequestVoteCurrency,
  AvailableShopItem,
} from '@/services/api-schema'
import { rawToDisplayBp, displayToRawBp } from '@/services/helpers'
import { showToast } from '@/main'
import ContentContainer from '../ContentContainer.vue'

const { t } = useI18n()

const settings = ref<ArcadiaSettings>()
const cssSheets = ref<CssSheet[]>([])
const userClasses = ref<UserClass[]>([])

const snatchBonusTransferOptions = [
  { label: t('arcadia_settings.snatched_torrent_bonus_points_transferred_to_none'), value: null },
  { label: t('arcadia_settings.snatched_torrent_bonus_points_transferred_to_uploader'), value: SnatchedTorrentBonusPointsTransferredTo.Uploader },
  { label: t('arcadia_settings.snatched_torrent_bonus_points_transferred_to_seeders'), value: SnatchedTorrentBonusPointsTransferredTo.CurrentSeeders },
]

const saving = ref(false)

const bpDecimalPlaces = computed(() => settings.value?.bonus_points_decimal_places ?? 0)

const makeBpComputed = (field: keyof ArcadiaSettings) =>
  computed({
    get: () => rawToDisplayBp((settings.value?.[field] as number) ?? 0, bpDecimalPlaces.value),
    set: (value: number) => {
      if (settings.value) (settings.value[field] as number) = displayToRawBp(value, bpDecimalPlaces.value)
    },
  })

const displayBonusPointsGivenOnUpload = makeBpComputed('bonus_points_given_on_upload')
const displayDefaultTorrentBonusPointsCost = makeBpComputed('default_torrent_bonus_points_cost')
const displayTorrentBonusPointsCostMin = makeBpComputed('torrent_bonus_points_cost_min')
const displayTorrentBonusPointsCostMax = makeBpComputed('torrent_bonus_points_cost_max')
const displayShopFreeleechTokenBasePrice = makeBpComputed('shop_freeleech_token_base_price')
const displayShopUploadBasePricePerGb = makeBpComputed('shop_upload_base_price_per_gb')
const displayDefaultBonusPointsOnRegistration = makeBpComputed('default_user_bonus_points_on_registration')

const torrentMaxReleaseDateAllowed = computed({
  get: () => {
    if (!settings.value?.torrent_max_release_date_allowed) return null
    return new Date(settings.value.torrent_max_release_date_allowed)
  },
  set: (value: Date | null) => {
    if (!settings.value) return
    if (!value) {
      settings.value.torrent_max_release_date_allowed = null
      return
    }
    const year = value.getFullYear()
    const month = String(value.getMonth() + 1).padStart(2, '0')
    const day = String(value.getDate()).padStart(2, '0')
    settings.value.torrent_max_release_date_allowed = `${year}-${month}-${day}`
  },
})

const resolver = ({ values }: FormResolverOptions) => {
  const errors: Record<string, { message: string }[]> = {}

  if (!values.default_css_sheet_name || values.default_css_sheet_name.trim().length === 0) {
    errors.default_css_sheet_name = [{ message: t('error.field_required') }]
  }

  if (!values.user_class_name_on_signup || values.user_class_name_on_signup.trim().length === 0) {
    errors.user_class_name_on_signup = [{ message: t('error.field_required') }]
  }

  if (values.global_download_factor < 0) {
    errors.global_download_factor = [{ message: t('error.field_required') }]
  }

  if (values.global_upload_factor < 0) {
    errors.global_upload_factor = [{ message: t('error.field_required') }]
  }

  if (values.global_snatch_bonus_points_cost_factor < 0) {
    errors.global_snatch_bonus_points_cost_factor = [{ message: t('error.field_required') }]
  }

  if (values.global_snatch_bonus_points_reward_factor < 0) {
    errors.global_snatch_bonus_points_reward_factor = [{ message: t('error.field_required') }]
  }

  return { errors }
}

const saveSettings = async ({ valid }: FormSubmitEvent) => {
  if (!settings.value) return
  if (valid) {
    saving.value = true
    if (settings.value.logo_subtitle?.trim() === '') settings.value.logo_subtitle = null

    // set those values to null so they're not an empty string or a boolean that should be null
    if (!settings.value.automated_message_on_signup_conversation_name?.trim()) {
      settings.value.automated_message_on_signup_conversation_name = null
      settings.value.automated_message_on_signup_locked = null
    } else {
      // if the user tried to submit an incomplete form once, or never checked this box, it'll remain null
      // but visually, it looks like it's just unchecked, and therefore false. so we just set it to false
      if (!settings.value.automated_message_on_signup_locked) {
        settings.value.automated_message_on_signup_locked = false
      }
    }
    if (!settings.value.automated_message_on_signup?.trim()) {
      settings.value.automated_message_on_signup = null
    }
    if (!settings.value.custom_js_code?.trim()) {
      settings.value.custom_js_code = null
    }
    if (!settings.value.custom_footer?.trim()) {
      settings.value.custom_footer = null
    }
    if (!settings.value.automated_message_on_torrent_marked_trumpable?.trim()) {
      settings.value.automated_message_on_torrent_marked_trumpable = null
    }

    updateArcadiaSettings(settings.value)
      .then(() => {
        showToast('Success', t('arcadia_settings.settings_updated'), 'success', 4000)
      })
      .finally(() => {
        saving.value = false
      })
  }
}

onMounted(() => {
  Promise.all([getArcadiaSettings(), getCSSSheets(), getAllUserClasses()]).then(([arcadiaSettings, cssData, userClassesData]) => {
    settings.value = arcadiaSettings
    cssSheets.value = cssData.css_sheets
    userClasses.value = userClassesData
  })
})
</script>

<style scoped>
.settings-section {
  margin-top: 20px;
}
.displayed-top-bar-stats :deep(.p-multiselect-label),
.displayable-user-stats :deep(.p-multiselect-label),
.torrent-request-vote-currencies :deep(.p-multiselect-label),
.available-shop-items :deep(.p-multiselect-label) {
  display: flex;
  flex-wrap: wrap;
}
</style>
